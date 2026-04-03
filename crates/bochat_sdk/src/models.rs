use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
    pub status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub message: String,
    pub name: String,
    pub token: String,
    pub phone: Option<String>,
    pub account: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotInfo {
    pub bot_id: String,
    pub owner_id: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub status: String,
    pub token: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotListResponse {
    pub bots: Vec<BotInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBotRequest {
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub group_code: Option<String>,
    pub bot_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub group_code: Option<String>,
    pub creator_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub group_id: String,
    pub content: Value,
    pub msg_type: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub msg_id: i64,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub sender_avatar_url: Option<String>,
    pub content: Value,
    pub msg_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupHistoryResponse {
    pub group_id: String,
    pub base_id: Option<i64>,
    pub limit: i64,
    pub next_base_id: Option<i64>,
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub file_id: String,
    pub url: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub account: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub password: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
}

#[cfg(feature = "ws")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: Value,
    pub timestamp: String,
}
