use crate::error::{AppError, AppResult};
use crate::models::Notification;
use sqlx::PgPool;

pub struct NewNotification<'a> {
    pub notification_id: &'a str,
    pub recipient_user_id: &'a str,
    pub kind: &'a str,
    pub title: &'a str,
    pub content: &'a str,
    pub requires_action: bool,
    pub action_payload: Option<&'a str>,
    pub related_request_id: Option<&'a str>,
    pub related_group_id: Option<&'a str>,
    pub related_bot_id: Option<&'a str>,
    pub now: &'a str,
}

pub struct NotificationListFilter<'a> {
    pub status: Option<&'a str>,
    pub kind: Option<&'a str>,
    pub limit: i64,
    pub offset: i64,
}

pub struct NotificationStats {
    pub unread_count: i64,
    pub pending_count: i64,
}

pub struct NotificationRepository;

impl NotificationRepository {
    pub async fn insert(pool: &PgPool, req: &NewNotification<'_>) -> AppResult<()> {
        let is_resolved = !req.requires_action;
        sqlx::query(
            r#"
            INSERT INTO notifications (
                notification_id, recipient_user_id, kind, title, content,
                requires_action, is_resolved, is_read, action_payload,
                related_request_id, related_group_id, related_bot_id,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, $8, $9, $10, $11, $12, $12)
            "#,
        )
        .bind(req.notification_id)
        .bind(req.recipient_user_id)
        .bind(req.kind)
        .bind(req.title)
        .bind(req.content)
        .bind(req.requires_action)
        .bind(is_resolved)
        .bind(req.action_payload)
        .bind(req.related_request_id)
        .bind(req.related_group_id)
        .bind(req.related_bot_id)
        .bind(req.now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn list(
        pool: &PgPool,
        recipient_user_id: &str,
        filter: &NotificationListFilter<'_>,
    ) -> AppResult<Vec<Notification>> {
        sqlx::query_as(
            r#"
            SELECT
                notification_id, recipient_user_id, kind, title, content,
                requires_action, is_resolved, is_read, action_payload,
                related_request_id, related_group_id, related_bot_id,
                created_at, updated_at, read_at, resolved_at
            FROM notifications
            WHERE recipient_user_id = $1
              AND ($2::text IS NULL
                OR ($2 = 'all')
                OR ($2 = 'unread' AND is_read = FALSE)
                OR ($2 = 'pending' AND requires_action = TRUE AND is_resolved = FALSE))
              AND ($3::text IS NULL OR kind = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(recipient_user_id)
        .bind(filter.status)
        .bind(filter.kind)
        .bind(filter.limit)
        .bind(filter.offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn stats(pool: &PgPool, recipient_user_id: &str) -> AppResult<NotificationStats> {
        let unread_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM notifications
            WHERE recipient_user_id = $1 AND is_read = FALSE
            "#,
        )
        .bind(recipient_user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let pending_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM notifications
            WHERE recipient_user_id = $1
              AND requires_action = TRUE
              AND is_resolved = FALSE
            "#,
        )
        .bind(recipient_user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(NotificationStats {
            unread_count,
            pending_count,
        })
    }

    pub async fn find_by_id(
        pool: &PgPool,
        recipient_user_id: &str,
        notification_id: &str,
    ) -> AppResult<Option<Notification>> {
        sqlx::query_as(
            r#"
            SELECT
                notification_id, recipient_user_id, kind, title, content,
                requires_action, is_resolved, is_read, action_payload,
                related_request_id, related_group_id, related_bot_id,
                created_at, updated_at, read_at, resolved_at
            FROM notifications
            WHERE notification_id = $1 AND recipient_user_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(recipient_user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn mark_read(
        pool: &PgPool,
        recipient_user_id: &str,
        notification_id: &str,
        now: &str,
    ) -> AppResult<bool> {
        let affected = sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = TRUE, read_at = COALESCE(read_at, $3), updated_at = $3
            WHERE notification_id = $1 AND recipient_user_id = $2
            "#,
        )
        .bind(notification_id)
        .bind(recipient_user_id)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        Ok(affected > 0)
    }

    pub async fn resolve(
        pool: &PgPool,
        recipient_user_id: &str,
        notification_id: &str,
        now: &str,
    ) -> AppResult<bool> {
        let affected = sqlx::query(
            r#"
            UPDATE notifications
            SET is_resolved = TRUE, is_read = TRUE, resolved_at = $3, read_at = COALESCE(read_at, $3), updated_at = $3
            WHERE notification_id = $1 AND recipient_user_id = $2 AND requires_action = TRUE AND is_resolved = FALSE
            "#,
        )
        .bind(notification_id)
        .bind(recipient_user_id)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        Ok(affected > 0)
    }

    pub async fn resolve_by_request_id(
        pool: &PgPool,
        request_id: &str,
        now: &str,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications
            SET is_resolved = TRUE, is_read = TRUE, resolved_at = $2, read_at = COALESCE(read_at, $2), updated_at = $2
            WHERE related_request_id = $1 AND requires_action = TRUE AND is_resolved = FALSE
            "#,
        )
        .bind(request_id)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
