use crate::error::{AppError, AppResult};
use crate::repositories::{
    FileReferenceKey, FileReferenceRepository, MessageReferenceCleanup, NewFileReference,
};
use sqlx::SqlitePool;

pub const REF_TYPE_MESSAGE: &str = "message";
pub const REF_TYPE_BOT_AVATAR: &str = "bot_avatar";
const FILE_DOWNLOAD_PATH_MARKER: &str = "/api/v1/file/download/";

pub fn extract_file_id_from_download_url(url: &str) -> Option<String> {
    let marker_pos = url.find(FILE_DOWNLOAD_PATH_MARKER)?;
    let rest = &url[marker_pos + FILE_DOWNLOAD_PATH_MARKER.len()..];
    let file_id = rest.split('/').next()?.trim();
    if file_id.is_empty() {
        return None;
    }
    Some(file_id.to_string())
}

pub fn extract_file_id_from_message_content(
    msg_type: &str,
    content: &serde_json::Value,
) -> Option<String> {
    if msg_type != "file" {
        return None;
    }

    if let Some(file_id) = content
        .get("file_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return Some(file_id.to_string());
    }

    let url = content.get("url")?.as_str()?;
    extract_file_id_from_download_url(url)
}

pub async fn add_reference(
    pool: &SqlitePool,
    file_id: &str,
    reference_type: &str,
    reference_id: &str,
) -> AppResult<bool> {
    let now = chrono::Utc::now().to_rfc3339();
    let affected = FileReferenceRepository::add_reference_ignore(
        pool,
        &NewFileReference {
            file_id,
            reference_type,
            reference_id,
            created_at: &now,
        },
    )
    .await?;

    Ok(affected > 0)
}

pub async fn remove_reference(
    pool: &SqlitePool,
    file_id: &str,
    reference_type: &str,
    reference_id: &str,
) -> AppResult<bool> {
    let affected = FileReferenceRepository::remove_reference(
        pool,
        &FileReferenceKey {
            file_id,
            reference_type,
            reference_id,
        },
    )
    .await?;
    Ok(affected > 0)
}

pub async fn file_exists(pool: &SqlitePool, file_id: &str) -> AppResult<bool> {
    FileReferenceRepository::file_exists(pool, file_id).await
}

pub async fn remove_message_references_for_group(
    pool: &SqlitePool,
    group_id: &str,
) -> AppResult<Vec<String>> {
    let cleanup = MessageReferenceCleanup {
        reference_type: REF_TYPE_MESSAGE,
        group_id,
    };
    let affected_file_ids: Vec<String> =
        FileReferenceRepository::list_affected_file_ids_by_group_messages(pool, &cleanup).await?;

    FileReferenceRepository::remove_message_references_by_group(pool, &cleanup).await?;

    Ok(affected_file_ids)
}

pub async fn cleanup_file_if_unreferenced(pool: &SqlitePool, file_id: &str) -> AppResult<bool> {
    let Some(storage_path) = FileReferenceRepository::find_storage_path(pool, file_id).await? else {
        return Ok(false);
    };

    let ref_count: i64 = FileReferenceRepository::count_references(pool, file_id).await?;

    // 物理文件生命周期只由业务引用决定（消息/头像等）。
    // file_uploaders 仅用于“谁上传过”的关系，不阻止回收。
    if ref_count > 0 {
        return Ok(false);
    }

    FileReferenceRepository::delete_uploaders_by_file_id(pool, file_id).await?;
    FileReferenceRepository::delete_references_by_file_id(pool, file_id).await?;
    FileReferenceRepository::delete_file_by_id(pool, file_id).await?;

    if let Err(err) = tokio::fs::remove_file(&storage_path).await {
        if err.kind() != std::io::ErrorKind::NotFound {
            return Err(AppError::InternalError(err.to_string()));
        }
    }

    if let Some(parent_dir) = std::path::Path::new(&storage_path).parent() {
        if let Err(err) = tokio::fs::remove_dir(parent_dir).await {
            if err.kind() != std::io::ErrorKind::NotFound
                && err.kind() != std::io::ErrorKind::DirectoryNotEmpty
            {
                return Err(AppError::InternalError(err.to_string()));
            }
        }
    }

    Ok(true)
}
