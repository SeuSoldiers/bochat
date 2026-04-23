use crate::error::{AppError, AppResult};
use crate::models::Bot;
use sqlx::SqlitePool;

pub struct NewBot<'a> {
    pub bot_id: &'a str,
    pub owner_id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
    pub status: &'a str,
    pub token: &'a str,
    pub secret: &'a str,
    pub now: &'a str,
}

pub struct BotRepository;

impl BotRepository {
    pub async fn find_by_id(pool: &SqlitePool, bot_id: &str) -> AppResult<Option<Bot>> {
        sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?",
        )
        .bind(bot_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_required_by_id(pool: &SqlitePool, bot_id: &str) -> AppResult<Bot> {
        Self::find_by_id(pool, bot_id)
            .await?
            .ok_or(AppError::BotNotFound)
    }

    pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<Bot>> {
        sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_by_owner(pool: &SqlitePool, owner_id: &str) -> AppResult<Vec<Bot>> {
        sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE owner_id = ? ORDER BY created_at DESC",
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_secret_and_status(
        pool: &SqlitePool,
        bot_id: &str,
    ) -> AppResult<Option<(String, String)>> {
        sqlx::query_as("SELECT secret, status FROM bots WHERE bot_id = ?")
            .bind(bot_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_default_active_bot_id(
        pool: &SqlitePool,
        owner_id: &str,
    ) -> AppResult<Option<String>> {
        sqlx::query_scalar(
            "SELECT bot_id FROM bots WHERE owner_id = ? AND status = 'active' ORDER BY created_at ASC LIMIT 1",
        )
        .bind(owner_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn insert(pool: &SqlitePool, new_bot: &NewBot<'_>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO bots (bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(new_bot.bot_id)
        .bind(new_bot.owner_id)
        .bind(new_bot.name)
        .bind(new_bot.description)
        .bind(new_bot.avatar_url)
        .bind(new_bot.status)
        .bind(new_bot.token)
        .bind(new_bot.secret)
        .bind(new_bot.now)
        .bind(new_bot.now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_profile(
        pool: &SqlitePool,
        bot_id: &str,
        name: &str,
        description: Option<&str>,
        avatar_url: Option<&str>,
        updated_at: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE bots
            SET name = ?, description = ?, avatar_url = ?, updated_at = ?
            WHERE bot_id = ?
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(avatar_url)
        .bind(updated_at)
        .bind(bot_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_by_id(pool: &SqlitePool, bot_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM bots WHERE bot_id = ?")
            .bind(bot_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
