use reqwest::Method;
use serde_json::Value;

use crate::client::{AuthKind, BochatClient};
use crate::error::SdkResult;
use crate::models::{GroupHistoryResponse, MessageResponse, SendMessageRequest};

/// Message sending and history API facade.
///
/// 消息发送与历史查询 API 门面。
#[derive(Clone)]
pub struct MessagesApi {
    client: BochatClient,
}

impl MessagesApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    /// Send a message as the currently selected bot.
    ///
    /// 以当前选中的 Bot 身份发送消息。
    ///
    /// `idempotency_key` is required by the backend and should be unique for the
    /// logical message you are sending.
    ///
    /// 后端要求必须传入 `idempotency_key`，应为当前逻辑消息提供唯一值。
    pub async fn send(&self, req: SendMessageRequest) -> SdkResult<MessageResponse> {
        self.client
            .request_json(Method::POST, "/api/v1/message/send", AuthKind::Bot, &req)
            .await
    }

    /// Convenience helper for sending a plain text message.
    ///
    /// 发送纯文本消息的便捷方法。
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

    /// Query group message history visible to the current bot.
    ///
    /// 查询当前 Bot 可见的群消息历史。
    ///
    /// `base_id` is exclusive. If omitted, history is loaded from the latest
    /// messages backwards.
    ///
    /// `base_id` 为排他游标；省略时会从最新消息开始向前翻页。
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

    /// Build the JSON payload for a file message body.
    ///
    /// 构造文件消息内容字段对应的 JSON 结构。
    pub fn file_content(url: impl Into<String>) -> Value {
        serde_json::json!({ "url": url.into() })
    }
}
