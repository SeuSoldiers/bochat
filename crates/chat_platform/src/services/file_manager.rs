use crate::error::AppResult;
use crate::repositories::{FileRepository, UploaderRelation};
use serde_json::Value;
use sqlx::PgPool;

/// 独立文件管理器：
/// 统一处理文件引用/清理，不在业务 handler 中散落引用计数逻辑。
pub struct FileManager;

impl FileManager {
    pub async fn on_bot_created(
        pool: &PgPool,
        bot_id: &str,
        avatar_url: Option<&str>,
    ) -> AppResult<()> {
        Self::on_bot_updated(pool, bot_id, None, avatar_url).await
    }

    pub async fn on_bot_updated(
        pool: &PgPool,
        bot_id: &str,
        old_avatar_url: Option<&str>,
        new_avatar_url: Option<&str>,
    ) -> AppResult<()> {
        let old_file_id = old_avatar_url.and_then(refs::extract_file_id_from_download_url);
        let new_file_id = new_avatar_url.and_then(refs::extract_file_id_from_download_url);

        if old_file_id == new_file_id {
            return Ok(());
        }

        if let Some(old_file_id) = old_file_id {
            let _ = refs::remove_reference(pool, &old_file_id, refs::REF_TYPE_BOT_AVATAR, bot_id)
                .await?;
            let _ = refs::cleanup_file_if_unreferenced(pool, &old_file_id).await?;
        }

        if let Some(new_file_id) = new_file_id {
            if refs::file_exists(pool, &new_file_id).await? {
                let _ = refs::add_reference(pool, &new_file_id, refs::REF_TYPE_BOT_AVATAR, bot_id)
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

    pub async fn on_bot_deleted(
        pool: &PgPool,
        bot_id: &str,
        old_avatar_url: Option<&str>,
    ) -> AppResult<()> {
        Self::on_bot_updated(pool, bot_id, old_avatar_url, None).await
    }

    pub async fn on_message_persisted(
        pool: &PgPool,
        msg_id: i64,
        msg_type: &str,
        content: &Value,
    ) -> AppResult<()> {
        let Some(file_id) = refs::extract_file_id_from_message_content(msg_type, content) else {
            return Ok(());
        };

        if !refs::file_exists(pool, &file_id).await? {
            tracing::warn!(
                "文件消息引用的文件不存在，跳过引用计数: msg_id={}, file_id={}",
                msg_id,
                file_id
            );
            return Ok(());
        }

        let _ = refs::add_reference(pool, &file_id, refs::REF_TYPE_MESSAGE, &msg_id.to_string())
            .await?;
        Ok(())
    }

    pub async fn on_group_deleting(pool: &PgPool, group_id: &str) -> AppResult<()> {
        let affected_file_ids = refs::remove_message_references_for_group(pool, group_id).await?;
        for file_id in affected_file_ids {
            let _ = refs::cleanup_file_if_unreferenced(pool, &file_id).await?;
        }
        Ok(())
    }

    pub async fn remove_uploader_and_cleanup(
        pool: &PgPool,
        file_id: &str,
        uploader_bot_id: &str,
    ) -> AppResult<(bool, bool)> {
        let relation_exists: bool = FileRepository::uploader_relation_exists(
            pool,
            &UploaderRelation {
                file_id,
                uploader_id: uploader_bot_id,
            },
        )
        .await?;

        if !relation_exists {
            return Ok((false, false));
        }

        FileRepository::remove_uploader(
            pool,
            &UploaderRelation {
                file_id,
                uploader_id: uploader_bot_id,
            },
        )
        .await?;

        let physical_deleted = refs::cleanup_file_if_unreferenced(pool, file_id).await?;
        Ok((true, physical_deleted))
    }
}

mod refs {
    use crate::error::{AppError, AppResult};
    use crate::repositories::{
        FileReferenceKey, FileReferenceRepository, MessageReferenceCleanup, NewFileReference,
    };
    use sqlx::PgPool;

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
        pool: &PgPool,
        file_id: &str,
        reference_type: &str,
        reference_id: &str,
    ) -> AppResult<bool> {
        let now = chrono::Utc::now().to_rfc3339();
        let affected = FileReferenceRepository::add_reference_ignore(
            pool,
            &NewFileReference {
                file_id,
                reference_type,
                reference_id,
                created_at: &now,
            },
        )
        .await?;

        Ok(affected > 0)
    }

    pub async fn remove_reference(
        pool: &PgPool,
        file_id: &str,
        reference_type: &str,
        reference_id: &str,
    ) -> AppResult<bool> {
        let affected = FileReferenceRepository::remove_reference(
            pool,
            &FileReferenceKey {
                file_id,
                reference_type,
                reference_id,
            },
        )
        .await?;
        Ok(affected > 0)
    }

    pub async fn file_exists(pool: &PgPool, file_id: &str) -> AppResult<bool> {
        FileReferenceRepository::file_exists(pool, file_id).await
    }

    pub async fn remove_message_references_for_group(
        pool: &PgPool,
        group_id: &str,
    ) -> AppResult<Vec<String>> {
        let cleanup = MessageReferenceCleanup {
            reference_type: REF_TYPE_MESSAGE,
            group_id,
        };
        let affected_file_ids: Vec<String> =
            FileReferenceRepository::list_affected_file_ids_by_group_messages(pool, &cleanup)
                .await?;

        FileReferenceRepository::remove_message_references_by_group(pool, &cleanup).await?;

        Ok(affected_file_ids)
    }

    pub async fn cleanup_file_if_unreferenced(pool: &PgPool, file_id: &str) -> AppResult<bool> {
        let Some(storage_path) = FileReferenceRepository::find_storage_path(pool, file_id).await?
        else {
            return Ok(false);
        };

        let ref_count: i64 = FileReferenceRepository::count_references(pool, file_id).await?;

        if ref_count > 0 {
            return Ok(false);
        }

        FileReferenceRepository::delete_uploaders_by_file_id(pool, file_id).await?;
        FileReferenceRepository::delete_references_by_file_id(pool, file_id).await?;
        FileReferenceRepository::delete_file_by_id(pool, file_id).await?;

        if let Err(err) = tokio::fs::remove_file(&storage_path).await
            && err.kind() != std::io::ErrorKind::NotFound
        {
            return Err(AppError::InternalError(err.to_string()));
        }

        if let Some(parent_dir) = std::path::Path::new(&storage_path).parent()
            && let Err(err) = tokio::fs::remove_dir(parent_dir).await
            && err.kind() != std::io::ErrorKind::NotFound
            && err.kind() != std::io::ErrorKind::DirectoryNotEmpty
        {
            return Err(AppError::InternalError(err.to_string()));
        }

        Ok(true)
    }
}
