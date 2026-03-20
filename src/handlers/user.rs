use axum::{
    extract::Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde_json::json;

use crate::utils::verify_user_token;
use crate::{
    error::{json_response, AppError, AppResult},
    http::{require_user_bearer_token, token_user_id},
    models::UpdateUserRequest,
    AppState,
};

const NONE_PREFIX: &str = "_none_";

fn normalize_optional_field(value: &str) -> Option<String> {
    if value.is_empty() || value.starts_with(NONE_PREFIX) {
        None
    } else {
        Some(value.to_string())
    }
}

fn to_db_identifier(field: &str, value: Option<&str>, user_id: &str) -> String {
    match value {
        Some(v) => v.to_string(),
        None => format!("{NONE_PREFIX}{field}_{user_id}"),
    }
}

fn validate_id_number(id_number: &str) -> bool {
    id_number.len() == 18 && id_number.chars().all(|c| c.is_ascii_digit() || c == 'X')
}

#[tracing::instrument(skip(state))]
pub async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let token = require_user_bearer_token(&headers)?;
    let user_id = token_user_id(&token)?.to_string();
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;

    let user: crate::models::User = sqlx::query_as(
        "SELECT user_id, name, id_number, phone, avatar_url, created_at, updated_at FROM users WHERE user_id = ?",
    )
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::UserNotFound)?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "name": user.name,
            "phone": normalize_optional_field(&user.phone),
            "id_number": normalize_optional_field(&user.id_number),
            "avatar_url": user.avatar_url,
            "created_at": user.created_at,
            "updated_at": user.updated_at,
        }),
    ))
}

#[tracing::instrument(skip(state, req))]
pub async fn update_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Response> {
    let token = require_user_bearer_token(&headers)?;
    let user_id = token_user_id(&token)?.to_string();
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;

    let current_user: crate::models::User = sqlx::query_as(
        "SELECT user_id, name, id_number, phone, avatar_url, created_at, updated_at FROM users WHERE user_id = ?",
    )
    .bind(&user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::UserNotFound)?;

    let next_name = match req.name.as_deref().map(str::trim) {
        Some(name) if name.is_empty() => {
            return Err(AppError::BadRequest("用户名不能为空".to_string()));
        }
        Some(name) => name.to_string(),
        None => current_user.name.clone(),
    };

    let next_phone = match req.phone.as_deref().map(str::trim) {
        Some("") => None,
        Some(value) => Some(value.to_string()),
        None => normalize_optional_field(&current_user.phone),
    };

    let next_id_number = match req.id_number.as_deref().map(str::trim) {
        Some("") => None,
        Some(value) => {
            if !validate_id_number(value) {
                return Err(AppError::InvalidIdNumber);
            }
            Some(value.to_string())
        }
        None => normalize_optional_field(&current_user.id_number),
    };

    if next_phone.is_none() && next_id_number.is_none() {
        return Err(AppError::BadRequest(
            "手机号和身份证号至少填写一项".to_string(),
        ));
    }

    let next_avatar_url = match req.avatar_url.as_deref().map(str::trim) {
        Some("") => None,
        Some(url) => Some(url.to_string()),
        None => current_user.avatar_url.clone(),
    };

    let db_phone = to_db_identifier("phone", next_phone.as_deref(), &user_id);
    let db_id_number = to_db_identifier("id_number", next_id_number.as_deref(), &user_id);
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE users
        SET name = ?, id_number = ?, phone = ?, avatar_url = ?, updated_at = ?
        WHERE user_id = ?
        "#,
    )
    .bind(&next_name)
    .bind(&db_id_number)
    .bind(&db_phone)
    .bind(&next_avatar_url)
    .bind(&now)
    .bind(&user_id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        let err_msg = e.to_string();
        if err_msg.contains("users.id_number") {
            AppError::IdNumberConflict
        } else if err_msg.contains("users.phone") {
            AppError::PhoneConflict
        } else {
            AppError::DatabaseError(err_msg)
        }
    })?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "用户信息更新成功",
            "name": next_name,
            "phone": next_phone,
            "id_number": next_id_number,
            "avatar_url": next_avatar_url,
            "created_at": current_user.created_at,
            "updated_at": now,
        }),
    ))
}

/// Delete user account (requires user token)
#[tracing::instrument(skip(state))]
pub async fn delete_user(State(state): State<AppState>, headers: HeaderMap) -> AppResult<Response> {
    let token = require_user_bearer_token(&headers)?;
    let user_id = token_user_id(&token)?.to_string();
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;

    // Delete all bots owned by this user
    sqlx::query("DELETE FROM bots WHERE owner_id = ?")
        .bind(&user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Delete the user
    sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(&user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("User account deleted: {}", user_id);

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "用户账户已删除",
        }),
    ))
}
