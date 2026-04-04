use crate::error::{AppError, AppResult};
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
    let result = sqlx::query(
        r#"
        INSERT OR IGNORE INTO file_references (file_id, reference_type, reference_id, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(file_id)
    .bind(reference_type)
    .bind(reference_id)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(result.rows_affected() > 0)
}

pub async fn remove_reference(
    pool: &SqlitePool,
    file_id: &str,
    reference_type: &str,
    reference_id: &str,
) -> AppResult<bool> {
    let result = sqlx::query(
        "DELETE FROM file_references WHERE file_id = ? AND reference_type = ? AND reference_id = ?",
    )
    .bind(file_id)
    .bind(reference_type)
    .bind(reference_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(result.rows_affected() > 0)
}

pub async fn file_exists(pool: &SqlitePool, file_id: &str) -> AppResult<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM files WHERE file_id = ?)")
        .bind(file_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

pub async fn remove_message_references_for_group(
    pool: &SqlitePool,
    group_id: &str,
) -> AppResult<Vec<String>> {
    let affected_file_ids: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT fr.file_id
        FROM file_references fr
        INNER JOIN messages m
            ON fr.reference_type = ? AND fr.reference_id = CAST(m.msg_id AS TEXT)
        WHERE m.group_id = ?
        "#,
    )
    .bind(REF_TYPE_MESSAGE)
    .bind(group_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    sqlx::query(
        r#"
        DELETE FROM file_references
        WHERE reference_type = ?
          AND reference_id IN (
              SELECT CAST(msg_id AS TEXT)
              FROM messages
              WHERE group_id = ?
          )
        "#,
    )
    .bind(REF_TYPE_MESSAGE)
    .bind(group_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(affected_file_ids)
}

pub async fn cleanup_file_if_unreferenced(pool: &SqlitePool, file_id: &str) -> AppResult<bool> {
    let file: Option<(String,)> =
        sqlx::query_as("SELECT storage_path FROM files WHERE file_id = ?")
            .bind(file_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let Some((storage_path,)) = file else {
        return Ok(false);
    };

    let ref_count: i64 =
        sqlx::query_scalar("SELECT COUNT(1) FROM file_references WHERE file_id = ?")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // 物理文件生命周期只由业务引用决定（消息/头像等）。
    // file_uploaders 仅用于“谁上传过”的关系，不阻止回收。
    if ref_count > 0 {
        return Ok(false);
    }

    sqlx::query("DELETE FROM file_uploaders WHERE file_id = ?")
        .bind(file_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    sqlx::query("DELETE FROM file_references WHERE file_id = ?")
        .bind(file_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    sqlx::query("DELETE FROM files WHERE file_id = ?")
        .bind(file_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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
