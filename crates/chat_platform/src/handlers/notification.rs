use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error::{json_response, AppError, AppResult},
    middlewares::UserAuth,
    models::{NotificationResponse, NotificationStatsResponse},
    repositories::{
        BotRepository, GroupJoinRequestRepository, GroupRepository, NewGroupMember,
        NotificationListFilter, NotificationRepository,
    },
    services::{
        audit::{record_best_effort, AuditRecord},
        authz::can_manage_target_user,
        notification::{create_best_effort as create_notification_best_effort, NotificationRecord},
    },
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub status: Option<String>,
    pub kind: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewByNotificationBody {
    pub note: Option<String>,
}

fn request_id_from_payload(payload: Option<&Value>) -> Option<String> {
    payload
        .and_then(|value| value.get("request_id"))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
}

#[tracing::instrument(skip_all)]
pub async fn list_notifications(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
    Query(query): Query<ListNotificationsQuery>,
) -> AppResult<Response> {
    let user_id = auth.user_id;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);

    let notifications = NotificationRepository::list(
        &state.pool,
        &user_id,
        &NotificationListFilter {
            status: query.status.as_deref(),
            kind: query.kind.as_deref(),
            limit,
            offset,
        },
    )
    .await?;
    let stats = NotificationRepository::stats(&state.pool, &user_id).await?;
    let payload: Vec<NotificationResponse> = notifications.into_iter().map(Into::into).collect();

    Ok(json_response(
        StatusCode::OK,
        json!({
            "notifications": payload,
            "stats": NotificationStatsResponse {
                unread_count: stats.unread_count,
                pending_count: stats.pending_count,
            },
            "limit": limit,
            "offset": offset,
        }),
    ))
}

#[tracing::instrument(skip_all)]
pub async fn mark_notification_read(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
    Path(notification_id): Path<String>,
) -> AppResult<Response> {
    let user_id = auth.user_id;
    let now = chrono::Utc::now().to_rfc3339();
    let updated = NotificationRepository::mark_read(&state.pool, &user_id, &notification_id, &now).await?;
    if !updated {
        return Err(AppError::NotificationNotFound);
    }
    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "通知已标记为已读",
            "notification_id": notification_id,
        }),
    ))
}

#[tracing::instrument(skip_all)]
pub async fn approve_notification_join_request(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
    Path(notification_id): Path<String>,
    Json(body): Json<ReviewByNotificationBody>,
) -> AppResult<Response> {
    review_notification_join_request(state, auth.user_id, notification_id, "approved", body.note).await
}

#[tracing::instrument(skip_all)]
pub async fn reject_notification_join_request(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
    Path(notification_id): Path<String>,
    Json(body): Json<ReviewByNotificationBody>,
) -> AppResult<Response> {
    review_notification_join_request(state, auth.user_id, notification_id, "rejected", body.note).await
}

async fn review_notification_join_request(
    state: AppState,
    requester_user_id: String,
    notification_id: String,
    status: &str,
    note: Option<String>,
) -> AppResult<Response> {
    let notification = NotificationRepository::find_by_id(&state.pool, &requester_user_id, &notification_id)
        .await?
        .ok_or(AppError::NotificationNotFound)?;

    if notification.kind != "group_invite_approval" || !notification.requires_action {
        return Err(AppError::BadRequest("该通知不支持审批操作".to_string()));
    }
    if notification.is_resolved {
        return Err(AppError::BadRequest("该通知已处理".to_string()));
    }

    let payload_value = notification
        .action_payload
        .as_deref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok());
    let request_id = notification
        .related_request_id
        .clone()
        .or_else(|| request_id_from_payload(payload_value.as_ref()))
        .ok_or(AppError::BadRequest("通知缺少关联申请".to_string()))?;

    let request = GroupJoinRequestRepository::find_by_id(&state.pool, &request_id)
        .await?
        .ok_or(AppError::JoinRequestNotFound)?;

    if !can_manage_target_user(&state.pool, &requester_user_id, &request.approver_user_id).await? {
        return Err(AppError::Forbidden("没有权限处理该申请".to_string()));
    }
    if request.status != "pending" {
        NotificationRepository::resolve_by_request_id(&state.pool, &request_id, &chrono::Utc::now().to_rfc3339())
            .await?;
        return Err(AppError::BadRequest("该申请已被处理".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();
    if status == "approved" {
        crate::repositories::GroupRepository::add_member_ignore(
            &state.pool,
            &NewGroupMember {
                group_id: &request.group_id,
                member_id: &request.bot_id,
                member_type: "bot",
                joined_at: &now,
            },
        )
        .await?;
    }

    let review_note = note.as_deref().map(str::trim).filter(|v| !v.is_empty());
    let updated =
        GroupJoinRequestRepository::update_status(&state.pool, &request_id, status, review_note, &now).await?;
    if !updated {
        return Err(AppError::BadRequest("该申请已被处理".to_string()));
    }

    let group_name = GroupRepository::find_by_id(&state.pool, &request.group_id)
        .await?
        .map(|group| group.name)
        .unwrap_or_else(|| request.group_id.clone());
    let bot_name = BotRepository::find_by_id(&state.pool, &request.bot_id)
        .await?
        .map(|bot| bot.name)
        .unwrap_or_else(|| request.bot_id.clone());

    create_notification_best_effort(
        &state.pool,
        NotificationRecord {
            recipient_user_id: &request.requester_user_id,
            kind: "group_invite_result",
            title: if status == "approved" {
                "加群申请已通过"
            } else {
                "加群申请已拒绝"
            },
            content: &format!(
                "Bot {} 的加群申请（群 {}）已被{}",
                bot_name,
                group_name,
                if status == "approved" { "同意" } else { "拒绝" }
            ),
            requires_action: false,
            action_payload: Some(json!({
                "request_id": request_id,
                "group_id": request.group_id,
                "group_name": group_name,
                "bot_id": request.bot_id,
                "bot_name": bot_name,
                "status": status
            })),
            related_request_id: Some(&request_id),
            related_group_id: Some(&request.group_id),
            related_bot_id: Some(&request.bot_id),
        },
    )
    .await;

    NotificationRepository::resolve_by_request_id(&state.pool, &request_id, &now).await?;
    let _ = NotificationRepository::resolve(&state.pool, &requester_user_id, &notification_id, &now).await?;

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &requester_user_id,
            user_id: Some(&requester_user_id),
            bot_id: Some(&request.bot_id),
            group_id: Some(&request.group_id),
            action: if status == "approved" {
                "notification.join_request.approve"
            } else {
                "notification.join_request.reject"
            },
            resource_type: "group_join_request",
            resource_id: Some(&request_id),
            details: Some(json!({ "notification_id": notification_id })),
        },
    )
    .await;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": if status == "approved" { "已同意申请" } else { "已拒绝申请" },
            "notification_id": notification_id,
            "request_id": request_id,
            "group_id": request.group_id,
            "bot_id": request.bot_id,
            "status": status,
        }),
    ))
}
