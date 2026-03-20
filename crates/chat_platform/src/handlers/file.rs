use axum::{
    body::Body,
    extract::{Host, Multipart, Path, State},
    http::{header, HeaderMap, Response, StatusCode},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::utils::verify_user_token;
use crate::{
    error::{json_response, AppError, AppResult},
    http::{require_user_bearer_token, token_user_id},
    AppState,
};

#[allow(dead_code)]
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

#[tracing::instrument(skip(state, payload))]
pub async fn upload_file(
    State(state): State<AppState>,
    Host(host): Host,
    headers: HeaderMap,
    mut payload: Multipart,
) -> AppResult<axum::response::Response> {
    // Extract and verify token
    let token = require_user_bearer_token(&headers)?;
    let user_id = token_user_id(&token)?.to_string();
    let _token_payload = verify_user_token(
        &token,
        &state.config.security.jwt_secret,
        state.config.security.token_expiry_secs,
    )?;

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
                user_id,
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

    tracing::info!(
        "开始处理文件上传: bot_id={}, filename={}, size={}, mime_type={}, sha256={}",
        user_id,
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

    if let Some(existing_file) = existing_file {
        let file_url = format!(
            "{}://{}/api/v1/file/download/{}",
            scheme, host, existing_file.file_id
        );

        tracing::info!(
            "文件复用命中: bot_id={}, existing_file_id={}, sha256={}",
            user_id,
            existing_file.file_id,
            content_hash
        );

        return Ok(json_response(
            StatusCode::CREATED,
            json!({
                "file_id": existing_file.file_id,
                "url": file_url,
                "created_at": existing_file.created_at,
            }),
        ));
    }

    let file_id = crate::utils::generate_file_id();
    let storage_path = upload_dir.join(&file_id);
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
    .bind(&user_id)
    .bind(&content_hash)
    .bind(&filename)
    .bind(total_size as i64)
    .bind(&mime_type)
    .bind(storage_path.to_string_lossy().to_string())
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let file_url = format!("{}://{}/api/v1/file/download/{}", scheme, host, file_id);

    tracing::info!(
        "文件上传成功: bot_id={}, file_id={}, sha256={}, path={}",
        user_id,
        file_id,
        content_hash,
        storage_path.to_string_lossy()
    );

    Ok(json_response(
        StatusCode::CREATED,
        json!({
            "file_id": file_id,
            "url": file_url,
            "created_at": now,
        }),
    ))
}

#[tracing::instrument(skip(state))]
pub async fn download_file(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
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
