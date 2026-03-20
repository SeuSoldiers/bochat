use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde_json::json;

use crate::{
    error::{json_response, AppError, AppResult},
    http::{bearer_token, token_bot_id},
    AppState,
};
use crate::utils::verify_token;

/// Delete user account (requires verification via any bot token)
#[tracing::instrument(skip(state))]
pub async fn delete_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let token = bearer_token(&headers)?;
    let bot_id = token_bot_id(&token)?;

    // Get user's bot
    let user_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Verify token
    let _token_payload = verify_token(&token, &user_bot.secret, 86400)?;

    let user_id = user_bot.owner_id.clone();

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

    Ok(json_response(StatusCode::OK, json!({
        "message": "用户账户已删除",
        "user_id": user_id,
    })))
}
