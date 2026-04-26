use crate::error::AppResult;
use crate::services::FileManager;
use sqlx::PgPool;

pub struct GroupService;

impl GroupService {
    pub async fn on_group_deleting(pool: &PgPool, group_id: &str) -> AppResult<()> {
        FileManager::on_group_deleting(pool, group_id).await
    }
}
