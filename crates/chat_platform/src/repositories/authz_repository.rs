use crate::error::{AppError, AppResult};
use sqlx::PgPool;

pub struct AuthzRepository;

impl AuthzRepository {
    pub async fn bot_owned_by_account(pool: &PgPool, bot_id: &str, account: &str) -> AppResult<bool> {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM bots b
                INNER JOIN users u ON u.user_id = b.owner_id
                WHERE b.bot_id = $1 AND u.account = $2
            )
            "#,
        )
        .bind(bot_id)
        .bind(account)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn user_matches_account(pool: &PgPool, user_id: &str, account: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1 AND account = $2)")
            .bind(user_id)
            .bind(account)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_active_bot_ids_by_account(pool: &PgPool, account: &str) -> AppResult<Vec<String>> {
        sqlx::query_scalar(
            r#"
            SELECT b.bot_id
            FROM bots b
            INNER JOIN users u ON u.user_id = b.owner_id
            WHERE u.account = $1 AND b.status = 'active'
            ORDER BY b.created_at ASC
            "#,
        )
        .bind(account)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn user_exists(pool: &PgPool, user_id: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
