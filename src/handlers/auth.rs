use actix_web::{web, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::RegisterRequest;
use crate::utils::{generate_bot_id, generate_token, generate_user_id};

/// 验证身份证号码格式：只检查位数
///
/// 参数:
/// - id_number: 要验证的身份证号
///
/// 返回:
/// - true: 如果身份证号是18位纯数字
/// - false: 否则
fn validate_id_number(id_number: &str) -> bool {
    // 标准身份证号是18位数字（暂时只检查位数和数字格式）
    let is_valid = id_number.len() == 18 && id_number.chars().all(|c| c.is_ascii_digit() || c == 'X');
    tracing::debug!("验证身份证号: {} -> {}", id_number, is_valid);
    is_valid
}

#[tracing::instrument(skip(pool))]
pub async fn register(
    pool: web::Data<DbPool>,
    req: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    // 记录注册请求
    tracing::info!("=== 开始处理用户注册请求 ===");
    tracing::debug!("请求数据: 姓名={}, 身份证号={}, 手机号={}", req.name, req.id_number, req.phone);

    // 验证必填字段
    if req.name.is_empty() || req.id_number.is_empty() || req.phone.is_empty() {
        tracing::warn!("注册失败: 缺少必填字段");
        return Err(AppError::BadRequest("缺少必填字段".to_string()));
    }

    // 验证身份证号格式 (18位)
    if !validate_id_number(&req.id_number) {
        tracing::warn!("注册失败: 身份证号格式无效 (期望18位): {}", req.id_number);
        return Err(AppError::InvalidIdNumber);
    }

    let user_id = generate_user_id();
    let now = chrono::Utc::now().to_rfc3339();

    tracing::debug!("生成新用户ID: {}", user_id);
    tracing::debug!("当前时间戳: {}", now);

    // 创建用户（实名认证）
    tracing::info!("正在数据库中创建用户记录: {}", user_id);
    sqlx::query(
        r#"
        INSERT INTO users (user_id, name, id_number, phone, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&req.name)
    .bind(&req.id_number)
    .bind(&req.phone)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("数据库错误: {}", e);
        if e.to_string().contains("UNIQUE") {
            tracing::warn!("身份证号已被使用: {}", req.id_number);
            AppError::IdNumberConflict
        } else {
            AppError::DatabaseError(e.to_string())
        }
    })?;

    tracing::info!("用户记录创建成功");

    // 为用户创建默认Bot
    let bot_id = generate_bot_id();
    let bot_secret = Uuid::new_v4().to_string();

    tracing::debug!("生成默认Bot信息:");
    tracing::debug!("  Bot ID: {}", bot_id);
    tracing::debug!("  Bot Secret: {} (前16位)", &bot_secret[..16.min(bot_secret.len())]);

    let bot_token = generate_token(&bot_id, &bot_secret)?;
    tracing::debug!("生成Bot Token: {} (前50位)", &bot_token[..50.min(bot_token.len())]);

    tracing::info!("正在数据库中创建Bot记录: {}", bot_id);
    sqlx::query(
        r#"
        INSERT INTO bots (bot_id, owner_id, name, description, status, token, secret, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&bot_id)
    .bind(&user_id)
    .bind(format!("{}的默认Bot", req.name))
    .bind(Some("用户注册时自动创建的默认Bot"))
    .bind("active")
    .bind(&bot_token)
    .bind(&bot_secret)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("创建Bot时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("Bot记录创建成功");

    tracing::info!(
        "✅ 用户注册成功 - 用户ID: {}, Bot ID: {}, 身份证号: {}",
        user_id,
        bot_id,
        req.id_number
    );

    Ok(HttpResponse::Created().json(json!({
        "message": "注册成功",
        "user_id": user_id,
        "name": req.name,
        "id_number": req.id_number,
        "phone": req.phone,
    })))
}

#[tracing::instrument(skip(pool))]
pub async fn get_user_by_id(
    pool: web::Data<DbPool>,
    user_id: &str,
) -> AppResult<crate::models::User> {
    tracing::debug!("查询用户信息: {}", user_id);

    let user = sqlx::query_as(
        "SELECT user_id, name, id_number, phone, created_at, updated_at FROM users WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询用户时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("用户不存在: {}", user_id);
        AppError::UserNotFound
    })?;

    tracing::debug!("用户查询成功: {}", user_id);
    Ok(user)
}

/// 用户登录
///
/// 流程:
/// 1. 验证身份证号和手机号
/// 2. 查询用户的默认 Bot
/// 3. 生成并返回 Token
#[tracing::instrument(skip(pool))]
pub async fn login(
    pool: web::Data<DbPool>,
    req: web::Json<crate::models::LoginRequest>,
) -> AppResult<HttpResponse> {
    tracing::info!("=== 开始处理用户登录请求 ===");
    tracing::debug!("请求数据: 身份证号={}, 手机号={}", req.id_number, req.phone);

    // 验证必填字段
    if req.id_number.is_empty() || req.phone.is_empty() {
        tracing::warn!("登录失败: 缺少必填字段");
        return Err(AppError::BadRequest("缺少必填字段".to_string()));
    }

    // 验证身份证号格式
    if !validate_id_number(&req.id_number) {
        tracing::warn!("登录失败: 身份证号格式无效: {}", req.id_number);
        return Err(AppError::InvalidIdNumber);
    }

    tracing::debug!("输入验证通过");

    // 查询用户
    tracing::info!("正在查询用户...");
    let user: crate::models::User = sqlx::query_as(
        "SELECT user_id, name, id_number, phone, created_at, updated_at FROM users WHERE id_number = ? AND phone = ?"
    )
    .bind(&req.id_number)
    .bind(&req.phone)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询用户时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("用户不存在或身份验证失败: 身份证号={}, 手机号={}", req.id_number, req.phone);
        AppError::UserNotFound
    })?;

    tracing::debug!("用户查询成功: {}", user.user_id);

    // 查询用户的默认 Bot（第一个创建的 bot）
    tracing::debug!("正在查询用户的默认 Bot...");
    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE owner_id = ? ORDER BY created_at ASC LIMIT 1"
    )
    .bind(&user.user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("用户的 Bot 不存在: {}", user.user_id);
        AppError::BotNotFound
    })?;

    tracing::debug!("Bot 查询成功: {}", bot.bot_id);

    tracing::info!(
        "✅ 用户登录成功 - 用户ID: {}, Bot ID: {}, 身份证号: {}",
        user.user_id,
        bot.bot_id,
        user.id_number
    );

    Ok(HttpResponse::Ok().json(json!({
        "message": "登录成功",
        "user_id": user.user_id,
        "name": user.name,
        "phone": user.phone,
        "bot_id": bot.bot_id,
        "token": bot.token,
    })))
}
