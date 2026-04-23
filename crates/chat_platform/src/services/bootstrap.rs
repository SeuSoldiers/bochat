use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    db::DbPool,
    error::{AppError, AppResult},
    repositories::{BootstrapRepository, NewBot, SeedUserInsert},
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

    let user_insert_result = BootstrapRepository::insert_user_ignore_account_conflict(
        pool,
        &SeedUserInsert {
            user_id: &candidate_user_id,
            name: SUPER_ADMIN_NAME,
            account: &account,
            password_hash: &candidate_hash,
            id_number: &candidate_id_number,
            now: &now,
        },
    )
    .await?;

    let user_id: String = BootstrapRepository::find_user_id_by_account(pool, &account)
        .await?
        .ok_or_else(|| AppError::InternalError("超级管理员账号创建后未找到记录".to_string()))?;

    let bot_exists: bool =
        BootstrapRepository::bot_exists_by_owner_and_name(pool, &user_id, SUPER_ADMIN_BOT_NAME)
            .await?;

    let mut bot_created = false;
    if !bot_exists {
        let bot_id = generate_bot_id();
        let bot_secret = Uuid::new_v4().to_string();
        let bot_token = generate_token(&bot_id, &bot_secret)?;

        crate::repositories::BotRepository::insert(
            pool,
            &NewBot {
                bot_id: &bot_id,
                owner_id: &user_id,
                name: SUPER_ADMIN_BOT_NAME,
                description: Some(SUPER_ADMIN_BOT_DESCRIPTION),
                avatar_url: None,
                status: "active",
                token: &bot_token,
                secret: &bot_secret,
                now: &now,
            },
        )
        .await?;

        bot_created = true;
    }

    Ok(SuperAdminSeedResult {
        user_created: user_insert_result > 0,
        bot_created,
        account,
        password_source_env,
    })
}
