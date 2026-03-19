use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateBotRequest, BotResponse};
use crate::utils::{generate_token, generate_bot_id, verify_token};

/// Create a new bot for the authenticated user
#[tracing::instrument(skip(pool))]
pub async fn create_bot(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    req: web::Json<CreateBotRequest>,
) -> AppResult<HttpResponse> {
    // Extract bot token from Authorization header
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

    // Get the bot to retrieve its secret
    let owner_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Now verify the token using the bot's secret
    let _token_payload = verify_token(token, &owner_bot.secret, 86400)?;

    let owner_id = owner_bot.owner_id;

    // Validate input
    if req.name.is_empty() {
        return Err(AppError::BadRequest("Bot name is required".to_string()));
    }

    // Create new bot
    let bot_id = generate_bot_id();
    let bot_secret = Uuid::new_v4().to_string();
    let bot_token = generate_token(&bot_id, &bot_secret)?;
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO bots (bot_id, owner_id, name, description, status, token, secret, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&bot_id)
    .bind(&owner_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind("active")
    .bind(&bot_token)
    .bind(&bot_secret)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Bot created: {} for user {}", bot_id, owner_id);

    Ok(HttpResponse::Created().json(json!({
        "bot_id": bot_id,
        "owner_id": owner_id,
        "name": req.name,
        "description": req.description,
        "status": "active",
        "token": bot_token,
        "created_at": now,
        "updated_at": now,
    })))
}

/// List all bots for the authenticated user
#[tracing::instrument(skip(pool))]
pub async fn list_bots(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    // Extract bot token from Authorization header
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

    // Get the bot to find the owner and verify token
    let owner_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Verify the token using the bot's secret
    let _token_payload = verify_token(token, &owner_bot.secret, 86400)?;

    // Get all bots for this user
    let bots: Vec<crate::models::Bot> = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE owner_id = ? ORDER BY created_at DESC"
    )
    .bind(&owner_bot.owner_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let responses: Vec<BotResponse> = bots.into_iter().map(|b| b.into()).collect();

    Ok(HttpResponse::Ok().json(json!({
        "bots": responses,
    })))
}

/// Get a specific bot by ID
#[tracing::instrument(skip(pool))]
pub async fn get_bot(
    pool: web::Data<DbPool>,
    bot_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(bot_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    let response: BotResponse = bot.into();
    Ok(HttpResponse::Ok().json(response))
}

/// Delete a bot (only owner can delete)
#[tracing::instrument(skip(pool))]
pub async fn delete_bot(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    bot_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let bot_id_to_delete = bot_id.into_inner();

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
    let requester_bot_id = parts[0];

    // Get requester bot
    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Verify token
    let _token_payload = verify_token(token, &requester_bot.secret, 86400)?;

    // Get bot to delete
    let bot_to_delete: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&bot_id_to_delete)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Check if requester owns the bot
    if requester_bot.owner_id != bot_to_delete.owner_id {
        return Err(AppError::BotPermissionDenied);
    }

    // Delete the bot (messages will be preserved as they don't have foreign key constraints)
    sqlx::query("DELETE FROM bots WHERE bot_id = ?")
        .bind(&bot_id_to_delete)
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Bot deleted: {} by user {}", bot_id_to_delete, requester_bot.owner_id);

    Ok(HttpResponse::Ok().json(json!({
        "message": "Bot deleted successfully",
        "bot_id": bot_id_to_delete,
    })))
}

