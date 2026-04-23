use crate::error::{AppError, AppResult};
use crate::models::Message;
use sqlx::SqlitePool;

pub struct GroupMessagesPage<'a> {
    pub group_id: &'a str,
    pub base_id: i64,
    pub limit: i64,
}

pub struct MessageIdempotencyQuery<'a> {
    pub sender_bot_id: &'a str,
    pub group_id: &'a str,
    pub idempotency_key: &'a str,
}

pub struct NewMessage<'a> {
    pub group_id: &'a str,
    pub sender_id: &'a str,
    pub content: &'a str,
    pub msg_type: &'a str,
    pub idempotency_key: &'a str,
    pub created_at: &'a str,
    pub sender_name: &'a str,
    pub sender_avatar_url: Option<&'a str>,
}

#[derive(sqlx::FromRow)]
pub struct MessageWithSenderRow {
    pub msg_id: i64,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub sender_avatar_url: Option<String>,
    pub content: String,
    pub msg_type: String,
    pub created_at: String,
}

pub struct MessageRepository;

impl MessageRepository {
    pub async fn find_by_id(pool: &SqlitePool, msg_id: i64) -> AppResult<Option<Message>> {
        sqlx::query_as::<_, Message>(
            "SELECT msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at FROM messages WHERE msg_id = ?",
        )
        .bind(msg_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_by_group_before(pool: &SqlitePool, page: &GroupMessagesPage<'_>) -> AppResult<Vec<Message>> {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at
            FROM messages
            WHERE group_id = ? AND msg_id < ?
            ORDER BY msg_id DESC
            LIMIT ?
            "#,
        )
        .bind(page.group_id)
        .bind(page.base_id)
        .bind(page.limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_idempotent_message(
        pool: &SqlitePool,
        query: &MessageIdempotencyQuery<'_>,
    ) -> AppResult<Option<MessageWithSenderRow>> {
        sqlx::query_as(
            r#"
            SELECT
                m.msg_id,
                m.group_id,
                m.sender_id,
                b.name as sender_name,
                b.avatar_url as sender_avatar_url,
                m.content,
                m.msg_type,
                m.created_at
            FROM messages m
            LEFT JOIN bots b ON b.bot_id = m.sender_id
            WHERE m.sender_id = ? AND m.group_id = ? AND m.idempotency_key = ?
            "#,
        )
        .bind(query.sender_bot_id)
        .bind(query.group_id)
        .bind(query.idempotency_key)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn insert_message_returning(
        pool: &SqlitePool,
        new_message: &NewMessage<'_>,
    ) -> AppResult<MessageWithSenderRow> {
        sqlx::query_as(
            r#"
            INSERT INTO messages (group_id, sender_id, content, msg_type, idempotency_key, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING
                msg_id,
                group_id,
                sender_id,
                ? as sender_name,
                ? as sender_avatar_url,
                content,
                msg_type,
                created_at
            "#,
        )
        .bind(new_message.group_id)
        .bind(new_message.sender_id)
        .bind(new_message.content)
        .bind(new_message.msg_type)
        .bind(new_message.idempotency_key)
        .bind(new_message.created_at)
        .bind(new_message.sender_name)
        .bind(new_message.sender_avatar_url)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_member_bot_ids(pool: &SqlitePool, group_id: &str) -> AppResult<Vec<String>> {
        sqlx::query_scalar("SELECT member_id FROM group_members WHERE group_id = ?")
            .bind(group_id)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
