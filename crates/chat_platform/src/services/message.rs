use crate::error::AppResult;
use crate::models::Message;
use crate::repositories::{GroupMessagesPage, MessageRepository};
use crate::services::FileManager;
use serde_json::Value;
use sqlx::SqlitePool;

pub struct MessageService;

impl MessageService {
    pub async fn get_message_by_id(pool: &SqlitePool, msg_id: i64) -> AppResult<Message> {
        MessageRepository::find_by_id(pool, msg_id)
            .await?
            .ok_or(crate::error::AppError::MessageNotFound)
    }

    pub async fn get_messages_for_group_before(
        pool: &SqlitePool,
        group_id: &str,
        base_id: i64,
        limit: i64,
    ) -> AppResult<Vec<Message>> {
        MessageRepository::list_by_group_before(
            pool,
            &GroupMessagesPage {
                group_id,
                base_id,
                limit,
            },
        )
        .await
    }

    pub async fn on_message_persisted(
        pool: &SqlitePool,
        msg_id: i64,
        msg_type: &str,
        content: &Value,
    ) -> AppResult<()> {
        FileManager::on_message_persisted(pool, msg_id, msg_type, content).await
    }
}
