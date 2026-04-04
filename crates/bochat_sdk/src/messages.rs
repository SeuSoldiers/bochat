use reqwest::Method;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

use crate::client::{AuthKind, BochatClient};
use crate::error::{SdkError, SdkResult};
use crate::models::{GroupHistoryResponse, MessageContent, MessageResponse, SendMessageRequest};

static MESSAGE_IDEMPOTENCY_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Serialize)]
struct SendMessagePayload {
    group_id: String,
    content: MessageContent,
    msg_type: Option<String>,
    idempotency_key: String,
}

fn generate_idempotency_key() -> String {
    let ts_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = MESSAGE_IDEMPOTENCY_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("sdk-{}-{}-{}", std::process::id(), ts_nanos, seq)
}

fn should_retry_send_error(err: &SdkError) -> bool {
    match err {
        SdkError::Transport(_) => true,
        SdkError::Api { status, .. } => *status >= 500 || *status == 429,
        SdkError::HttpStatus { status, .. } => *status >= 500 || *status == 429,
        _ => false,
    }
}

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
    pub async fn send(&self, req: SendMessageRequest) -> SdkResult<MessageResponse> {
        let payload = SendMessagePayload {
            group_id: req.group_id,
            content: req.content,
            msg_type: req.msg_type,
            idempotency_key: generate_idempotency_key(),
        };
        self.send_with_retry(payload).await
    }

    /// Convenience helper for sending a plain text message.
    ///
    /// 发送纯文本消息的便捷方法。
    pub async fn send_text(
        &self,
        group_id: impl Into<String>,
        text: impl Into<String>,
    ) -> SdkResult<MessageResponse> {
        let req = SendMessageRequest {
            group_id: group_id.into(),
            content: MessageContent::text(text),
            msg_type: Some("text".to_string()),
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
    pub fn file_content(url: impl Into<String>) -> MessageContent {
        MessageContent::file(url)
    }

    async fn send_with_retry(&self, payload: SendMessagePayload) -> SdkResult<MessageResponse> {
        let policy = self.client.retry_policy();
        let attempts = policy.max_attempts.max(1);

        for attempt in 0..attempts {
            match self
                .client
                .request_json(
                    Method::POST,
                    "/api/v1/message/send",
                    AuthKind::Bot,
                    &payload,
                )
                .await
            {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    if should_retry_send_error(&err) && attempt + 1 < attempts {
                        sleep(policy.next_delay(attempt)).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(SdkError::Transport("消息发送失败".to_string()))
    }
}
