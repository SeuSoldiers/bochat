use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, Response, StatusCode},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::utils::verify_token;
use crate::{
    error::{json_response, AppError, AppResult},
    http::{require_bot_bearer_token, token_bot_id},
    AppState,
};

#[allow(dead_code)]
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

#[tracing::instrument(skip_all)]
pub async fn upload_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut payload: Multipart,
) -> AppResult<axum::response::Response> {
    // Extract and verify token
    let token = require_bot_bearer_token(&headers)?;
    let bot_id = token_bot_id(&token)
        .map_err(|_| AppError::InvalidBotToken)?
        .to_string();
    let (bot_secret, bot_status): (String, String) =
        sqlx::query_as("SELECT secret, status FROM bots WHERE bot_id = ?")
            .bind(&bot_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or(AppError::InvalidBotToken)?;

    if bot_status != "active" {
        return Err(AppError::BotInactive);
    }

    let _token_payload =
        verify_token(&token, &bot_secret, state.config.security.token_expiry_secs)?;

    let mut uploaded_filename: Option<String> = None;
    let mut uploaded_mime: Option<String> = None;
    let now = chrono::Utc::now().to_rfc3339();
    let upload_dir = std::path::PathBuf::from(&state.config.storage.file_storage_path);
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let mut total_size: u64 = 0;
    let mut file_bytes = Vec::new();

    if let Some(field) = payload
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let filename = field
            .file_name()
            .map(sanitize_filename)
            .unwrap_or_else(|| "upload.bin".to_string());
        uploaded_filename = Some(filename);
        uploaded_mime = Some(
            field
                .content_type()
                .map(|mime| mime.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string()),
        );

        let bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        total_size = bytes.len() as u64;
        if total_size > state.config.security.max_file_size_mb * 1024 * 1024 {
            tracing::warn!(
                "文件上传被拒绝: 超出大小限制, bot_id={}, size={}",
                bot_id,
                total_size
            );
            return Err(AppError::FileTooLarge);
        }

        file_bytes.extend_from_slice(&bytes);
    }

    let filename =
        uploaded_filename.ok_or_else(|| AppError::BadRequest("未上传文件".to_string()))?;
    let mime_type = uploaded_mime.unwrap_or_else(|| "application/octet-stream".to_string());
    let content_hash = hex::encode(Sha256::digest(&file_bytes));

    tracing::trace!(
        "开始处理文件上传: bot_id={}, filename={}, size={}, mime_type={}, sha256={}",
        bot_id,
        filename,
        total_size,
        mime_type,
        content_hash
    );

    let existing_file: Option<crate::models::File> = sqlx::query_as(
        "SELECT file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at FROM files WHERE content_hash = ?"
    )
    .bind(&content_hash)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http")
        .to_string();
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1:8080")
        .to_string();

    if let Some(existing_file) = existing_file {
        sqlx::query(
            "INSERT OR IGNORE INTO file_uploaders (file_id, uploader_id, created_at) VALUES (?, ?, ?)",
        )
        .bind(&existing_file.file_id)
        .bind(&bot_id)
        .bind(&now)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let file_url = format!(
            "{}://{}/api/v1/file/download/{}/{}",
            scheme, host, existing_file.file_id, existing_file.filename
        );

        tracing::trace!(
            "文件复用命中: bot_id={}, existing_file_id={}, sha256={}",
            bot_id,
            existing_file.file_id,
            content_hash
        );

        return Ok(json_response(
            StatusCode::CREATED,
            json!({
                "file_id": existing_file.file_id,
                "filename": existing_file.filename,
                "url": file_url,
                "created_at": existing_file.created_at,
            }),
        ));
    }

    let file_id = crate::utils::generate_file_id();
    let file_dir = upload_dir.join(&file_id);
    tokio::fs::create_dir_all(&file_dir)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    let storage_path = file_dir.join(&filename);
    let mut file = tokio::fs::File::create(&storage_path)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    file.write_all(&file_bytes)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO files (file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&file_id)
    .bind(&bot_id)
    .bind(&content_hash)
    .bind(&filename)
    .bind(total_size as i64)
    .bind(&mime_type)
    .bind(storage_path.to_string_lossy().to_string())
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    sqlx::query(
        "INSERT OR IGNORE INTO file_uploaders (file_id, uploader_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(&file_id)
    .bind(&bot_id)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let file_url = format!(
        "{}://{}/api/v1/file/download/{}/{}",
        scheme, host, file_id, filename
    );

    tracing::trace!(
        "文件上传成功: bot_id={}, file_id={}, sha256={}, path={}",
        bot_id,
        file_id,
        content_hash,
        storage_path.to_string_lossy()
    );

    Ok(json_response(
        StatusCode::CREATED,
        json!({
            "file_id": file_id,
            "filename": filename,
            "url": file_url,
            "created_at": now,
        }),
    ))
}

#[tracing::instrument(skip_all)]
pub async fn download_file(
    State(state): State<AppState>,
    Path((file_id, requested_filename)): Path<(String, String)>,
) -> AppResult<Response<Body>> {
    let file: crate::models::File = sqlx::query_as(
        "SELECT file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at FROM files WHERE file_id = ?"
    )
    .bind(file_id.as_str())
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::FileNotFound)?;

    let bytes = tokio::fs::read(&file.storage_path)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if requested_filename != file.filename {
        return Err(AppError::BadRequest("文件名不匹配".to_string()));
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, file.mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", file.filename),
        )
        .body(Body::from(bytes))
        .map_err(|e| AppError::InternalError(e.to_string()))
}

