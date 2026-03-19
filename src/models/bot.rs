use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BotType {
    Personal,
    Custom,
}

impl BotType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BotType::Personal => "personal",
            BotType::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "personal" => Some(BotType::Personal),
            "custom" => Some(BotType::Custom),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bot {
    pub bot_id: String,
    pub bot_type: String,
    pub owner_id: String,
    pub name: String,
    #[serde(skip)]
    pub token: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub bot_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotResponse {
    pub bot_id: String,
    pub bot_type: String,
    pub owner_id: String,
    pub name: String,
    pub token: String,
    pub created_at: String,
}

impl From<Bot> for BotResponse {
    fn from(bot: Bot) -> Self {
        BotResponse {
            bot_id: bot.bot_id,
            bot_type: bot.bot_type,
            owner_id: bot.owner_id,
            name: bot.name,
            token: bot.token,
            created_at: bot.created_at,
        }
    }
}
