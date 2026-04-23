use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde_json::json;
use std::collections::HashSet;

use crate::models::CreateMessageRequest;
use crate::repositories::{
    BotRepository, GroupMemberLink, GroupRepository, MessageIdempotencyQuery, MessageRepository,
    MessageWithSenderRow, NewMessage,
};
use crate::services::authz::{bot_has_global_group_access, list_super_admin_bot_ids};
use crate::services::FileService;
use crate::utils::verify_token;
use crate::ws::WsEvent;
use crate::{
    error::{json_response, AppError, AppResult},
    http::{require_bot_bearer_token, token_bot_id},
    AppState,
};

#[tracing::instrument(skip_all)]
pub async fn send_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(msg_req): Json<CreateMessageRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始处理消息发送请求 ===");
    tracing::debug!(
        "目标群聊: {}, 消息内容: {}",
        msg_req.group_id,
        msg_req.content
    );

    // 从 Authorization 头提取 Bearer token
    let token = require_bot_bearer_token(&headers).map_err(|err| {
        tracing::warn!("消息发送失败: {}", err);
        err
    })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id
    let requester_bot_id = if let Ok(bot_id) = token_bot_id(&token) {
        bot_id
    } else {
        tracing::warn!("消息发送失败: Token 格式无效");
        return Err(AppError::InvalidBotToken);
    };
    tracing::debug!("从 token 解析出请求者 Bot ID: {}", requester_bot_id);

    // 查询请求者 bot 并使用其 secret 验证 token
    tracing::debug!("正在查询发送者 Bot...");
    let requester_bot: crate::models::Bot =
        BotRepository::find_by_id(&state.pool, requester_bot_id)
            .await
            .map_err(|e| {
                tracing::error!("查询发送者 Bot 时数据库错误: {}", e);
                e
            })?
            .ok_or_else(|| {
                tracing::warn!("发送者 Bot 不存在: {}", requester_bot_id);
                AppError::InvalidBotToken
            })?;

    tracing::debug!("发送者 Bot 查询成功，所有者: {}", requester_bot.owner_id);

    // 使用 bot 的 secret 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(&token, &requester_bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");
    let sender_bot = requester_bot;

    // 检查发送者 bot 是否活跃
    if sender_bot.status != "active" {
        tracing::warn!("消息发送失败: 发送者 Bot 状态非活跃: {}", sender_bot.status);
        return Err(AppError::BotInactive);
    }

    tracing::debug!("发送者 Bot 状态检查通过");

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
            member_id: &sender_bot.bot_id,
        },
    )
        .await
        .map_err(|e| {
            tracing::error!("检查群聊成员时数据库错误: {}", e);
            e
        })?;

    if !is_member && !bot_has_global_group_access(&state.pool, &sender_bot.bot_id).await? {
        tracing::warn!(
            "消息发送失败: Bot 不是群聊成员 - Bot ID: {}, 群聊 ID: {}",
            sender_bot.bot_id,
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

    let existing_message: Option<MessageWithSenderRow> = MessageRepository::find_idempotent_message(
        &state.pool,
        &MessageIdempotencyQuery {
            sender_bot_id: &sender_bot.bot_id,
            group_id: &msg_req.group_id,
            idempotency_key,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!("查询幂等消息时数据库错误: {}", e);
        e
    })?;

    if let Some(existing_message) = existing_message {
        let existing_content = serde_json::from_str(&existing_message.content)
            .unwrap_or_else(|_| serde_json::Value::String(existing_message.content.clone()));
        FileService::on_message_saved(
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

    tracing::info!("所有验证通过，正在保存消息到数据库");
    tracing::debug!("消息类型: {}, 时间戳: {}", msg_type, now);

    let inserted_message: MessageWithSenderRow = MessageRepository::insert_message_returning(
        &state.pool,
        &NewMessage {
            group_id: &msg_req.group_id,
            sender_id: &sender_bot.bot_id,
            content: &content,
            msg_type,
            idempotency_key,
            created_at: &now,
            sender_name: &sender_bot.name,
            sender_avatar_url: sender_bot.avatar_url.as_deref(),
        },
    )
    .await
    .map_err(|e| {
        tracing::error!("保存消息时数据库错误: {}", e);
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
    FileService::on_message_saved(
        &state.pool,
        inserted_message.msg_id,
        &inserted_message.msg_type,
        &response_content,
    )
    .await?;

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
        sender_bot.bot_id
    );

    Ok(json_response(StatusCode::CREATED, response_payload))
}
