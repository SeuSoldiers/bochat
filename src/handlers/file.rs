use actix_multipart::Multipart;
use actix_web::{http::header, web, HttpRequest, HttpResponse};
use futures_util::TryStreamExt as _;
use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::utils::verify_token;

#[allow(dead_code)]
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

#[tracing::instrument(skip(pool, config, http_req, payload))]
pub async fn upload_file(
    pool: web::Data<DbPool>,
    config: web::Data<crate::config::Config>,
    http_req: HttpRequest,
    mut payload: Multipart,
) -> AppResult<HttpResponse> {
    // Extract and verify token
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }
    let bot_id = parts[0];

    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
        .bind(bot_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or(AppError::Unauthorized)?;

    let _token_payload = verify_token(token, &bot.secret, config.security.token_expiry_secs)?;

    let mut uploaded_filename: Option<String> = None;
    let mut uploaded_mime: Option<String> = None;
    let file_id = crate::utils::generate_file_id();
    let now = chrono::Utc::now().to_rfc3339();
    let upload_dir = std::path::PathBuf::from(&config.storage.file_storage_path);
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let storage_path = upload_dir.join(&file_id);
    let mut file = tokio::fs::File::create(&storage_path)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let mut total_size: u64 = 0;

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let content_disposition = field.content_disposition();

        let filename = content_disposition
            .get_filename()
            .map(|name| sanitize_filename(name))
            .unwrap_or_else(|| "upload.bin".to_string());
        uploaded_filename = Some(filename);
        uploaded_mime = Some(
            field
                .content_type()
                .map(|mime| mime.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string()),
        );

        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?
        {
            total_size += chunk.len() as u64;
            if total_size > config.security.max_file_size_mb * 1024 * 1024 {
                return Err(AppError::FileTooLarge);
            }

            file.write_all(&chunk)
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
        }
        break;
    }

    let filename = uploaded_filename.ok_or_else(|| AppError::BadRequest("未上传文件".to_string()))?;
    let mime_type = uploaded_mime.unwrap_or_else(|| "application/octet-stream".to_string());

    sqlx::query(
        r#"
        INSERT INTO files (file_id, owner_id, filename, size, mime_type, storage_path, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&file_id)
    .bind(&bot.bot_id)
    .bind(&filename)
    .bind(total_size as i64)
    .bind(&mime_type)
    .bind(storage_path.to_string_lossy().to_string())
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let connection = http_req.connection_info();
    let file_url = format!(
        "{}://{}/api/v1/file/download/{}",
        connection.scheme(),
        connection.host(),
        file_id
    );

    Ok(HttpResponse::Created().json(json!({
        "file_id": file_id,
        "url": file_url,
        "created_at": now,
    })))
}

#[tracing::instrument(skip(pool, _config, _http_req))]
pub async fn download_file(
    pool: web::Data<DbPool>,
    _config: web::Data<crate::config::Config>,
    _http_req: HttpRequest,
    file_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let file: crate::models::File = sqlx::query_as(
        "SELECT file_id, owner_id, filename, size, mime_type, storage_path, created_at FROM files WHERE file_id = ?"
    )
    .bind(file_id.as_str())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::FileNotFound)?;

    let bytes = tokio::fs::read(&file.storage_path)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, file.mime_type))
        .insert_header((
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", file.filename),
        ))
        .body(bytes))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
