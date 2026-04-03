use crate::error::AppResult;
use crate::models::Message;
use sqlx::SqlitePool;

pub struct MessageService;

impl MessageService {
    pub async fn get_message_by_id(pool: &SqlitePool, msg_id: i64) -> AppResult<Message> {
        sqlx::query_as::<_, Message>(
            "SELECT msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at FROM messages WHERE msg_id = ?"
        )
        .bind(msg_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?
        .ok_or(crate::error::AppError::MessageNotFound)
    }

    pub async fn get_messages_for_group_before(
        pool: &SqlitePool,
        group_id: &str,
        base_id: i64,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at
            FROM messages
            WHERE group_id = ? AND msg_id < ?
            ORDER BY msg_id DESC
            LIMIT ?
            "#,
        )
        .bind(group_id)
        .bind(base_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    }
}
