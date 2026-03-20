use crate::error::AppResult;
use sqlx::sqlite::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            user_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            id_number TEXT NOT NULL UNIQUE,
            phone TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create index on id_number for faster lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_users_id_number
        ON users(id_number)
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
            owner_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            token TEXT NOT NULL UNIQUE,
            secret TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (owner_id) REFERENCES users(user_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    if let Err(e) = sqlx::query("ALTER TABLE bots ADD COLUMN avatar_url TEXT")
        .execute(pool)
        .await
    {
        if !e.to_string().contains("duplicate column name") {
            return Err(crate::error::AppError::DatabaseError(e.to_string()));
        }
    }

    // Create index on owner_id for faster bot lookups by user
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_bots_owner_id
        ON bots(owner_id)
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create groups table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            group_id TEXT PRIMARY KEY,
            group_code TEXT UNIQUE,
            creator_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (creator_id) REFERENCES users(user_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create index on creator_id for faster group lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_groups_creator_id
        ON groups(creator_id)
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create index on group_code for faster lookups by group code
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_groups_group_code
        ON groups(group_code)
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create group_members table (only bots can be members)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS group_members (
            group_id TEXT NOT NULL,
            member_id TEXT NOT NULL,
            member_type TEXT NOT NULL DEFAULT 'bot',
            joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (group_id, member_id),
            FOREIGN KEY (group_id) REFERENCES groups(group_id),
            FOREIGN KEY (member_id) REFERENCES bots(bot_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create index on member_id for faster member lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_group_members_member_id
        ON group_members(member_id)
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
            group_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            content TEXT NOT NULL,
            msg_type TEXT DEFAULT 'text',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (group_id) REFERENCES groups(group_id),
            FOREIGN KEY (sender_id) REFERENCES bots(bot_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    // Create index on messages table for faster queries
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_group_id_created_at
        ON messages(group_id, created_at DESC)
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
            content_hash TEXT,
            filename TEXT NOT NULL,
            size INTEGER NOT NULL,
            mime_type TEXT NOT NULL,
            storage_path TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (owner_id) REFERENCES bots(bot_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    if let Err(e) = sqlx::query("ALTER TABLE files ADD COLUMN content_hash TEXT")
        .execute(pool)
        .await
    {
        if !e.to_string().contains("duplicate column name") {
            return Err(crate::error::AppError::DatabaseError(e.to_string()));
        }
    }

    sqlx::query(
        r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_files_content_hash
        ON files(content_hash)
        WHERE content_hash IS NOT NULL
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| crate::error::AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}
