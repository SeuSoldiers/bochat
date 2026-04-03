use reqwest::Method;
use serde_json::Value;

use crate::client::{AuthKind, BochatClient};
use crate::error::SdkResult;
use crate::models::{GroupHistoryResponse, MessageResponse, SendMessageRequest};

#[derive(Clone)]
pub struct MessagesApi {
    client: BochatClient,
}

impl MessagesApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    pub async fn send(&self, req: SendMessageRequest) -> SdkResult<MessageResponse> {
        self.client
            .request_json(Method::POST, "/api/v1/message/send", AuthKind::Bot, &req)
            .await
    }

    pub async fn send_text(
        &self,
        group_id: impl Into<String>,
        text: impl Into<String>,
        idempotency_key: impl Into<String>,
    ) -> SdkResult<MessageResponse> {
        let req = SendMessageRequest {
            group_id: group_id.into(),
            content: serde_json::json!({ "text": text.into() }),
            msg_type: Some("text".to_string()),
            idempotency_key: idempotency_key.into(),
        };
        self.send(req).await
    }

    pub async fn history(
        &self,
        group_id: &str,
        base_id: Option<i64>,
        limit: Option<i64>,
    ) -> SdkResult<GroupHistoryResponse> {
        let mut path = format!("/api/v1/groups/{}/messages", group_id);
        let mut query = Vec::new();
        if let Some(v) = base_id {
            query.push(format!("base_id={}", v));
        }
        if let Some(v) = limit {
            query.push(format!("limit={}", v));
        }
        if !query.is_empty() {
            path.push('?');
            path.push_str(&query.join("&"));
        }

        self.client.get_json(&path, AuthKind::Bot).await
    }

    pub fn file_content(url: impl Into<String>) -> Value {
        serde_json::json!({ "url": url.into() })
    }
}
