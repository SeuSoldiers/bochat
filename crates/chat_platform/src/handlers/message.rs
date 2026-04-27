use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Response,
    Json,
};
use serde_json::json;
use std::collections::HashSet;

use crate::models::CreateMessageRequest;
use crate::repositories::{
    GroupMemberLink, GroupRepository, MessageIdempotencyQuery, MessageRepository,
    MessageWithSenderRow, NewMessage,
};
use crate::services::authz::{bot_has_global_group_access, list_super_admin_bot_ids};
use crate::services::audit::{record_best_effort, AuditRecord};
use crate::services::file_scan::notify_group_owner_if_file_flagged_best_effort;
use crate::services::MessageService;
use crate::ws::WsEvent;
use crate::{
    error::{json_response, AppError, AppResult},
    middlewares::BotAuth,
    AppState,
};

#[tracing::instrument(skip_all)]
pub async fn send_message(
    State(state): State<AppState>,
    Extension(auth): Extension<BotAuth>,
    Json(msg_req): Json<CreateMessageRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始处理消息发送请求 ===");
    tracing::debug!(
        "目标群聊: {}, 消息内容: {}",
        msg_req.group_id,
        msg_req.content
    );

    let requester_bot_id = auth.bot_id;
    tracing::debug!("从 token 解析出请求者 Bot ID: {}", requester_bot_id);

    // 验证群聊存在
    tracing::debug!("正在检查群聊是否存在: {}", msg_req.group_id);
    let group_exists: bool = GroupRepository::exists(&state.pool, &msg_req.group_id)
        .await
        .map_err(|e| {
            tracing::error!("检查群聊存在性时数据库错误: {}", e);
            e
        })?;

    if !group_exists {
        tracing::warn!("消息发送失败: 群聊不存在: {}", msg_req.group_id);
        return Err(AppError::BadRequest("群聊不存在".to_string()));
    }

    tracing::debug!("群聊存在性检查通过");

    // 验证 bot 是群聊的成员
    tracing::debug!("正在检查 Bot 是否为群聊成员...");
    let is_member: bool = GroupRepository::is_member(
        &state.pool,
        &GroupMemberLink {
            group_id: &msg_req.group_id,
            member_id: &requester_bot_id,
        },
    )
        .await
        .map_err(|e| {
            tracing::error!("检查群聊成员时数据库错误: {}", e);
            e
        })?;

    if !is_member && !bot_has_global_group_access(&state.pool, &requester_bot_id).await? {
        tracing::warn!(
            "消息发送失败: Bot 不是群聊成员 - Bot ID: {}, 群聊 ID: {}",
            requester_bot_id,
            msg_req.group_id
        );
        return Err(AppError::BotNotInGroup);
    }

    tracing::debug!("群聊成员检查通过");

    let msg_type = msg_req.msg_type.as_deref().unwrap_or("text");
    let idempotency_key = msg_req.idempotency_key.trim();
    if idempotency_key.is_empty() {
        return Err(AppError::BadRequest("idempotency_key 不能为空".to_string()));
    }
    let content = msg_req.content.to_string();

    let existing_message: Option<MessageWithSenderRow> = state
        .message_record_manager
        .find_idempotent_message(&MessageIdempotencyQuery {
            sender_bot_id: &requester_bot_id,
            group_id: &msg_req.group_id,
            idempotency_key,
        })
        .await
        .map_err(|e| {
            tracing::error!("查询幂等消息时错误: {}", e);
            e
        })?;

    if let Some(existing_message) = existing_message {
        let existing_content = serde_json::from_str(&existing_message.content)
            .unwrap_or_else(|_| serde_json::Value::String(existing_message.content.clone()));
        MessageService::on_message_persisted(
            &state.pool,
            existing_message.msg_id,
            &existing_message.msg_type,
            &existing_content,
        )
        .await?;

        return Ok(json_response(
            StatusCode::OK,
            json!({
                "msg_id": existing_message.msg_id,
                "group_id": existing_message.group_id,
                "sender_id": existing_message.sender_id,
                "sender_name": existing_message.sender_name,
                "sender_avatar_url": existing_message.sender_avatar_url,
                "content": existing_content,
                "msg_type": existing_message.msg_type,
                "created_at": existing_message.created_at,
            }),
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let msg_id = state.message_record_manager.next_id();

    tracing::info!("所有验证通过，正在写入消息缓存并异步落库");
    tracing::debug!("消息ID: {}, 消息类型: {}, 时间戳: {}", msg_id, msg_type, now);

    let inserted_message: MessageWithSenderRow = state
        .message_record_manager
        .send_message(&NewMessage {
            msg_id,
            group_id: &msg_req.group_id,
            sender_id: &requester_bot_id,
            content: &content,
            msg_type,
            idempotency_key,
            created_at: &now,
            sender_name: &auth.name,
            sender_avatar_url: auth.avatar_url.as_deref(),
        })
        .await
        .map_err(|e| {
            tracing::error!("保存消息时错误: {}", e);
            e
        })?;

    let response_content = serde_json::from_str(&inserted_message.content)
        .unwrap_or_else(|_| serde_json::Value::String(inserted_message.content.clone()));
    let response_payload = json!({
        "msg_id": inserted_message.msg_id,
        "group_id": inserted_message.group_id,
        "sender_id": inserted_message.sender_id,
        "sender_name": inserted_message.sender_name,
        "sender_avatar_url": inserted_message.sender_avatar_url,
        "content": response_content,
        "msg_type": inserted_message.msg_type,
        "created_at": inserted_message.created_at,
    });
    MessageService::on_message_persisted(
        &state.pool,
        inserted_message.msg_id,
        &inserted_message.msg_type,
        &response_content,
    )
    .await?;
    notify_group_owner_if_file_flagged_best_effort(
        &state,
        &inserted_message.msg_type,
        &response_content,
        &msg_req.group_id,
        &requester_bot_id,
        &auth.owner_id,
    )
    .await;

    let member_bot_ids: Vec<String> =
        MessageRepository::list_member_bot_ids(&state.pool, &msg_req.group_id).await?;

    let super_admin_bot_ids = list_super_admin_bot_ids(&state.pool).await?;

    let ws_event = WsEvent {
        event_type: "message".to_string(),
        payload: response_payload.clone(),
        timestamp: now.clone(),
    };

    let mut recipients: HashSet<String> = member_bot_ids.into_iter().collect();
    recipients.extend(super_admin_bot_ids);

    for bot_id in recipients {
        state
            .ws_manager
            .broadcast_message(&bot_id, ws_event.clone())
            .await;
    }

    tracing::info!(
        "✅ 消息发送成功 - 消息ID: {}, 群聊ID: {}, 发送者: {}",
        inserted_message.msg_id,
        msg_req.group_id,
        requester_bot_id
    );
    let msg_id_str = inserted_message.msg_id.to_string();

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "bot",
            actor_id: &requester_bot_id,
            user_id: Some(&auth.owner_id),
            bot_id: Some(&requester_bot_id),
            group_id: Some(&msg_req.group_id),
            action: "message.send",
            resource_type: "message",
            resource_id: Some(&msg_id_str),
            details: Some(json!({ "msg_type": inserted_message.msg_type })),
        },
    )
    .await;

    Ok(json_response(StatusCode::CREATED, response_payload))
}
