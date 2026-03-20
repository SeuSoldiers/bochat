use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BotStatus {
    Active,
    Inactive,
}

impl BotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BotStatus::Active => "active",
            BotStatus::Inactive => "inactive",
        }
    }
}

impl FromStr for BotStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(BotStatus::Active),
            "inactive" => Ok(BotStatus::Inactive),
            _ => Err(format!("Unknown bot status: {}", s)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bot {
    pub bot_id: String,
    pub owner_id: String, // 所属用户ID
    pub name: String,
    pub description: Option<String>,
    pub status: String, // active / inactive
    pub token: String,  // Bot令牌，用于API认证
    #[serde(skip)]
    pub secret: String, // Bot密钥，用于令牌签名
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotResponse {
    pub bot_id: String,
    pub owner_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub token: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Bot> for BotResponse {
    fn from(bot: Bot) -> Self {
        BotResponse {
            bot_id: bot.bot_id,
            owner_id: bot.owner_id,
            name: bot.name,
            description: bot.description,
            status: bot.status,
            token: bot.token,
            created_at: bot.created_at,
            updated_at: bot.updated_at,
        }
    }
}
