use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(MessageType::Text),
            "file" => Some(MessageType::File),
            _ => None,
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
