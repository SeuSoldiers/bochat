use axum::{extract::State, http::StatusCode, response::Response, Json};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::models::RegisterRequest;
use crate::repositories::{BotRepository, NewBot, NewUser, UserRepository};
use crate::services::audit::{record_best_effort, AuditRecord};
use crate::services::authz::user_is_super_admin;
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

#[tracing::instrument(skip_all)]
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
    let new_user = NewUser {
        user_id: &user_id,
        name: &name,
        account: &account,
        password_hash: &password_hash,
        id_number: &db_id_number,
        avatar_url: None,
        now: &now,
    };
    UserRepository::insert_user(&state.pool, &new_user)
    .await
    .map_err(|e| {
        tracing::error!("数据库错误: {}", e);
        e
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
    let default_bot_name = format!("{}的默认Bot", name);
    let new_bot = NewBot {
        bot_id: &bot_id,
        owner_id: &user_id,
        name: &default_bot_name,
        description: Some("用户注册时自动创建的默认Bot"),
        avatar_url: None,
        status: "active",
        token: &bot_token,
        secret: &bot_secret,
        now: &now,
    };
    BotRepository::insert(&state.pool, &new_bot)
    .await
    .map_err(|e| {
        tracing::error!("创建Bot时数据库错误: {}", e);
        e
    })?;

    tracing::info!("Bot记录创建成功");

    tracing::info!("✅ 用户注册成功 - 用户ID: {}, Bot ID: {}", user_id, bot_id);

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &user_id,
            user_id: Some(&user_id),
            bot_id: Some(&bot_id),
            group_id: None,
            action: "auth.register",
            resource_type: "user",
            resource_id: Some(&user_id),
            details: Some(json!({ "account": account })),
        },
    )
    .await;

    Ok(json_response(
        StatusCode::CREATED,
        json!({
            "message": "注册成功",
            "name": name,
            "account": account,
            "is_super_admin": false,
            "token": generate_user_token(&user_id, &state.config.security.jwt_secret)?,
            "created_at": now,
        }),
    ))
}

#[tracing::instrument(skip_all)]
pub async fn get_user_by_id(
    pool: &crate::db::DbPool,
    user_id: &str,
) -> AppResult<crate::models::User> {
    tracing::debug!("查询用户信息: {}", user_id);

    let user = UserRepository::find_by_id(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("查询用户时数据库错误: {}", e);
            e
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
#[tracing::instrument(skip_all)]
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
    let user = UserRepository::find_login_by_account(&state.pool, &account)
        .await
        .map_err(|e| {
            tracing::error!("查询用户时数据库错误: {}", e);
            e
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
    let is_super_admin = user_is_super_admin(&state.pool, &user.user_id).await?;
    tracing::info!("✅ 用户登录成功 - 用户ID: {}", user.user_id);

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &user.user_id,
            user_id: Some(&user.user_id),
            bot_id: None,
            group_id: None,
            action: "auth.login",
            resource_type: "user",
            resource_id: Some(&user.user_id),
            details: Some(json!({ "account": account })),
        },
    )
    .await;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "登录成功",
            "name": user.name,
            "is_super_admin": is_super_admin,
            "token": user_token,
        }),
    ))
}
