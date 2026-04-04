use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    db::DbPool,
    error::{AppError, AppResult},
    utils::{generate_bot_id, generate_token, generate_user_id},
};

pub const SUPER_ADMIN_ACCOUNT: &str = "super_admin";
const SUPER_ADMIN_NAME: &str = "超级管理员";
const SUPER_ADMIN_BOT_NAME: &str = "超级管理员Bot";
const SUPER_ADMIN_BOT_DESCRIPTION: &str = "系统启动自动创建，具备全群聊天权限";
const NONE_PREFIX: &str = "_none_";
const DEFAULT_SUPER_ADMIN_PASSWORD: &str = "Admin123456";

fn hash_password(password: &str, pepper: &str) -> String {
    let salt = Uuid::new_v4().simple().to_string();
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hasher.update(pepper.as_bytes());
    let digest = hasher.finalize();
    format!("{}${}", salt, hex::encode(digest))
}

fn placeholder_identifier(field: &str, user_id: &str) -> String {
    format!("{NONE_PREFIX}{field}_{user_id}")
}

#[derive(Debug, Clone)]
pub struct SuperAdminSeedResult {
    pub user_created: bool,
    pub bot_created: bool,
    pub account: String,
    pub password_source_env: bool,
}

#[tracing::instrument(skip(pool, jwt_secret))]
pub async fn ensure_super_admin_account(
    pool: &DbPool,
    jwt_secret: &str,
) -> AppResult<SuperAdminSeedResult> {
    let account =
        std::env::var("SUPER_ADMIN_ACCOUNT").unwrap_or_else(|_| SUPER_ADMIN_ACCOUNT.to_string());
    let password = std::env::var("SUPER_ADMIN_PASSWORD")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SUPER_ADMIN_PASSWORD.to_string());

    let password_source_env = std::env::var("SUPER_ADMIN_PASSWORD")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let now = chrono::Utc::now().to_rfc3339();
    let candidate_user_id = generate_user_id();
    let candidate_id_number = placeholder_identifier("id_number", &candidate_user_id);
    let candidate_hash = hash_password(&password, jwt_secret);

    let user_insert_result = sqlx::query(
        r#"
        INSERT INTO users (user_id, name, account, password_hash, id_number, avatar_url, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(account) DO NOTHING
        "#,
    )
    .bind(&candidate_user_id)
    .bind(SUPER_ADMIN_NAME)
    .bind(&account)
    .bind(&candidate_hash)
    .bind(&candidate_id_number)
    .bind(None::<String>)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let user_id: String = sqlx::query_scalar("SELECT user_id FROM users WHERE account = ?")
        .bind(&account)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::InternalError("超级管理员账号创建后未找到记录".to_string()))?;

    let bot_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM bots WHERE owner_id = ? AND name = ?)")
            .bind(&user_id)
            .bind(SUPER_ADMIN_BOT_NAME)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut bot_created = false;
    if !bot_exists {
        let bot_id = generate_bot_id();
        let bot_secret = Uuid::new_v4().to_string();
        let bot_token = generate_token(&bot_id, &bot_secret)?;

        sqlx::query(
            r#"
            INSERT INTO bots (bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&bot_id)
        .bind(&user_id)
        .bind(SUPER_ADMIN_BOT_NAME)
        .bind(Some(SUPER_ADMIN_BOT_DESCRIPTION))
        .bind(None::<String>)
        .bind("active")
        .bind(&bot_token)
        .bind(&bot_secret)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        bot_created = true;
    }

    Ok(SuperAdminSeedResult {
        user_created: user_insert_result.rows_affected() > 0,
        bot_created,
        account,
        password_source_env,
    })
}
