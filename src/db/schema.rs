use sqlx::sqlite::SqlitePool;
use crate::error::AppResult;

pub async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            user_id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create bots table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS bots (
            bot_id TEXT PRIMARY KEY,
            bot_type TEXT NOT NULL,
            owner_id TEXT NOT NULL,
            name TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (owner_id) REFERENCES users(user_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create messages table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            msg_id INTEGER PRIMARY KEY AUTOINCREMENT,
            sender_id TEXT NOT NULL,
            to_id TEXT NOT NULL,
            content TEXT NOT NULL,
            msg_type TEXT DEFAULT 'text',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (sender_id) REFERENCES bots(bot_id),
            FOREIGN KEY (to_id) REFERENCES bots(bot_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create index on messages table for faster queries
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_to_id_created_at
        ON messages(to_id, created_at DESC)
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create files table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS files (
            file_id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            filename TEXT NOT NULL,
            size INTEGER NOT NULL,
            mime_type TEXT NOT NULL,
            storage_path TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (owner_id) REFERENCES users(user_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}
