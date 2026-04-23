use crate::{
    db::DbPool,
    error::{AppError, AppResult},
    repositories::AuthzRepository,
};

use super::bootstrap::SUPER_ADMIN_ACCOUNT;

fn resolve_super_admin_account() -> String {
    std::env::var("SUPER_ADMIN_ACCOUNT").unwrap_or_else(|_| SUPER_ADMIN_ACCOUNT.to_string())
}

#[tracing::instrument(skip(pool))]
pub async fn bot_has_global_group_access(pool: &DbPool, bot_id: &str) -> AppResult<bool> {
    let super_admin_account = resolve_super_admin_account();
    AuthzRepository::bot_owned_by_account(pool, bot_id, &super_admin_account).await
}

#[tracing::instrument(skip(pool))]
pub async fn user_is_super_admin(pool: &DbPool, user_id: &str) -> AppResult<bool> {
    let super_admin_account = resolve_super_admin_account();

    AuthzRepository::user_matches_account(pool, user_id, &super_admin_account).await
}

#[tracing::instrument(skip(pool))]
pub async fn can_manage_target_user(
    pool: &DbPool,
    requester_user_id: &str,
    target_user_id: &str,
) -> AppResult<bool> {
    if requester_user_id == target_user_id {
        return Ok(true);
    }

    user_is_super_admin(pool, requester_user_id).await
}

#[tracing::instrument(skip(pool))]
pub async fn list_super_admin_bot_ids(pool: &DbPool) -> AppResult<Vec<String>> {
    let super_admin_account = resolve_super_admin_account();

    AuthzRepository::list_active_bot_ids_by_account(pool, &super_admin_account).await
}

#[tracing::instrument(skip(pool))]
pub async fn ensure_user_exists(pool: &DbPool, user_id: &str) -> AppResult<()> {
    let exists: bool = AuthzRepository::user_exists(pool, user_id).await?;

    if !exists {
        return Err(AppError::InvalidUserToken);
    }

    Ok(())
}
