use actix_web::{web, HttpResponse};
use serde_json::json;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateUserRequest, LoginRequest};
use crate::utils::{generate_token, generate_user_id};

// Simple password hashing function (in production, use bcrypt or argon2)
fn hash_password(password: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

#[tracing::instrument(skip(pool, config))]
pub async fn register(
    pool: web::Data<DbPool>,
    config: web::Data<crate::config::Config>,
    req: web::Json<CreateUserRequest>,
) -> AppResult<HttpResponse> {
    // Validate input
    if req.username.is_empty() || req.email.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest("Missing required fields".to_string()));
    }

    let user_id = generate_user_id();
    let password_hash = hash_password(&req.password);
    let now = chrono::Utc::now().to_rfc3339();

    // Create user
    sqlx::query(
        r#"
        INSERT INTO users (user_id, username, email, password_hash, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            if e.to_string().contains("username") {
                AppError::UsernameConflict
            } else {
                AppError::EmailConflict
            }
        } else {
            AppError::DatabaseError(e.to_string())
        }
    })?;

    // Create personal bot for the user
    let bot_id = crate::utils::generate_bot_id();
    let bot_token = generate_token(&bot_id, &config.security.jwt_secret)?;

    sqlx::query(
        r#"
        INSERT INTO bots (bot_id, bot_type, owner_id, name, token, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&bot_id)
    .bind("personal")
    .bind(&user_id)
    .bind(format!("{}'s personal bot", req.username))
    .bind(&bot_token)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("User registered: {}", user_id);

    Ok(HttpResponse::Created().json(json!({
        "user_id": user_id,
        "username": req.username,
        "email": req.email,
        "bot_id": bot_id,
        "token": bot_token,
        "created_at": now,
    })))
}

#[tracing::instrument(skip(pool, config))]
pub async fn login(
    pool: web::Data<DbPool>,
    config: web::Data<crate::config::Config>,
    req: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    // Find user by username
    let user: crate::models::User = sqlx::query_as(
        "SELECT user_id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::InvalidCredentials)?;

    // Verify password
    if !verify_password(&req.password, &user.password_hash) {
        return Err(AppError::InvalidCredentials);
    }

    // Get user's personal bot
    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, bot_type, owner_id, name, token, created_at FROM bots WHERE owner_id = ? AND bot_type = 'personal' LIMIT 1"
    )
    .bind(&user.user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Generate new token
    let new_token = generate_token(&bot.bot_id, &config.security.jwt_secret)?;

    // Update bot token in database
    sqlx::query("UPDATE bots SET token = ? WHERE bot_id = ?")
        .bind(&new_token)
        .bind(&bot.bot_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("User logged in: {}", user.user_id);

    Ok(HttpResponse::Ok().json(json!({
        "user_id": user.user_id,
        "username": user.username,
        "email": user.email,
        "bot_id": bot.bot_id,
        "token": new_token,
    })))
}
