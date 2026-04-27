use axum::{
    extract::{Extension, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::{json_response, AppError, AppResult},
    middlewares::UserAuth,
    models::AuditLogResponse,
    repositories::{AuditLogRepository, AuditLogsFilter},
    services::authz::user_is_super_admin,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct AuditLogsQuery {
    pub user_id: Option<String>,
    pub bot_id: Option<String>,
    pub group_id: Option<String>,
    pub action: Option<String>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn ensure_super_admin_only(state: &AppState, user_id: &str) -> AppResult<()> {
    if !user_is_super_admin(&state.pool, user_id).await? {
        return Err(AppError::Forbidden("只有超级管理员可以访问审计日志".to_string()));
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn list_audit_logs(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
    Query(query): Query<AuditLogsQuery>,
) -> AppResult<Response> {
    ensure_super_admin_only(&state, &auth.user_id).await?;

    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0).max(0);
    let logs = AuditLogRepository::list(
        &state.pool,
        &AuditLogsFilter {
            user_id: query.user_id.as_deref(),
            bot_id: query.bot_id.as_deref(),
            group_id: query.group_id.as_deref(),
            action: query.action.as_deref(),
            start_at: query.start_at.as_deref(),
            end_at: query.end_at.as_deref(),
            limit,
            offset,
        },
    )
    .await?;

    let responses: Vec<AuditLogResponse> = logs.into_iter().map(Into::into).collect();
    Ok(json_response(
        StatusCode::OK,
        json!({
            "logs": responses,
            "limit": limit,
            "offset": offset,
        }),
    ))
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[tracing::instrument(skip_all)]
pub async fn export_audit_logs_csv(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
    Query(query): Query<AuditLogsQuery>,
) -> AppResult<Response> {
    ensure_super_admin_only(&state, &auth.user_id).await?;

    let limit = query.limit.unwrap_or(10_000).clamp(1, 20_000);
    let logs = AuditLogRepository::list(
        &state.pool,
        &AuditLogsFilter {
            user_id: query.user_id.as_deref(),
            bot_id: query.bot_id.as_deref(),
            group_id: query.group_id.as_deref(),
            action: query.action.as_deref(),
            start_at: query.start_at.as_deref(),
            end_at: query.end_at.as_deref(),
            limit,
            offset: 0,
        },
    )
    .await?;

    let mut csv = String::from(
        "log_id,created_at,actor_type,actor_id,user_id,bot_id,group_id,action,resource_type,resource_id,details\n",
    );
    for log in logs {
        let details = log.details.unwrap_or_default();
        let row = format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            log.log_id,
            csv_escape(&log.created_at),
            csv_escape(&log.actor_type),
            csv_escape(&log.actor_id),
            csv_escape(log.user_id.as_deref().unwrap_or("")),
            csv_escape(log.bot_id.as_deref().unwrap_or("")),
            csv_escape(log.group_id.as_deref().unwrap_or("")),
            csv_escape(&log.action),
            csv_escape(&log.resource_type),
            csv_escape(log.resource_id.as_deref().unwrap_or("")),
            csv_escape(&details),
        );
        csv.push_str(&row);
    }

    let mut response = (StatusCode::OK, csv).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"audit_logs.csv\""),
    );
    Ok(response)
}
