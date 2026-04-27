use crate::db::DbPool;
use crate::repositories::{AuditLogRepository, NewAuditLog};
use serde_json::Value;

pub struct AuditRecord<'a> {
    pub actor_type: &'a str,
    pub actor_id: &'a str,
    pub user_id: Option<&'a str>,
    pub bot_id: Option<&'a str>,
    pub group_id: Option<&'a str>,
    pub action: &'a str,
    pub resource_type: &'a str,
    pub resource_id: Option<&'a str>,
    pub details: Option<Value>,
}

pub async fn record_best_effort(pool: &DbPool, record: AuditRecord<'_>) {
    let created_at = chrono::Utc::now().to_rfc3339();
    let details_owned = record.details.map(|v| v.to_string());
    let details_ref = details_owned.as_deref();

    let result = AuditLogRepository::insert(
        pool,
        &NewAuditLog {
            actor_type: record.actor_type,
            actor_id: record.actor_id,
            user_id: record.user_id,
            bot_id: record.bot_id,
            group_id: record.group_id,
            action: record.action,
            resource_type: record.resource_type,
            resource_id: record.resource_id,
            details: details_ref,
            created_at: &created_at,
        },
    )
    .await;

    if let Err(err) = result {
        tracing::error!("写入审计日志失败: {}", err);
    }
}
