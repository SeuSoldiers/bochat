use sqlx::SqlitePool;
use crate::error::AppResult;
use crate::models::File;

pub struct FileService;

impl FileService {
    pub async fn get_file_by_id(pool: &SqlitePool, file_id: &str) -> AppResult<File> {
        sqlx::query_as::<_, File>(
            "SELECT file_id, owner_id, filename, size, mime_type, storage_path, created_at FROM files WHERE file_id = ?"
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
            "SELECT file_id, owner_id, filename, size, mime_type, storage_path, created_at FROM files WHERE owner_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))
    }
}
