use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateBotRequest, BotResponse};
use crate::utils::{generate_token, generate_bot_id, verify_token};

/// Create a new bot for the authenticated user
#[tracing::instrument(skip(pool, config))]
pub async fn create_bot(
    pool: web::Data<DbPool>,
    config: web::Data<crate::config::Config>,
    http_req: HttpRequest,
    req: web::Json<CreateBotRequest>,
) -> AppResult<HttpResponse> {
    // Extract and verify bot token from Authorization header
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    // Verify token to get the user's bot (through owner)
    let token_payload = verify_token(token, &config.security.jwt_secret, config.security.token_expiry_secs)?;

    // Get the bot's owner (user)
    let owner_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&token_payload.bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

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
#[tracing::instrument(skip(pool, config))]
pub async fn list_bots(
    pool: web::Data<DbPool>,
    config: web::Data<crate::config::Config>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    // Extract and verify bot token from Authorization header
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    // Verify token
    let token_payload = verify_token(token, &config.security.jwt_secret, config.security.token_expiry_secs)?;

    // Get the bot to find the owner
    let owner_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&token_payload.bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

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
