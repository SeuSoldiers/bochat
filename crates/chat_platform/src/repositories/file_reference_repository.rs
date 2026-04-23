use crate::error::{AppError, AppResult};
use sqlx::SqlitePool;

pub struct FileReferenceKey<'a> {
    pub file_id: &'a str,
    pub reference_type: &'a str,
    pub reference_id: &'a str,
}

pub struct NewFileReference<'a> {
    pub file_id: &'a str,
    pub reference_type: &'a str,
    pub reference_id: &'a str,
    pub created_at: &'a str,
}

pub struct MessageReferenceCleanup<'a> {
    pub reference_type: &'a str,
    pub group_id: &'a str,
}

pub struct FileReferenceRepository;

impl FileReferenceRepository {
    pub async fn add_reference_ignore(pool: &SqlitePool, reference: &NewFileReference<'_>) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO file_references (file_id, reference_type, reference_id, created_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(reference.file_id)
        .bind(reference.reference_type)
        .bind(reference.reference_id)
        .bind(reference.created_at)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    pub async fn remove_reference(pool: &SqlitePool, key: &FileReferenceKey<'_>) -> AppResult<u64> {
        let result = sqlx::query(
            "DELETE FROM file_references WHERE file_id = ? AND reference_type = ? AND reference_id = ?",
        )
        .bind(key.file_id)
        .bind(key.reference_type)
        .bind(key.reference_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    pub async fn file_exists(pool: &SqlitePool, file_id: &str) -> AppResult<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM files WHERE file_id = ?)")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_affected_file_ids_by_group_messages(
        pool: &SqlitePool,
        cleanup: &MessageReferenceCleanup<'_>,
    ) -> AppResult<Vec<String>> {
        sqlx::query_scalar(
            r#"
            SELECT DISTINCT fr.file_id
            FROM file_references fr
            INNER JOIN messages m
                ON fr.reference_type = ? AND fr.reference_id = CAST(m.msg_id AS TEXT)
            WHERE m.group_id = ?
            "#,
        )
        .bind(cleanup.reference_type)
        .bind(cleanup.group_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn remove_message_references_by_group(
        pool: &SqlitePool,
        cleanup: &MessageReferenceCleanup<'_>,
    ) -> AppResult<()> {
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
        .bind(cleanup.reference_type)
        .bind(cleanup.group_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn find_storage_path(pool: &SqlitePool, file_id: &str) -> AppResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT storage_path FROM files WHERE file_id = ?")
            .bind(file_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.map(|(storage_path,)| storage_path))
    }

    pub async fn count_references(pool: &SqlitePool, file_id: &str) -> AppResult<i64> {
        sqlx::query_scalar("SELECT COUNT(1) FROM file_references WHERE file_id = ?")
            .bind(file_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn delete_uploaders_by_file_id(pool: &SqlitePool, file_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM file_uploaders WHERE file_id = ?")
            .bind(file_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_references_by_file_id(pool: &SqlitePool, file_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM file_references WHERE file_id = ?")
            .bind(file_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_file_by_id(pool: &SqlitePool, file_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM files WHERE file_id = ?")
            .bind(file_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
