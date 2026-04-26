use crate::error::AppResult;
use crate::models::File;
use crate::repositories::{FileRepository, UserFilesPage};
use sqlx::SqlitePool;

pub struct FileService;

impl FileService {
    pub async fn get_file_by_id(pool: &SqlitePool, file_id: &str) -> AppResult<File> {
        FileRepository::find_by_id(pool, file_id)
            .await?
            .ok_or(crate::error::AppError::FileNotFound)
    }

    pub async fn get_user_files(
        pool: &SqlitePool,
        user_id: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<File>> {
        FileRepository::list_by_owner_via_uploaders(
            pool,
            &UserFilesPage {
                user_id,
                limit,
                offset,
            },
        )
        .await
    }
}
