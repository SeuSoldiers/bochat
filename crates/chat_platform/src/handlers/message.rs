use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde_json::json;

use crate::models::CreateMessageRequest;
use crate::services::authz::{bot_has_global_group_access, list_super_admin_bot_ids};
use crate::utils::verify_token;
use crate::ws::WsEvent;
use crate::{
    error::{json_response, AppError, AppResult},
    http::{require_bot_bearer_token, token_bot_id},
    AppState,
};

#[tracing::instrument(skip(state, msg_req))]
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
    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询发送者 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
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
    let group_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE group_id = ?)")
            .bind(&msg_req.group_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("检查群聊存在性时数据库错误: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

    if !group_exists {
        tracing::warn!("消息发送失败: 群聊不存在: {}", msg_req.group_id);
        return Err(AppError::BadRequest("群聊不存在".to_string()));
    }

    tracing::debug!("群聊存在性检查通过");

    // 验证 bot 是群聊的成员
    tracing::debug!("正在检查 Bot 是否为群聊成员...");
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = ? AND member_id = ?)",
    )
    .bind(&msg_req.group_id)
    .bind(&sender_bot.bot_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("检查群聊成员时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
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

    #[derive(sqlx::FromRow)]
    struct MessageRow {
        msg_id: i64,
        group_id: String,
        sender_id: String,
        sender_name: Option<String>,
        sender_avatar_url: Option<String>,
        content: String,
        msg_type: String,
        created_at: String,
    }

    let existing_message: Option<MessageRow> = sqlx::query_as(
        r#"
        SELECT
            m.msg_id,
            m.group_id,
            m.sender_id,
            b.name as sender_name,
            b.avatar_url as sender_avatar_url,
            m.content,
            m.msg_type,
            m.created_at
        FROM messages m
        LEFT JOIN bots b ON b.bot_id = m.sender_id
        WHERE m.sender_id = ? AND m.group_id = ? AND m.idempotency_key = ?
        "#,
    )
    .bind(&sender_bot.bot_id)
    .bind(&msg_req.group_id)
    .bind(idempotency_key)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询幂等消息时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    if let Some(existing_message) = existing_message {
        let existing_content = serde_json::from_str(&existing_message.content)
            .unwrap_or_else(|_| serde_json::Value::String(existing_message.content.clone()));

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

    let inserted_message: MessageRow = sqlx::query_as(
        r#"
        INSERT INTO messages (group_id, sender_id, content, msg_type, idempotency_key, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING
            msg_id,
            group_id,
            sender_id,
            ? as sender_name,
            ? as sender_avatar_url,
            content,
            msg_type,
            created_at
        "#,
    )
    .bind(&msg_req.group_id)
    .bind(&sender_bot.bot_id)
    .bind(&content)
    .bind(msg_type)
    .bind(idempotency_key)
    .bind(&now)
    .bind(&sender_bot.name)
    .bind(&sender_bot.avatar_url)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("保存消息时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
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

    let member_bot_ids: Vec<String> =
        sqlx::query_scalar("SELECT member_id FROM group_members WHERE group_id = ?")
            .bind(&msg_req.group_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let super_admin_bot_ids = list_super_admin_bot_ids(&state.pool).await?;

    let ws_event = WsEvent {
        event_type: "message".to_string(),
        payload: response_payload.clone(),
        timestamp: now.clone(),
    };

    for bot_id in member_bot_ids {
        state
            .ws_manager
            .broadcast_message(&bot_id, ws_event.clone())
            .await;
    }

    for bot_id in super_admin_bot_ids {
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
