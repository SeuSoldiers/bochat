use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub notification_id: String,
    pub recipient_user_id: String,
    pub kind: String,
    pub title: String,
    pub content: String,
    pub requires_action: bool,
    pub is_resolved: bool,
    pub is_read: bool,
    pub action_payload: Option<String>,
    pub related_request_id: Option<String>,
    pub related_group_id: Option<String>,
    pub related_bot_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub read_at: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub notification_id: String,
    pub recipient_user_id: String,
    pub kind: String,
    pub title: String,
    pub content: String,
    pub requires_action: bool,
    pub is_resolved: bool,
    pub is_read: bool,
    pub action_payload: Option<Value>,
    pub related_request_id: Option<String>,
    pub related_group_id: Option<String>,
    pub related_bot_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub read_at: Option<String>,
    pub resolved_at: Option<String>,
}

impl From<Notification> for NotificationResponse {
    fn from(value: Notification) -> Self {
        let action_payload = value
            .action_payload
            .as_deref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .or_else(|| {
                value
                    .action_payload
                    .as_ref()
                    .map(|raw| Value::String(raw.clone()))
            });
        Self {
            notification_id: value.notification_id,
            recipient_user_id: value.recipient_user_id,
            kind: value.kind,
            title: value.title,
            content: value.content,
            requires_action: value.requires_action,
            is_resolved: value.is_resolved,
            is_read: value.is_read,
            action_payload,
            related_request_id: value.related_request_id,
            related_group_id: value.related_group_id,
            related_bot_id: value.related_bot_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            read_at: value.read_at,
            resolved_at: value.resolved_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationStatsResponse {
    pub unread_count: i64,
    pub pending_count: i64,
}

