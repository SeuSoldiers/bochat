use crate::error::AppResult;
use crate::models::Bot;
use sqlx::SqlitePool;

pub struct BotService;

impl BotService {
    pub async fn get_bot_by_id(pool: &SqlitePool, bot_id: &str) -> AppResult<Bot> {
        sqlx::query_as::<_, Bot>(
            "SELECT bot_id, bot_type, owner_id, name, token, created_at FROM bots WHERE bot_id = ?",
        )
        .bind(bot_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?
        .ok_or(crate::error::AppError::BotNotFound)
    }

    pub async fn get_user_bots(pool: &SqlitePool, user_id: &str) -> AppResult<Vec<Bot>> {
        sqlx::query_as::<_, Bot>(
            "SELECT bot_id, bot_type, owner_id, name, token, created_at FROM bots WHERE owner_id = ?"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    }
}
