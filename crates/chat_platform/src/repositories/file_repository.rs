use crate::error::{AppError, AppResult};
use crate::models::File;
use sqlx::PgPool;

pub struct UserFilesPage<'a> {
    pub user_id: &'a str,
    pub limit: i64,
    pub offset: i64,
}

pub struct UploaderRelation<'a> {
    pub file_id: &'a str,
    pub uploader_id: &'a str,
}

pub struct NewFile<'a> {
    pub file_id: &'a str,
    pub owner_id: &'a str,
    pub content_hash: &'a str,
    pub filename: &'a str,
    pub size: i64,
    pub mime_type: &'a str,
    pub storage_path: &'a str,
    pub created_at: &'a str,
}

pub struct NewFileUploader<'a> {
    pub file_id: &'a str,
    pub uploader_id: &'a str,
    pub created_at: &'a str,
}

pub struct FileRepository;

impl FileRepository {
    pub async fn find_by_content_hash(
        pool: &PgPool,
        content_hash: &str,
    ) -> AppResult<Option<File>> {
        sqlx::query_as(
            "SELECT file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at FROM files WHERE content_hash = $1",
        )
        .bind(content_hash)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn find_by_id(pool: &PgPool, file_id: &str) -> AppResult<Option<File>> {
        sqlx::query_as(
            "SELECT file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at FROM files WHERE file_id = $1",
        )
        .bind(file_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn exists_file_id(pool: &PgPool, file_id: &str) -> AppResult<Option<String>> {
        sqlx::query_scalar("SELECT file_id FROM files WHERE file_id = $1")
            .bind(file_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_by_owner_via_uploaders(
        pool: &PgPool,
        query: &UserFilesPage<'_>,
    ) -> AppResult<Vec<File>> {
        sqlx::query_as::<_, File>(
            "SELECT f.file_id, f.owner_id, f.content_hash, f.filename, f.size, f.mime_type, f.storage_path, f.created_at FROM files f INNER JOIN file_uploaders fu ON fu.file_id = f.file_id INNER JOIN bots b ON b.bot_id = fu.uploader_id WHERE b.owner_id = $1 GROUP BY f.file_id ORDER BY f.created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(query.user_id)
        .bind(query.limit)
        .bind(query.offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn uploader_relation_exists(
        pool: &PgPool,
        relation: &UploaderRelation<'_>,
    ) -> AppResult<bool> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM file_uploaders WHERE file_id = $1 AND uploader_id = $2)",
        )
        .bind(relation.file_id)
        .bind(relation.uploader_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn remove_uploader(pool: &PgPool, relation: &UploaderRelation<'_>) -> AppResult<()> {
        sqlx::query("DELETE FROM file_uploaders WHERE file_id = $1 AND uploader_id = $2")
            .bind(relation.file_id)
            .bind(relation.uploader_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn insert_file(pool: &PgPool, new_file: &NewFile<'_>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO files (file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(new_file.file_id)
        .bind(new_file.owner_id)
        .bind(new_file.content_hash)
        .bind(new_file.filename)
        .bind(new_file.size)
        .bind(new_file.mime_type)
        .bind(new_file.storage_path)
        .bind(new_file.created_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn add_uploader_ignore(
        pool: &PgPool,
        new_uploader: &NewFileUploader<'_>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO file_uploaders (file_id, uploader_id, created_at) VALUES ($1, $2, $3) ON CONFLICT (file_id, uploader_id) DO NOTHING",
        )
        .bind(new_uploader.file_id)
        .bind(new_uploader.uploader_id)
        .bind(new_uploader.created_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
