use crate::error::{AppError, AppResult};
use crate::models::AuditLog;
use sqlx::PgPool;

pub struct NewAuditLog<'a> {
    pub actor_type: &'a str,
    pub actor_id: &'a str,
    pub user_id: Option<&'a str>,
    pub bot_id: Option<&'a str>,
    pub group_id: Option<&'a str>,
    pub action: &'a str,
    pub resource_type: &'a str,
    pub resource_id: Option<&'a str>,
    pub details: Option<&'a str>,
    pub created_at: &'a str,
}

pub struct AuditLogsFilter<'a> {
    pub user_id: Option<&'a str>,
    pub bot_id: Option<&'a str>,
    pub group_id: Option<&'a str>,
    pub action: Option<&'a str>,
    pub start_at: Option<&'a str>,
    pub end_at: Option<&'a str>,
    pub limit: i64,
    pub offset: i64,
}

pub struct AuditLogRepository;

impl AuditLogRepository {
    pub async fn insert(pool: &PgPool, log: &NewAuditLog<'_>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                actor_type, actor_id, user_id, bot_id, group_id, action, resource_type, resource_id, details, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(log.actor_type)
        .bind(log.actor_id)
        .bind(log.user_id)
        .bind(log.bot_id)
        .bind(log.group_id)
        .bind(log.action)
        .bind(log.resource_type)
        .bind(log.resource_id)
        .bind(log.details)
        .bind(log.created_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn list(pool: &PgPool, filter: &AuditLogsFilter<'_>) -> AppResult<Vec<AuditLog>> {
        sqlx::query_as(
            r#"
            SELECT
                log_id, actor_type, actor_id, user_id, bot_id, group_id, action,
                resource_type, resource_id, details, created_at
            FROM audit_logs
            WHERE ($1::text IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR bot_id = $2)
              AND ($3::text IS NULL OR group_id = $3)
              AND ($4::text IS NULL OR action = $4)
              AND ($5::text IS NULL OR created_at >= $5)
              AND ($6::text IS NULL OR created_at <= $6)
            ORDER BY created_at DESC, log_id DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(filter.user_id)
        .bind(filter.bot_id)
        .bind(filter.group_id)
        .bind(filter.action)
        .bind(filter.start_at)
        .bind(filter.end_at)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
