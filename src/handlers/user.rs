use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::utils::verify_token;

/// Delete user account (requires verification via any bot token)
#[tracing::instrument(skip(pool))]
pub async fn delete_user(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    // Extract token from Authorization header
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    // Parse token to get bot_id
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }
    let bot_id = parts[0];

    // Get user's bot
    let user_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Verify token
    let _token_payload = verify_token(token, &user_bot.secret, 86400)?;

    let user_id = user_bot.owner_id.clone();

    // Delete all bots owned by this user
    sqlx::query("DELETE FROM bots WHERE owner_id = ?")
        .bind(&user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Delete the user
    sqlx::query("DELETE FROM users WHERE user_id = ?")
        .bind(&user_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("User account deleted: {}", user_id);

    Ok(HttpResponse::Ok().json(json!({
        "message": "用户账户已删除",
        "user_id": user_id,
    })))
}
