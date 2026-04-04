use axum::{extract::State, http::StatusCode, response::Response, Json};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::models::RegisterRequest;
use crate::utils::{generate_bot_id, generate_token, generate_user_id, generate_user_token};
use crate::{
    error::{json_response, AppError, AppResult},
    AppState,
};

const NONE_PREFIX: &str = "_none_";
const ACCOUNT_MIN_LEN: usize = 4;
const ACCOUNT_MAX_LEN: usize = 32;
const PASSWORD_MIN_LEN: usize = 8;
const PASSWORD_MAX_LEN: usize = 64;

fn sanitize_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}

fn validate_account(account: &str) -> bool {
    let len = account.len();
    if !(ACCOUNT_MIN_LEN..=ACCOUNT_MAX_LEN).contains(&len) {
        return false;
    }

    account
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn validate_password(password: &str) -> bool {
    let len = password.len();
    if !(PASSWORD_MIN_LEN..=PASSWORD_MAX_LEN).contains(&len) {
        return false;
    }

    let has_letter = password.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let all_printable_ascii = password.chars().all(|c| c.is_ascii_graphic());

    has_letter && has_digit && all_printable_ascii
}

fn hash_password(password: &str, pepper: &str) -> String {
    let salt = Uuid::new_v4().simple().to_string();
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hasher.update(pepper.as_bytes());
    let digest = hasher.finalize();
    format!("{}${}", salt, hex::encode(digest))
}

fn verify_password(password: &str, password_hash: &str, pepper: &str) -> bool {
    let Some((salt, hash)) = password_hash.split_once('$') else {
        return false;
    };

    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hasher.update(pepper.as_bytes());
    let digest = hasher.finalize();
    hex::encode(digest) == hash
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
    let account = sanitize_optional(&req.account)
        .ok_or_else(|| AppError::BadRequest("账号不能为空".to_string()))?;
    let password = sanitize_optional(&req.password)
        .ok_or_else(|| AppError::BadRequest("密码不能为空".to_string()))?;

    if !validate_account(&account) {
        return Err(AppError::BadRequest(format!(
            "账号格式不正确：长度需为{}-{}位，仅支持字母、数字、下划线",
            ACCOUNT_MIN_LEN, ACCOUNT_MAX_LEN
        )));
    }

    if !validate_password(&password) {
        return Err(AppError::BadRequest(format!(
            "密码格式不正确：长度需为{}-{}位，且必须包含字母和数字，仅支持可见 ASCII 字符",
            PASSWORD_MIN_LEN, PASSWORD_MAX_LEN
        )));
    }

    tracing::debug!(
        "请求数据: 昵称={}, 账号={}，密码长度={}",
        name,
        account,
        password.len()
    );

    let user_id = generate_user_id();
    let now = chrono::Utc::now().to_rfc3339();
    let db_id_number = to_db_identifier("id_number", None, &user_id);
    let password_hash = hash_password(&password, &state.config.security.jwt_secret);

    tracing::debug!("生成新用户ID: {}", user_id);
    tracing::debug!("当前时间戳: {}", now);

    // 创建用户（实名认证）
    tracing::info!("正在数据库中创建用户记录: {}", user_id);
    sqlx::query(
        r#"
        INSERT INTO users (user_id, name, account, password_hash, id_number, avatar_url, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&name)
    .bind(&account)
    .bind(&password_hash)
    .bind(&db_id_number)
    .bind(None::<String>)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("数据库错误: {}", e);
        let err_msg = e.to_string();
        if err_msg.contains("users.account") || err_msg.contains("idx_users_account_unique") {
            AppError::AccountConflict
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
            "account": account,
            "token": generate_user_token(&user_id, &state.config.security.jwt_secret)?,
            "created_at": now,
        }),
    ))
}

#[derive(sqlx::FromRow)]
struct UserLoginRow {
    user_id: String,
    name: String,
    password_hash: Option<String>,
}

#[tracing::instrument(skip(pool))]
pub async fn get_user_by_id(
    pool: &crate::db::DbPool,
    user_id: &str,
) -> AppResult<crate::models::User> {
    tracing::debug!("查询用户信息: {}", user_id);

    let user = sqlx::query_as(
        "SELECT user_id, name, id_number, avatar_url, created_at, updated_at FROM users WHERE user_id = ?"
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
/// 1. 验证账号和密码格式
/// 2. 查询用户并校验密码
/// 3. 生成并返回 Token
#[tracing::instrument(skip(state))]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<crate::models::LoginRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始处理用户登录请求 ===");
    let account = sanitize_optional(&req.account)
        .ok_or_else(|| AppError::BadRequest("账号不能为空".to_string()))?;
    let password = sanitize_optional(&req.password)
        .ok_or_else(|| AppError::BadRequest("密码不能为空".to_string()))?;

    if !validate_account(&account) {
        return Err(AppError::BadRequest(format!(
            "账号格式不正确：长度需为{}-{}位，仅支持字母、数字、下划线",
            ACCOUNT_MIN_LEN, ACCOUNT_MAX_LEN
        )));
    }

    if !validate_password(&password) {
        return Err(AppError::BadRequest(format!(
            "密码格式不正确：长度需为{}-{}位，且必须包含字母和数字，仅支持可见 ASCII 字符",
            PASSWORD_MIN_LEN, PASSWORD_MAX_LEN
        )));
    }

    tracing::debug!("请求数据: 账号={}, 密码长度={}", account, password.len());

    tracing::debug!("输入验证通过");

    // 查询用户
    tracing::info!("正在查询用户...");
    let user = sqlx::query_as::<_, UserLoginRow>(
        "SELECT user_id, name, password_hash FROM users WHERE account = ?",
    )
    .bind(&account)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询用户时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("登录失败: 账号不存在");
        AppError::InvalidCredentials
    })?;

    let Some(stored_hash) = user.password_hash.as_deref() else {
        tracing::warn!("登录失败: 账号未设置密码");
        return Err(AppError::InvalidCredentials);
    };

    if !verify_password(&password, stored_hash, &state.config.security.jwt_secret) {
        tracing::warn!("登录失败: 密码校验不通过");
        return Err(AppError::InvalidCredentials);
    }

    tracing::debug!("用户查询成功: {}", user.user_id);

    let user_token = generate_user_token(&user.user_id, &state.config.security.jwt_secret)?;
    tracing::info!("✅ 用户登录成功 - 用户ID: {}", user.user_id);

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "登录成功",
            "name": user.name,
            "token": user_token,
        }),
    ))
}
