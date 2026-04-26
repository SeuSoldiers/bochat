use axum::{
    body::Body,
    extract::{Extension, Multipart, Path, State},
    http::{header, Response, StatusCode},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::{
    error::{json_response, AppError, AppResult},
    middlewares::BotAuth,
    repositories::{FileRepository, NewFile, NewFileUploader},
    services::FileManager,
    AppState,
};

#[allow(dead_code)]
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

#[tracing::instrument(skip_all)]
pub async fn upload_file(
    State(state): State<AppState>,
    Extension(auth): Extension<BotAuth>,
    headers: axum::http::HeaderMap,
    mut payload: Multipart,
) -> AppResult<axum::response::Response> {
    let bot_id = auth.bot_id;

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

    let existing_file: Option<crate::models::File> =
        FileRepository::find_by_content_hash(&state.pool, &content_hash).await?;

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
        FileRepository::add_uploader_ignore(
            &state.pool,
            &NewFileUploader {
                file_id: &existing_file.file_id,
                uploader_id: &bot_id,
                created_at: &now,
            },
        )
        .await?;

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

    FileRepository::insert_file(
        &state.pool,
        &NewFile {
            file_id: &file_id,
            owner_id: &bot_id,
            content_hash: &content_hash,
            filename: &filename,
            size: total_size as i64,
            mime_type: &mime_type,
            storage_path: &storage_path.to_string_lossy(),
            created_at: &now,
        },
    )
    .await?;

    FileRepository::add_uploader_ignore(
        &state.pool,
        &NewFileUploader {
            file_id: &file_id,
            uploader_id: &bot_id,
            created_at: &now,
        },
    )
    .await?;

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
    let file: crate::models::File = FileRepository::find_by_id(&state.pool, file_id.as_str())
        .await?
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
    Extension(auth): Extension<BotAuth>,
    Path(file_id): Path<String>,
) -> AppResult<axum::response::Response> {
    let bot_id = auth.bot_id;

    let file_id_for_response: String = FileRepository::exists_file_id(&state.pool, &file_id)
        .await?
        .ok_or(AppError::FileNotFound)?;

    let (uploader_removed, physical_deleted) =
        FileManager::remove_uploader_and_cleanup(&state.pool, &file_id, &bot_id).await?;
    if !uploader_removed {
        return Err(AppError::Forbidden("只能删除自己上传过的文件".to_string()));
    }

    Ok(json_response(
        StatusCode::OK,
        json!({
            "file_id": file_id_for_response,
            "uploader_removed": uploader_removed,
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
