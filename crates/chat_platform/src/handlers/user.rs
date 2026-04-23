use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::Response,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    error::{json_response, AppError, AppResult},
    middlewares::UserAuth,
    models::{UpdateUserRequest, UserResponse},
    repositories::UserRepository,
    AppState,
};

const PASSWORD_MIN_LEN: usize = 8;
const PASSWORD_MAX_LEN: usize = 64;

fn validate_password(password: &str) -> bool {
    let len = password.len();
    if !(PASSWORD_MIN_LEN..=PASSWORD_MAX_LEN).contains(&len) {
        return false;
    }

    let has_letter = password.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let all_printable_ascii = password.chars().all(|c| c.is_ascii_graphic());

    has_letter && has_digit && all_printable_ascii
}

fn hash_password(password: &str, pepper: &str) -> String {
    let salt = Uuid::new_v4().simple().to_string();
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hasher.update(pepper.as_bytes());
    let digest = hasher.finalize();
    format!("{}${}", salt, hex::encode(digest))
}

#[tracing::instrument(skip_all)]
pub async fn get_current_user(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
) -> AppResult<Response> {
    let user_id = auth.user_id;

    let user: crate::models::User = UserRepository::find_by_id(&state.pool, &user_id)
        .await?
        .ok_or(AppError::InvalidUserToken)?;

    Ok(json_response(
        StatusCode::OK,
        json!(UserResponse::from(user)),
    ))
}

#[tracing::instrument(skip_all)]
pub async fn update_current_user(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Response> {
    let user_id = auth.user_id;

    let current_user: crate::models::User = UserRepository::find_by_id(&state.pool, &user_id)
        .await?
        .ok_or(AppError::InvalidUserToken)?;

    let next_name = match req.name.as_deref().map(str::trim) {
        Some("") => {
            return Err(AppError::BadRequest("昵称不能为空".to_string()));
        }
        Some(name) => name.to_string(),
        None => current_user.name.clone(),
    };

    let next_avatar_url = match req.avatar_url.as_deref().map(str::trim) {
        Some("") => None,
        Some(url) => Some(url.to_string()),
        None => current_user.avatar_url.clone(),
    };

    let next_password_hash = match req.password.as_deref().map(str::trim) {
        Some("") => None,
        Some(password) => {
            if !validate_password(password) {
                return Err(AppError::BadRequest(format!(
                    "密码格式不正确：长度需为{}-{}位，且必须包含字母和数字，仅支持可见 ASCII 字符",
                    PASSWORD_MIN_LEN, PASSWORD_MAX_LEN
                )));
            }
            Some(hash_password(password, &state.config.security.jwt_secret))
        }
        None => None,
    };

    let now = chrono::Utc::now().to_rfc3339();

    UserRepository::update_profile(
        &state.pool,
        &user_id,
        &next_name,
        next_avatar_url.as_deref(),
        next_password_hash.as_deref(),
        &now,
    )
    .await?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "用户信息更新成功",
            "name": next_name,
            "avatar_url": next_avatar_url,
            "created_at": current_user.created_at,
            "updated_at": now,
        }),
    ))
}

/// Delete user account (requires user token)
#[tracing::instrument(skip_all)]
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
) -> AppResult<Response> {
    let user_id = auth.user_id;

    // Delete all bots owned by this user
    UserRepository::delete_bots_by_owner(&state.pool, &user_id).await?;

    // Delete the user
    UserRepository::delete_user_by_id(&state.pool, &user_id).await?;

    tracing::info!("User account deleted: {}", user_id);

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "用户账户已删除",
        }),
    ))
}
