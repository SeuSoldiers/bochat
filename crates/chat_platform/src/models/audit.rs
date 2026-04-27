use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub log_id: i64,
    pub actor_type: String,
    pub actor_id: String,
    pub user_id: Option<String>,
    pub bot_id: Option<String>,
    pub group_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub log_id: i64,
    pub actor_type: String,
    pub actor_id: String,
    pub user_id: Option<String>,
    pub bot_id: Option<String>,
    pub group_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<Value>,
    pub created_at: String,
}

impl From<AuditLog> for AuditLogResponse {
    fn from(value: AuditLog) -> Self {
        let details = value
            .details
            .as_deref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .or_else(|| value.details.as_ref().map(|raw| Value::String(raw.clone())));
        Self {
            log_id: value.log_id,
            actor_type: value.actor_type,
            actor_id: value.actor_id,
            user_id: value.user_id,
            bot_id: value.bot_id,
            group_id: value.group_id,
            action: value.action,
            resource_type: value.resource_type,
            resource_id: value.resource_id,
            details,
            created_at: value.created_at,
        }
    }
}