#[tracing::instrument(skip_all)]
pub async fn delete_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_id): Path<String>,
) -> AppResult<axum::response::Response> {
    let token = require_bot_bearer_token(&headers)?;
    let bot_id = token_bot_id(&token)
        .map_err(|_| AppError::InvalidBotToken)?
        .to_string();

    let (bot_secret, bot_status): (String, String) =
        sqlx::query_as("SELECT secret, status FROM bots WHERE bot_id = ?")
            .bind(&bot_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or(AppError::InvalidBotToken)?;

    if bot_status != "active" {
        return Err(AppError::BotInactive);
    }

    let _token_payload =
        verify_token(&token, &bot_secret, state.config.security.token_expiry_secs)?;

    let file: (String, String) =
        sqlx::query_as("SELECT file_id, storage_path FROM files WHERE file_id = ?")
            .bind(&file_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or(AppError::FileNotFound)?;

    let relation_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM file_uploaders WHERE file_id = ? AND uploader_id = ?)",
    )
    .bind(&file_id)
    .bind(&bot_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !relation_exists {
        return Err(AppError::Forbidden("只能删除自己上传过的文件".to_string()));
    }

    sqlx::query("DELETE FROM file_uploaders WHERE file_id = ? AND uploader_id = ?")
        .bind(&file_id)
        .bind(&bot_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let uploader_count: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM file_uploaders WHERE file_id = ?")
            .bind(&file_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut physical_deleted = false;
    if uploader_count == 0 {
        sqlx::query("DELETE FROM files WHERE file_id = ?")
            .bind(&file_id)
            .execute(&state.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Err(err) = tokio::fs::remove_file(&file.1).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                return Err(AppError::InternalError(err.to_string()));
            }
        }

        if let Some(parent_dir) = std::path::Path::new(&file.1).parent() {
            if let Err(err) = tokio::fs::remove_dir(parent_dir).await {
                if err.kind() != std::io::ErrorKind::NotFound
                    && err.kind() != std::io::ErrorKind::DirectoryNotEmpty
                {
                    return Err(AppError::InternalError(err.to_string()));
                }
            }
        }

        physical_deleted = true;
    }

    Ok(json_response(
        StatusCode::OK,
        json!({
            "file_id": file.0,
            "uploader_removed": true,
            "physical_deleted": physical_deleted,
        }),
    ))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
