use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::CreateMessageRequest;
use crate::utils::verify_token;

#[tracing::instrument(skip(pool, msg_req))]
pub async fn send_message(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    msg_req: web::Json<CreateMessageRequest>,
) -> AppResult<HttpResponse> {
    // Extract token from Authorization header
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    // Parse token to get bot_id without verification yet
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }
    let bot_id = parts[0];

    // Get sender bot and verify token using its secret
    let sender_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Verify token using the bot's secret
    let _token_payload = verify_token(token, &sender_bot.secret, 86400)?;

    if sender_bot.status != "active" {
        return Err(AppError::BadRequest("Sender bot is not active".to_string()));
    }

    // Validate recipient bot exists
    let recipient_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM bots WHERE bot_id = ?)")
        .bind(&msg_req.to_id)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !recipient_exists {
        return Err(AppError::BadRequest("Recipient bot not found".to_string()));
    }

    let msg_type = msg_req.msg_type.as_deref().unwrap_or("text");
    let content = msg_req.content.to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert message into database
    let msg_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO messages (sender_id, to_id, content, msg_type, created_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING msg_id
        "#,
    )
    .bind(bot_id)
    .bind(&msg_req.to_id)
    .bind(&content)
    .bind(msg_type)
    .bind(&now)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Message sent: {} from {} to {}", msg_id, bot_id, msg_req.to_id);

    Ok(HttpResponse::Created().json(json!({
        "msg_id": msg_id,
        "sender_id": bot_id,
        "to_id": msg_req.to_id,
        "content": msg_req.content,
        "msg_type": msg_type,
        "created_at": now,
    })))
}

