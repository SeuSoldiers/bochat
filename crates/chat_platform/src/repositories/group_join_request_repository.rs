use crate::error::{AppError, AppResult};
use crate::models::{GroupJoinRequest, GroupJoinRequestListItem};
use sqlx::PgPool;

pub struct NewGroupJoinRequest<'a> {
    pub request_id: &'a str,
    pub group_id: &'a str,
    pub bot_id: &'a str,
    pub requester_user_id: &'a str,
    pub approver_user_id: &'a str,
    pub request_type: &'a str,
    pub request_reason: &'a str,
    pub now: &'a str,
}

pub struct GroupJoinRequestRepository;

impl GroupJoinRequestRepository {
    pub async fn insert(pool: &PgPool, req: &NewGroupJoinRequest<'_>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO group_join_requests (
                request_id, group_id, bot_id, requester_user_id, approver_user_id, request_type, request_reason, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8, $9)
            "#,
        )
        .bind(req.request_id)
        .bind(req.group_id)
        .bind(req.bot_id)
        .bind(req.requester_user_id)
        .bind(req.approver_user_id)
        .bind(req.request_type)
        .bind(req.request_reason)
        .bind(req.now)
        .bind(req.now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn find_by_id(pool: &PgPool, request_id: &str) -> AppResult<Option<GroupJoinRequest>> {
        sqlx::query_as(
            r#"
            SELECT request_id, group_id, bot_id, requester_user_id, approver_user_id, request_type, request_reason, status, review_note, created_at, updated_at, reviewed_at
            FROM group_join_requests
            WHERE request_id = $1
            "#,
        )
        .bind(request_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_pending(
        pool: &PgPool,
        group_id: &str,
        bot_id: &str,
        approver_user_id: &str,
        request_type: &str,
    ) -> AppResult<Option<GroupJoinRequest>> {
        sqlx::query_as(
            r#"
            SELECT request_id, group_id, bot_id, requester_user_id, approver_user_id, request_type, request_reason, status, review_note, created_at, updated_at, reviewed_at
            FROM group_join_requests
            WHERE group_id = $1
              AND bot_id = $2
              AND approver_user_id = $3
              AND request_type = $4
              AND status = 'pending'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(group_id)
        .bind(bot_id)
        .bind(approver_user_id)
        .bind(request_type)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_inbox(
        pool: &PgPool,
        approver_user_id: &str,
        status: Option<&str>,
    ) -> AppResult<Vec<GroupJoinRequestListItem>> {
        sqlx::query_as(
            r#"
            SELECT
                r.request_id, r.group_id, g.name AS group_name, g.group_code, r.bot_id, b.name AS bot_name,
                b.owner_id AS bot_owner_id, r.requester_user_id, r.approver_user_id, r.request_type,
                r.request_reason, r.status, r.review_note, r.created_at, r.updated_at, r.reviewed_at
            FROM group_join_requests r
            INNER JOIN groups g ON g.group_id = r.group_id
            INNER JOIN bots b ON b.bot_id = r.bot_id
            WHERE r.approver_user_id = $1
              AND ($2::text IS NULL OR r.status = $2)
            ORDER BY r.created_at DESC
            "#,
        )
        .bind(approver_user_id)
        .bind(status)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_outbox(
        pool: &PgPool,
        requester_user_id: &str,
        status: Option<&str>,
    ) -> AppResult<Vec<GroupJoinRequestListItem>> {
        sqlx::query_as(
            r#"
            SELECT
                r.request_id, r.group_id, g.name AS group_name, g.group_code, r.bot_id, b.name AS bot_name,
                b.owner_id AS bot_owner_id, r.requester_user_id, r.approver_user_id, r.request_type,
                r.request_reason, r.status, r.review_note, r.created_at, r.updated_at, r.reviewed_at
            FROM group_join_requests r
            INNER JOIN groups g ON g.group_id = r.group_id
            INNER JOIN bots b ON b.bot_id = r.bot_id
            WHERE r.requester_user_id = $1
              AND ($2::text IS NULL OR r.status = $2)
            ORDER BY r.created_at DESC
            "#,
        )
        .bind(requester_user_id)
        .bind(status)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn update_status(
        pool: &PgPool,
        request_id: &str,
        status: &str,
        review_note: Option<&str>,
        now: &str,
    ) -> AppResult<bool> {
        let affected = sqlx::query(
            r#"
            UPDATE group_join_requests
            SET status = $1, review_note = $2, reviewed_at = $3, updated_at = $3
            WHERE request_id = $4 AND status = 'pending'
            "#,
        )
        .bind(status)
        .bind(review_note)
        .bind(now)
        .bind(request_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .rows_affected();
        Ok(affected > 0)
    }
}
