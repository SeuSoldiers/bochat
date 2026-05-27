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
    pub idempotency_key: Option<String>,
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
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl From<Message> for MessageResponse {
    fn from(msg: Message) -> Self {
        let content = msg
            .content_as_json()
            .unwrap_or_else(|_| serde_json::Value::String(msg.content.clone()));

        MessageResponse {
            msg_id: msg.msg_id,
            group_id: msg.group_id,
            sender_id: msg.sender_id,
            sender_name: None,
            sender_avatar_url: None,
            content,
            msg_type: msg.msg_type,
            created_at: msg.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::str::FromStr;

    use super::{Message, MessageResponse, MessageType};

    #[test]
    fn message_type_str_and_parse() {
        assert_eq!(MessageType::Text.as_str(), "text");
        assert_eq!(MessageType::File.as_str(), "file");
        assert!(matches!(MessageType::from_str("text"), Ok(MessageType::Text)));
        assert!(MessageType::from_str("unknown").is_err());
    }

    #[test]
    fn message_response_parses_json_content() {
        let msg = Message {
            msg_id: 1,
            group_id: "g1".to_string(),
            sender_id: "b1".to_string(),
            content: r#"{"text":"hello"}"#.to_string(),
            msg_type: "text".to_string(),
            idempotency_key: Some("k1".to_string()),
            created_at: "now".to_string(),
        };

        let resp = MessageResponse::from(msg);
        assert_eq!(resp.msg_id, 1);
        assert_eq!(resp.content, json!({"text":"hello"}));
    }

    #[test]
    fn message_response_fallbacks_to_string_for_invalid_json() {
        let msg = Message {
            msg_id: 2,
            group_id: "g1".to_string(),
            sender_id: "b1".to_string(),
            content: "not-json".to_string(),
            msg_type: "text".to_string(),
            idempotency_key: None,
            created_at: "now".to_string(),
        };

        let resp = MessageResponse::from(msg);
        assert_eq!(resp.content, serde_json::Value::String("not-json".to_string()));
    }
}
