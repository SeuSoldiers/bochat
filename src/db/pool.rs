use crate::config::DatabaseConfig;
use crate::error::AppResult;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

pub type DbPool = SqlitePool;

pub async fn init_pool(config: &DatabaseConfig) -> AppResult<DbPool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect(&config.url)
        .await
        .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    Ok(pool)
}
