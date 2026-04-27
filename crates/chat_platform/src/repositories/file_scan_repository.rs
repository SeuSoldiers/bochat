use crate::error::{AppError, AppResult};
use crate::models::FileScanRecord;
use sqlx::PgPool;

pub struct NewPendingFileScan<'a> {
    pub file_id: &'a str,
    pub now: &'a str,
}

pub struct FileScanResultUpdate<'a> {
    pub file_id: &'a str,
    pub status: &'a str,
    pub risk_level: &'a str,
    pub scan_result: Option<&'a str>,
    pub scanned_at: &'a str,
    pub now: &'a str,
}

pub struct FileScanRepository;

impl FileScanRepository {
    pub async fn insert_pending_ignore(pool: &PgPool, input: &NewPendingFileScan<'_>) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            INSERT INTO file_scan_records (file_id, status, risk_level, scan_result, scanned_at, created_at, updated_at)
            VALUES ($1, 'pending', 'unknown', NULL, NULL, $2, $2)
            ON CONFLICT (file_id) DO NOTHING
            "#,
        )
        .bind(input.file_id)
        .bind(input.now)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_by_file_id(pool: &PgPool, file_id: &str) -> AppResult<Option<FileScanRecord>> {
        sqlx::query_as(
            r#"
            SELECT file_id, status, risk_level, scan_result, scanned_at, created_at, updated_at
            FROM file_scan_records
            WHERE file_id = $1
            "#,
        )
        .bind(file_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn update_result(pool: &PgPool, input: &FileScanResultUpdate<'_>) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE file_scan_records
            SET status = $1,
                risk_level = $2,
                scan_result = $3,
                scanned_at = $4,
                updated_at = $5
            WHERE file_id = $6
            "#,
        )
        .bind(input.status)
        .bind(input.risk_level)
        .bind(input.scan_result)
        .bind(input.scanned_at)
        .bind(input.now)
        .bind(input.file_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
