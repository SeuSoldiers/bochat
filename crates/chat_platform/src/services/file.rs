use crate::error::AppResult;
use crate::models::File;
use sqlx::SqlitePool;

pub struct FileService;

impl FileService {
    pub async fn get_file_by_id(pool: &SqlitePool, file_id: &str) -> AppResult<File> {
        sqlx::query_as::<_, File>(
            "SELECT file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at FROM files WHERE file_id = ?"
        )
        .bind(file_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?
        .ok_or(crate::error::AppError::FileNotFound)
    }

    pub async fn get_user_files(
        pool: &SqlitePool,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<File>> {
        sqlx::query_as::<_, File>(
            "SELECT f.file_id, f.owner_id, f.content_hash, f.filename, f.size, f.mime_type, f.storage_path, f.created_at\n             FROM files f\n             INNER JOIN file_uploaders fu ON fu.file_id = f.file_id\n             INNER JOIN bots b ON b.bot_id = fu.uploader_id\n             WHERE b.owner_id = ?\n             GROUP BY f.file_id\n             ORDER BY f.created_at DESC\n             LIMIT ? OFFSET ?"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    }
}
