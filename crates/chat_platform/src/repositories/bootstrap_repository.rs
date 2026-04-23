use crate::error::{AppError, AppResult};
use sqlx::SqlitePool;

pub struct BootstrapRepository;

pub struct SeedUserInsert<'a> {
    pub user_id: &'a str,
    pub name: &'a str,
    pub account: &'a str,
    pub password_hash: &'a str,
    pub id_number: &'a str,
    pub now: &'a str,
}

impl BootstrapRepository {
    pub async fn insert_user_ignore_account_conflict(
        pool: &SqlitePool,
        payload: &SeedUserInsert<'_>,
    ) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (user_id, name, account, password_hash, id_number, avatar_url, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account) DO NOTHING
            "#,
        )
        .bind(payload.user_id)
        .bind(payload.name)
        .bind(payload.account)
        .bind(payload.password_hash)
        .bind(payload.id_number)
        .bind(None::<String>)
        .bind(payload.now)
        .bind(payload.now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    pub async fn find_user_id_by_account(pool: &SqlitePool, account: &str) -> AppResult<Option<String>> {
        sqlx::query_scalar("SELECT user_id FROM users WHERE account = ?")
            .bind(account)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn bot_exists_by_owner_and_name(pool: &SqlitePool, owner_id: &str, name: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM bots WHERE owner_id = ? AND name = ?)")
            .bind(owner_id)
            .bind(name)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
