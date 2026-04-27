use crate::error::{AppError, AppResult};
use crate::models::Message;
use sqlx::PgPool;

pub struct GroupMessagesPage<'a> {
    pub group_id: &'a str,
    pub base_id: i64,
    pub limit: i64,
}

#[derive(Debug)]
pub struct MessageIdempotencyQuery<'a> {
    pub sender_bot_id: &'a str,
    pub group_id: &'a str,
    pub idempotency_key: &'a str,
}

pub struct NewMessage<'a> {
    pub msg_id: i64,
    pub group_id: &'a str,
    pub sender_id: &'a str,
    pub content: &'a str,
    pub msg_type: &'a str,
    pub idempotency_key: &'a str,
    pub created_at: &'a str,
    pub sender_name: &'a str,
    pub sender_avatar_url: Option<&'a str>,
}

#[derive(Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct MessageWithSenderRow {
    pub msg_id: i64,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub sender_avatar_url: Option<String>,
    pub content: String,
    pub msg_type: String,
    pub idempotency_key: Option<String>,
    pub created_at: String,
}

pub struct MessageRepository;

impl MessageRepository {
    pub async fn find_by_id(pool: &PgPool, msg_id: i64) -> AppResult<Option<Message>> {
        sqlx::query_as::<_, Message>(
            "SELECT msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at FROM messages WHERE msg_id = $1",
        )
        .bind(msg_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_by_group_before(
        pool: &PgPool,
        page: &GroupMessagesPage<'_>,
    ) -> AppResult<Vec<Message>> {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at
            FROM messages
            WHERE group_id = $1 AND msg_id < $2
            ORDER BY msg_id DESC
            LIMIT $3
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
        pool: &PgPool,
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
                m.idempotency_key,
                m.created_at
            FROM messages m
            LEFT JOIN bots b ON b.bot_id = m.sender_id
            WHERE m.sender_id = $1 AND m.group_id = $2 AND m.idempotency_key = $3
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
        pool: &PgPool,
        new_message: &NewMessage<'_>,
    ) -> AppResult<MessageWithSenderRow> {
        let row = sqlx::query_as::<_, MessageInsertRow>(
            r#"
            INSERT INTO messages (msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at
            "#,
        )
        .bind(new_message.msg_id)
        .bind(new_message.group_id)
        .bind(new_message.sender_id)
        .bind(new_message.content)
        .bind(new_message.msg_type)
        .bind(new_message.idempotency_key)
        .bind(new_message.created_at)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(MessageWithSenderRow {
            msg_id: row.msg_id,
            group_id: row.group_id,
            sender_id: row.sender_id,
            sender_name: Some(new_message.sender_name.to_string()),
            sender_avatar_url: new_message.sender_avatar_url.map(|s| s.to_string()),
            content: row.content,
            msg_type: row.msg_type,
            idempotency_key: row.idempotency_key,
            created_at: row.created_at,
        })
    }

    /// 按ID查询消息（带发送者信息）
    pub async fn find_by_id_enriched(
        pool: &PgPool,
        msg_id: i64,
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
                m.idempotency_key,
                m.created_at
            FROM messages m
            LEFT JOIN bots b ON b.bot_id = m.sender_id
            WHERE m.msg_id = $1
            "#,
        )
        .bind(msg_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    /// 获取群聊全量消息（带发送者信息），用于缓存预加载
    pub async fn list_all_enriched_by_group(
        pool: &PgPool,
        group_id: &str,
    ) -> AppResult<Vec<MessageWithSenderRow>> {
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
                m.idempotency_key,
                m.created_at
            FROM messages m
            LEFT JOIN bots b ON b.bot_id = m.sender_id
            WHERE m.group_id = $1
            ORDER BY m.msg_id DESC
            "#,
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_member_bot_ids(pool: &PgPool, group_id: &str) -> AppResult<Vec<String>> {
        sqlx::query_scalar("SELECT member_id FROM group_members WHERE group_id = $1")
            .bind(group_id)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}

/// INSERT...RETURNING 中只返回实际列（PostgreSQL 不支持 RETURNING 中的字面量绑定）
#[derive(sqlx::FromRow)]
struct MessageInsertRow {
    pub msg_id: i64,
    pub group_id: String,
    pub sender_id: String,
    pub content: String,
    pub msg_type: String,
    pub idempotency_key: Option<String>,
    pub created_at: String,
}
