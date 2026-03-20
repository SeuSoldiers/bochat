use axum::{extract::State, http::StatusCode, response::Response, Json};
use serde_json::json;
use uuid::Uuid;

use crate::models::RegisterRequest;
use crate::utils::{generate_bot_id, generate_token, generate_user_id, generate_user_token};
use crate::{
    error::{json_response, AppError, AppResult},
    AppState,
};

const NONE_PREFIX: &str = "_none_";

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
    let is_valid =
        id_number.len() == 18 && id_number.chars().all(|c| c.is_ascii_digit() || c == 'X');
    tracing::debug!("验证身份证号: {} -> {}", id_number, is_valid);
    is_valid
}

fn sanitize_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}

fn to_db_identifier(field: &str, value: Option<&str>, user_id: &str) -> String {
    match value {
        Some(v) => v.to_string(),
        None => format!("{NONE_PREFIX}{field}_{user_id}"),
    }
}

fn default_user_name() -> String {
    let short_uuid = Uuid::new_v4().simple().to_string();
    format!("用户-{}", &short_uuid[..8])
}

#[tracing::instrument(skip(state))]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Response> {
    // 记录注册请求
    tracing::info!("=== 开始处理用户注册请求 ===");
    let name = req
        .name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(default_user_name);
    let phone = sanitize_optional(&req.phone);
    let id_number = sanitize_optional(&req.id_number);

    tracing::debug!(
        "请求数据: 姓名={}, 身份证号存在={}, 手机号存在={}",
        name,
        id_number.is_some(),
        phone.is_some()
    );

    if phone.is_none() && id_number.is_none() {
        tracing::warn!("注册失败: 手机号和身份证号至少填写一项");
        return Err(AppError::BadRequest(
            "手机号和身份证号至少填写一项".to_string(),
        ));
    }

    if let Some(ref id) = id_number {
        if !validate_id_number(id) {
            tracing::warn!("注册失败: 身份证号格式无效 (期望18位): {}", id);
            return Err(AppError::InvalidIdNumber);
        }
    }

    let user_id = generate_user_id();
    let now = chrono::Utc::now().to_rfc3339();
    let db_phone = to_db_identifier("phone", phone.as_deref(), &user_id);
    let db_id_number = to_db_identifier("id_number", id_number.as_deref(), &user_id);

    tracing::debug!("生成新用户ID: {}", user_id);
    tracing::debug!("当前时间戳: {}", now);

    // 创建用户（实名认证）
    tracing::info!("正在数据库中创建用户记录: {}", user_id);
    sqlx::query(
        r#"
        INSERT INTO users (user_id, name, id_number, phone, avatar_url, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&name)
    .bind(&db_id_number)
    .bind(&db_phone)
    .bind(None::<String>)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("数据库错误: {}", e);
        let err_msg = e.to_string();
        if err_msg.contains("users.id_number") {
            AppError::IdNumberConflict
        } else if err_msg.contains("users.phone") {
            AppError::PhoneConflict
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
    tracing::debug!(
        "  Bot Secret: {} (前16位)",
        &bot_secret[..16.min(bot_secret.len())]
    );

    let bot_token = generate_token(&bot_id, &bot_secret)?;
    tracing::debug!(
        "生成Bot Token: {} (前50位)",
        &bot_token[..50.min(bot_token.len())]
    );

    tracing::info!("正在数据库中创建Bot记录: {}", bot_id);
    sqlx::query(
        r#"
        INSERT INTO bots (bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&bot_id)
    .bind(&user_id)
    .bind(format!("{}的默认Bot", name))
    .bind(Some("用户注册时自动创建的默认Bot"))
    .bind(None::<String>)
    .bind("active")
    .bind(&bot_token)
    .bind(&bot_secret)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("创建Bot时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("Bot记录创建成功");

    tracing::info!("✅ 用户注册成功 - 用户ID: {}, Bot ID: {}", user_id, bot_id);

    Ok(json_response(
        StatusCode::CREATED,
        json!({
            "message": "注册成功",
            "name": name,
            "phone": phone,
            "token": generate_user_token(&user_id, &state.config.security.jwt_secret)?,
            "created_at": now,
        }),
    ))
}

#[tracing::instrument(skip(pool))]
pub async fn get_user_by_id(
    pool: &crate::db::DbPool,
    user_id: &str,
) -> AppResult<crate::models::User> {
    tracing::debug!("查询用户信息: {}", user_id);

    let user = sqlx::query_as(
        "SELECT user_id, name, id_number, phone, avatar_url, created_at, updated_at FROM users WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool)
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
#[tracing::instrument(skip(state))]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<crate::models::LoginRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始处理用户登录请求 ===");
    let phone = sanitize_optional(&req.phone);
    let id_number = sanitize_optional(&req.id_number);

    tracing::debug!(
        "请求数据: 身份证号存在={}, 手机号存在={}",
        id_number.is_some(),
        phone.is_some()
    );

    if phone.is_none() && id_number.is_none() {
        tracing::warn!("登录失败: 手机号和身份证号至少填写一项");
        return Err(AppError::BadRequest(
            "手机号和身份证号至少填写一项".to_string(),
        ));
    }

    if let Some(ref id) = id_number {
        if !validate_id_number(id) {
            tracing::warn!("登录失败: 身份证号格式无效: {}", id);
            return Err(AppError::InvalidIdNumber);
        }
    }

    tracing::debug!("输入验证通过");

    // 查询用户
    tracing::info!("正在查询用户...");
    let user = match (phone.as_deref(), id_number.as_deref()) {
        (Some(p), Some(id)) => {
            sqlx::query_as::<_, crate::models::User>(
                "SELECT user_id, name, id_number, phone, avatar_url, created_at, updated_at FROM users WHERE phone = ? OR id_number = ?"
            )
            .bind(p)
            .bind(id)
            .fetch_optional(&state.pool)
            .await
        }
        (Some(p), None) => {
            sqlx::query_as::<_, crate::models::User>(
                "SELECT user_id, name, id_number, phone, avatar_url, created_at, updated_at FROM users WHERE phone = ?"
            )
            .bind(p)
            .fetch_optional(&state.pool)
            .await
        }
        (None, Some(id)) => {
            sqlx::query_as::<_, crate::models::User>(
                "SELECT user_id, name, id_number, phone, avatar_url, created_at, updated_at FROM users WHERE id_number = ?"
            )
            .bind(id)
            .fetch_optional(&state.pool)
            .await
        }
        (None, None) => unreachable!(),
    }
    .map_err(|e| {
        tracing::error!("查询用户时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("用户不存在或身份验证失败");
        AppError::UserNotFound
    })?;

    tracing::debug!("用户查询成功: {}", user.user_id);

    let user_token = generate_user_token(&user.user_id, &state.config.security.jwt_secret)?;

    tracing::info!("✅ 用户登录成功 - 用户ID: {}", user.user_id);

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "登录成功",
            "name": user.name,
            "phone": crate::models::user::UserResponse::from(user.clone()).phone,
            "token": user_token,
        }),
    ))
}
