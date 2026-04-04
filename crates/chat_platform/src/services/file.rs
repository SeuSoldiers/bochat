use crate::error::{AppError, AppResult};
use crate::models::File;
use crate::services::file_reference;
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

    pub async fn on_message_saved(
        pool: &SqlitePool,
        msg_id: i64,
        msg_type: &str,
        content: &serde_json::Value,
    ) -> AppResult<()> {
        let Some(file_id) = file_reference::extract_file_id_from_message_content(msg_type, content)
        else {
            return Ok(());
        };

        if !file_reference::file_exists(pool, &file_id).await? {
            tracing::warn!(
                "文件消息引用的文件不存在，跳过引用计数: msg_id={}, file_id={}",
                msg_id,
                file_id
            );
            return Ok(());
        }

        let _ = file_reference::add_reference(
            pool,
            &file_id,
            file_reference::REF_TYPE_MESSAGE,
            &msg_id.to_string(),
        )
        .await?;
        Ok(())
    }

    pub async fn on_group_deleted(pool: &SqlitePool, group_id: &str) -> AppResult<()> {
        let affected_file_ids =
            file_reference::remove_message_references_for_group(pool, group_id).await?;
        for file_id in affected_file_ids {
            let _ = file_reference::cleanup_file_if_unreferenced(pool, &file_id).await?;
        }
        Ok(())
    }

    pub async fn on_bot_avatar_changed(
        pool: &SqlitePool,
        bot_id: &str,
        old_avatar_url: Option<&str>,
        new_avatar_url: Option<&str>,
    ) -> AppResult<()> {
        let old_file_id =
            old_avatar_url.and_then(file_reference::extract_file_id_from_download_url);
        let new_file_id =
            new_avatar_url.and_then(file_reference::extract_file_id_from_download_url);

        if old_file_id == new_file_id {
            return Ok(());
        }

        if let Some(old_file_id) = old_file_id {
            let _ = file_reference::remove_reference(
                pool,
                &old_file_id,
                file_reference::REF_TYPE_BOT_AVATAR,
                bot_id,
            )
            .await?;
            let _ = file_reference::cleanup_file_if_unreferenced(pool, &old_file_id).await?;
        }

        if let Some(new_file_id) = new_file_id {
            if file_reference::file_exists(pool, &new_file_id).await? {
                let _ = file_reference::add_reference(
                    pool,
                    &new_file_id,
                    file_reference::REF_TYPE_BOT_AVATAR,
                    bot_id,
                )
                .await?;
            } else {
                tracing::warn!(
                    "Bot 头像引用的文件不存在，跳过引用计数: bot_id={}, file_id={}",
                    bot_id,
                    new_file_id
                );
            }
        }

        Ok(())
    }

    pub async fn remove_uploader_and_cleanup(
        pool: &SqlitePool,
        file_id: &str,
        uploader_bot_id: &str,
    ) -> AppResult<(bool, bool)> {
        let relation_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM file_uploaders WHERE file_id = ? AND uploader_id = ?)",
        )
        .bind(file_id)
        .bind(uploader_bot_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if !relation_exists {
            return Ok((false, false));
        }

        sqlx::query("DELETE FROM file_uploaders WHERE file_id = ? AND uploader_id = ?")
            .bind(file_id)
            .bind(uploader_bot_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let physical_deleted = file_reference::cleanup_file_if_unreferenced(pool, file_id).await?;
        Ok((true, physical_deleted))
    }
}
