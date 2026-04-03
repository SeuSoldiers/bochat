use crate::{
    db::DbPool,
    error::{AppError, AppResult},
};

use super::bootstrap::SUPER_ADMIN_ACCOUNT;

#[tracing::instrument(skip(pool))]
pub async fn bot_has_global_group_access(pool: &DbPool, bot_id: &str) -> AppResult<bool> {
    let super_admin_account =
        std::env::var("SUPER_ADMIN_ACCOUNT").unwrap_or_else(|_| SUPER_ADMIN_ACCOUNT.to_string());

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
