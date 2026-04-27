use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard structured error payload returned by the backend.
///
/// 后端返回的标准结构化错误载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
    pub status: u16,
}

/// Response payload returned by register/login endpoints.
///
/// 注册与登录接口返回的响应结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub message: String,
    pub name: String,
    pub token: String,
    pub account: Option<String>,
    pub created_at: Option<String>,
    pub is_super_admin: Option<bool>,
}

/// Public user profile payload for `GET/PUT /api/v1/users/me`.
///
/// `GET/PUT /api/v1/users/me` 使用的用户资料结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub avatar_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub is_super_admin: Option<bool>,
}

/// Bot information returned by bot-related APIs.
///
/// Bot 相关接口返回的 Bot 信息。
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

/// Response wrapper for `GET /api/v1/bots`.
///
/// `GET /api/v1/bots` 的响应包装结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotListResponse {
    pub bots: Vec<BotInfo>,
}

/// Request body for creating a bot.
///
/// 创建 Bot 的请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
}

/// Request body for updating a bot.
///
/// 更新 Bot 的请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBotRequest {
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
}

/// Request body for creating a group.
///
/// 创建群聊的请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub group_code: Option<String>,
    pub bot_id: Option<String>,
}

/// Group information returned by group APIs.
///
/// 群接口返回的群信息。
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

/// Request body for sending a group message.
///
/// 发送群消息的请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub group_id: String,
    pub content: MessageContent,
    pub msg_type: Option<String>,
}

/// Structured message content used by send/history APIs.
///
/// 发送消息与历史消息接口使用的结构化消息内容。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text { text: String },
    File { url: String },
    Custom(Value),
}

impl MessageContent {
    /// Build a text content payload.
    ///
    /// 构建文本消息内容。
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Build a file content payload.
    ///
    /// 构建文件消息内容。
    pub fn file(url: impl Into<String>) -> Self {
        Self::File { url: url.into() }
    }

    /// Build a custom JSON content payload.
    ///
    /// 构建自定义 JSON 消息内容。
    pub fn custom(value: Value) -> Self {
        Self::Custom(value)
    }

    /// Return text when this is a text message.
    ///
    /// 当消息为文本类型时返回文本内容。
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text.as_str()),
            _ => None,
        }
    }

    /// Return file URL when this is a file message.
    ///
    /// 当消息为文件类型时返回文件 URL。
    pub fn as_file_url(&self) -> Option<&str> {
        match self {
            Self::File { url } => Some(url.as_str()),
            _ => None,
        }
    }

    /// Convert to raw JSON value.
    ///
    /// 转换为原始 JSON 值。
    pub fn to_value(&self) -> Value {
        match self {
            Self::Text { text } => serde_json::json!({ "text": text }),
            Self::File { url } => serde_json::json!({ "url": url }),
            Self::Custom(value) => value.clone(),
        }
    }
}

impl From<Value> for MessageContent {
    fn from(value: Value) -> Self {
        serde_json::from_value(value.clone()).unwrap_or(Self::Custom(value))
    }
}

/// Message payload returned by send/history APIs.
///
/// 发送消息与历史消息接口返回的消息结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub msg_id: i64,
    pub group_id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub sender_avatar_url: Option<String>,
    pub content: MessageContent,
    pub msg_type: String,
    pub created_at: String,
}

/// Group history page returned by the backend.
///
/// 后端返回的一页群消息历史。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupHistoryResponse {
    pub group_id: String,
    pub base_id: Option<i64>,
    pub limit: i64,
    pub next_base_id: Option<i64>,
    pub messages: Vec<MessageResponse>,
}

/// Metadata returned after a successful file upload.
///
/// 文件上传成功后返回的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub file_id: String,
    pub filename: String,
    pub url: String,
    pub created_at: String,
}

/// Request body for registering a new user.
///
/// 注册新用户的请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub account: String,
    pub password: String,
    pub name: Option<String>,
}

/// Request body for logging in an existing user.
///
/// 登录已有用户的请求体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: String,
}

/// Request body for updating the current user profile.
///
/// 更新当前用户资料的请求体。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub password: Option<String>,
    pub avatar_url: Option<String>,
}

#[cfg(feature = "ws")]
/// Generic WebSocket event sent by the backend.
///
/// 后端发送的通用 WebSocket 事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: Value,
    pub timestamp: String,
}

#[cfg(feature = "ws")]
/// Strongly typed payload for the initial `connection` event.
///
/// 初始 `connection` 事件对应的强类型载荷。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WsConnectionPayload {
    pub bot_id: String,
    pub bot_name: String,
    pub group_ids: Vec<String>,
}

#[cfg(feature = "ws")]
impl WsEvent {
    /// Read `group_id` from the event payload when present.
    ///
    /// 当事件载荷中存在 `group_id` 时读取该字段。
    pub fn group_id(&self) -> Option<&str> {
        self.payload.get("group_id")?.as_str()
    }

    /// Parse the payload as a connection payload.
    ///
    /// 将事件载荷解析为连接事件载荷。
    pub fn as_connection_payload(&self) -> Option<WsConnectionPayload> {
        if self.event_type != "connection" {
            return None;
        }
        serde_json::from_value::<WsConnectionPayload>(self.payload.clone()).ok()
    }

    /// Parse the payload as a structured message payload.
    ///
    /// 将事件载荷解析为结构化消息载荷。
    pub fn as_message_payload(&self) -> Option<MessageResponse> {
        if self.event_type != "message" {
            return None;
        }
        serde_json::from_value::<MessageResponse>(self.payload.clone()).ok()
    }
}
