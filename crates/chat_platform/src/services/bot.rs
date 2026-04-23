use crate::error::AppResult;
use crate::models::Bot;
use crate::repositories::BotRepository;
use sqlx::SqlitePool;

pub struct BotService;

impl BotService {
    pub async fn get_bot_by_id(pool: &SqlitePool, bot_id: &str) -> AppResult<Bot> {
        BotRepository::find_required_by_id(pool, bot_id).await
    }

    pub async fn get_user_bots(pool: &SqlitePool, user_id: &str) -> AppResult<Vec<Bot>> {
        BotRepository::list_by_owner(pool, user_id).await
    }
}
