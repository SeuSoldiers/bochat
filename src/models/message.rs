use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Text,
    File,
}

impl MessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageType::Text => "text",
            MessageType::File => "file",
        }
    }
}

impl FromStr for MessageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(MessageType::Text),
            "file" => Ok(MessageType::File),
            _ => Err(format!("Unknown message type: {}", s)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub msg_id: i64,
    pub group_id: String,  // 群聊ID
    pub sender_id: String, // 发送者Bot ID
    pub content: String,
    pub msg_type: String,
    pub created_at: String,
}

impl Message {
    pub fn content_as_json(&self) -> serde_json::Result<Value> {
        serde_json::from_str(&self.content)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub group_id: String, // 群聊ID
    pub content: Value,
    pub msg_type: Option<String>,
    pub bot_id: Option<String>, // 指定以哪个 Bot 身份发送
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub msg_id: i64,
    pub group_id: String,
    pub sender_id: String,
    pub content: Value,
    pub msg_type: String,
    pub created_at: String,
}

impl From<Message> for MessageResponse {
    fn from(msg: Message) -> Self {
        let content = msg
            .content_as_json()
            .unwrap_or_else(|_| serde_json::Value::String(msg.content.clone()));

        MessageResponse {
            msg_id: msg.msg_id,
            group_id: msg.group_id,
            sender_id: msg.sender_id,
            content,
            msg_type: msg.msg_type,
            created_at: msg.created_at,
        }
    }
}
