use serde_json::Value;

use crate::{
    db::DbPool,
    repositories::{NewNotification, NotificationRepository},
    utils::generate_notification_id,
};

pub struct NotificationRecord<'a> {
    pub recipient_user_id: &'a str,
    pub kind: &'a str,
    pub title: &'a str,
    pub content: &'a str,
    pub requires_action: bool,
    pub action_payload: Option<Value>,
    pub related_request_id: Option<&'a str>,
    pub related_group_id: Option<&'a str>,
    pub related_bot_id: Option<&'a str>,
}

pub async fn create_best_effort(pool: &DbPool, record: NotificationRecord<'_>) {
    let now = chrono::Utc::now().to_rfc3339();
    let action_payload = record
        .action_payload
        .as_ref()
        .and_then(|payload| serde_json::to_string(payload).ok());

    if let Err(e) = NotificationRepository::insert(
        pool,
        &NewNotification {
            notification_id: &generate_notification_id(),
            recipient_user_id: record.recipient_user_id,
            kind: record.kind,
            title: record.title,
            content: record.content,
            requires_action: record.requires_action,
            action_payload: action_payload.as_deref(),
            related_request_id: record.related_request_id,
            related_group_id: record.related_group_id,
            related_bot_id: record.related_bot_id,
            now: &now,
        },
    )
    .await
    {
        tracing::warn!(
            "通知写入失败(已忽略): recipient_user_id={}, kind={}, error={}",
            record.recipient_user_id,
            record.kind,
            e
        );
    }
}

