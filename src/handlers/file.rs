use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::utils::verify_token;

#[allow(dead_code)]
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

#[tracing::instrument(skip(pool, config, http_req, _body))]
pub async fn upload_file(
    pool: web::Data<DbPool>,
    config: web::Data<crate::config::Config>,
    http_req: HttpRequest,
    _body: web::Payload,
) -> AppResult<HttpResponse> {
    // Extract and verify token
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let token_payload = verify_token(token, &config.security.jwt_secret, config.security.token_expiry_secs)?;

    // Get user from bot_id
    let _user_id: String = sqlx::query_scalar("SELECT owner_id FROM bots WHERE bot_id = ?")
        .bind(&token_payload.bot_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or(AppError::Unauthorized)?;

    // TODO: In production, implement actual file upload logic
    // For now, return a placeholder response
    let file_id = crate::utils::generate_file_id();
    let now = chrono::Utc::now().to_rfc3339();

    Ok(HttpResponse::Created().json(json!({
        "file_id": file_id,
        "url": format!("/api/v1/file/download/{}", file_id),
        "created_at": now,
    })))
}

#[tracing::instrument(skip(pool, config, http_req))]
pub async fn download_file(
    pool: web::Data<DbPool>,
    config: web::Data<crate::config::Config>,
    http_req: HttpRequest,
    file_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    // Extract and verify token
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let _token_payload = verify_token(token, &config.security.jwt_secret, config.security.token_expiry_secs)?;

    // Verify file exists
    let file_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM files WHERE file_id = ?)")
        .bind(file_id.as_str())
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !file_exists {
        return Err(AppError::FileNotFound);
    }

    // TODO: Implement file download logic
    Ok(HttpResponse::Ok().json(json!({
        "message": "File download not yet implemented"
    })))
}
