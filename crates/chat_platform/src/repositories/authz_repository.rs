use crate::error::{AppError, AppResult};
use sqlx::SqlitePool;

pub struct AuthzRepository;

impl AuthzRepository {
    pub async fn bot_owned_by_account(pool: &SqlitePool, bot_id: &str, account: &str) -> AppResult<bool> {
        sqlx::query_scalar(
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
        .bind(account)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn user_matches_account(pool: &SqlitePool, user_id: &str, account: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = ? AND account = ?)")
            .bind(user_id)
            .bind(account)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_active_bot_ids_by_account(pool: &SqlitePool, account: &str) -> AppResult<Vec<String>> {
        sqlx::query_scalar(
            r#"
            SELECT b.bot_id
            FROM bots b
            INNER JOIN users u ON u.user_id = b.owner_id
            WHERE u.account = ? AND b.status = 'active'
            ORDER BY b.created_at ASC
            "#,
        )
        .bind(account)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn user_exists(pool: &SqlitePool, user_id: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE user_id = ?)")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
