use actix_web::{web, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::RegisterRequest;
use crate::utils::{generate_bot_id, generate_token, generate_user_id};

// 验证身份证号码格式：只检查位数
fn validate_id_number(id_number: &str) -> bool {
    // 标准身份证号是18位数字
    id_number.len() == 18 && id_number.chars().all(|c| c.is_ascii_digit())
}

#[tracing::instrument(skip(pool))]
pub async fn register(
    pool: web::Data<DbPool>,
    req: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    // Validate input
    if req.name.is_empty() || req.id_number.is_empty() || req.phone.is_empty() {
        return Err(AppError::BadRequest("Missing required fields".to_string()));
    }

    // Validate ID number format (18 digits)
    if !validate_id_number(&req.id_number) {
        return Err(AppError::InvalidIdNumber);
    }

    let user_id = generate_user_id();
    let now = chrono::Utc::now().to_rfc3339();

    // Create user with real-name authentication
    sqlx::query(
        r#"
        INSERT INTO users (user_id, name, id_number, phone, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&user_id)
    .bind(&req.name)
    .bind(&req.id_number)
    .bind(&req.phone)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::IdNumberConflict
        } else {
            AppError::DatabaseError(e.to_string())
        }
    })?;

    // Create a default bot for the user
    let bot_id = generate_bot_id();
    let bot_secret = Uuid::new_v4().to_string();
    let bot_token = generate_token(&bot_id, &bot_secret)?;

    sqlx::query(
        r#"
        INSERT INTO bots (bot_id, owner_id, name, description, status, token, secret, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&bot_id)
    .bind(&user_id)
    .bind(format!("{}'s default bot", req.name))
    .bind(Some("Default bot created upon user registration"))
    .bind("active")
    .bind(&bot_token)
    .bind(&bot_secret)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!(
        "User registered: {} with ID number {}",
        user_id,
        req.id_number
    );

    Ok(HttpResponse::Created().json(json!({
        "user_id": user_id,
        "name": req.name,
        "id_number": req.id_number,
        "phone": req.phone,
        "bot_id": bot_id,
        "bot_token": bot_token,
        "created_at": now,
    })))
}

#[tracing::instrument(skip(pool))]
pub async fn get_user_by_id(
    pool: web::Data<DbPool>,
    user_id: &str,
) -> AppResult<crate::models::User> {
    sqlx::query_as(
        "SELECT user_id, name, id_number, phone, created_at, updated_at FROM users WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::UserNotFound)
}
