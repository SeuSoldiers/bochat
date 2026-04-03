use crate::{
    db::DbPool,
    error::{AppError, AppResult},
};

use super::bootstrap::SUPER_ADMIN_ACCOUNT;

fn resolve_super_admin_account() -> String {
    std::env::var("SUPER_ADMIN_ACCOUNT").unwrap_or_else(|_| SUPER_ADMIN_ACCOUNT.to_string())
}

#[tracing::instrument(skip(pool))]
pub async fn bot_has_global_group_access(pool: &DbPool, bot_id: &str) -> AppResult<bool> {
    let super_admin_account = resolve_super_admin_account();

    let has_access: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM bots b
            INNER JOIN users u ON u.user_id = b.owner_id
            WHERE b.bot_id = ? AND u.account = ?
        )
        "#,
    )
    .bind(bot_id)
    .bind(&super_admin_account)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(has_access)
}

#[tracing::instrument(skip(pool))]
pub async fn user_is_super_admin(pool: &DbPool, user_id: &str) -> AppResult<bool> {
    let super_admin_account = resolve_super_admin_account();

    let is_super_admin: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = ? AND account = ?)")
            .bind(user_id)
            .bind(&super_admin_account)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(is_super_admin)
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

    let bot_ids: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT b.bot_id
        FROM bots b
        INNER JOIN users u ON u.user_id = b.owner_id
        WHERE u.account = ? AND b.status = 'active'
        ORDER BY b.created_at ASC
        "#,
    )
    .bind(&super_admin_account)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(bot_ids)
}
