use crate::error::AppResult;
use crate::models::Bot;
use crate::repositories::BotRepository;
use crate::services::FileManager;
use sqlx::PgPool;

pub struct BotService;

impl BotService {
    pub async fn get_bot_by_id(pool: &PgPool, bot_id: &str) -> AppResult<Bot> {
        BotRepository::find_required_by_id(pool, bot_id).await
    }

    pub async fn get_user_bots(pool: &PgPool, user_id: &str) -> AppResult<Vec<Bot>> {
        BotRepository::list_by_owner(pool, user_id).await
    }

    pub async fn on_bot_created(
        pool: &PgPool,
        bot_id: &str,
        avatar_url: Option<&str>,
    ) -> AppResult<()> {
        FileManager::on_bot_created(pool, bot_id, avatar_url).await
    }

    pub async fn on_bot_updated(
        pool: &PgPool,
        bot_id: &str,
        old_avatar_url: Option<&str>,
        new_avatar_url: Option<&str>,
    ) -> AppResult<()> {
        FileManager::on_bot_updated(pool, bot_id, old_avatar_url, new_avatar_url).await
    }

    pub async fn on_bot_deleted(
        pool: &PgPool,
        bot_id: &str,
        old_avatar_url: Option<&str>,
    ) -> AppResult<()> {
        FileManager::on_bot_deleted(pool, bot_id, old_avatar_url).await
    }
}
