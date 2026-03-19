use sqlx::SqlitePool;
use crate::error::AppResult;
use crate::models::Message;

pub struct MessageService;

impl MessageService {
    pub async fn get_message_by_id(pool: &SqlitePool, msg_id: i64) -> AppResult<Message> {
        sqlx::query_as::<_, Message>(
            "SELECT msg_id, sender_id, to_id, content, msg_type, created_at FROM messages WHERE msg_id = ?"
        )
        .bind(msg_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?
        .ok_or(crate::error::AppError::MessageNotFound)
    }

    pub async fn get_messages_for_bot(
        pool: &SqlitePool,
        bot_id: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Message>> {
        sqlx::query_as::<_, Message>(
            "SELECT msg_id, sender_id, to_id, content, msg_type, created_at FROM messages WHERE to_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(bot_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    }
}
