use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::CreateMessageRequest;
use crate::utils::verify_token;
use crate::ws::{WsEvent, WsManager};

#[tracing::instrument(skip(pool, msg_req, ws_manager))]
pub async fn send_message(
    pool: web::Data<DbPool>,
    ws_manager: web::Data<WsManager>,
    http_req: HttpRequest,
    msg_req: web::Json<CreateMessageRequest>,
) -> AppResult<HttpResponse> {
    tracing::info!("=== 开始处理消息发送请求 ===");
    tracing::debug!(
        "目标群聊: {}, 消息内容: {}",
        msg_req.group_id,
        msg_req.content
    );

    // 从 Authorization 头提取 Bearer token
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::warn!("消息发送失败: 缺少 Authorization header");
            AppError::Unauthorized
        })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        tracing::warn!("消息发送失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    }
    let requester_bot_id = parts[0];
    tracing::debug!("从 token 解析出请求者 Bot ID: {}", requester_bot_id);

    // 查询请求者 bot 并使用其 secret 验证 token
    tracing::debug!("正在查询发送者 Bot...");
    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询发送者 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("发送者 Bot 不存在: {}", requester_bot_id);
        AppError::BotNotFound
    })?;

    tracing::debug!("发送者 Bot 查询成功，所有者: {}", requester_bot.owner_id);

    // 使用 bot 的 secret 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(token, &requester_bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    let sender_bot = if let Some(target_bot_id) = msg_req.bot_id.as_ref() {
        let target_bot: crate::models::Bot = sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
        )
        .bind(target_bot_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("查询目标发送 Bot 时数据库错误: {}", e);
            AppError::DatabaseError(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!("目标发送 Bot 不存在: {}", target_bot_id);
            AppError::BotNotFound
        })?;

        if target_bot.owner_id != requester_bot.owner_id {
            tracing::warn!(
                "发送消息失败: 目标 Bot 不属于当前用户, owner_id={}, requester_owner={}",
                target_bot.owner_id,
                requester_bot.owner_id
            );
            return Err(AppError::BotPermissionDenied);
        }

        target_bot
    } else {
        requester_bot
    };

    // 检查发送者 bot 是否活跃
    if sender_bot.status != "active" {
        tracing::warn!("消息发送失败: 发送者 Bot 状态非活跃: {}", sender_bot.status);
        return Err(AppError::BadRequest("发送者 Bot 未激活".to_string()));
    }

    tracing::debug!("发送者 Bot 状态检查通过");

    // 验证群聊存在
    tracing::debug!("正在检查群聊是否存在: {}", msg_req.group_id);
    let group_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE group_id = ?)")
            .bind(&msg_req.group_id)
            .fetch_one(pool.get_ref())
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
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("检查群聊成员时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    if !is_member {
        tracing::warn!(
            "消息发送失败: Bot 不是群聊成员 - Bot ID: {}, 群聊 ID: {}",
            sender_bot.bot_id,
            msg_req.group_id
        );
        return Err(AppError::BotPermissionDenied);
    }

    tracing::debug!("群聊成员检查通过");

    let msg_type = msg_req.msg_type.as_deref().unwrap_or("text");
    let content = msg_req.content.to_string();
    let now = chrono::Utc::now().to_rfc3339();

    tracing::info!("所有验证通过，正在保存消息到数据库");
    tracing::debug!("消息类型: {}, 时间戳: {}", msg_type, now);

    // 将消息插入数据库
    let msg_id: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO messages (group_id, sender_id, content, msg_type, created_at)
        VALUES (?, ?, ?, ?, ?)
        RETURNING msg_id
        "#,
    )
    .bind(&msg_req.group_id)
    .bind(&sender_bot.bot_id)
    .bind(&content)
    .bind(msg_type)
    .bind(&now)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("保存消息时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    let response_payload = json!({
        "msg_id": msg_id,
        "group_id": msg_req.group_id,
        "sender_id": sender_bot.bot_id,
        "sender_name": sender_bot.name,
        "sender_avatar_url": sender_bot.avatar_url,
        "content": msg_req.content,
        "msg_type": msg_type,
        "created_at": now,
    });

    let member_bot_ids: Vec<String> =
        sqlx::query_scalar("SELECT member_id FROM group_members WHERE group_id = ?")
            .bind(&msg_req.group_id)
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let ws_event = WsEvent {
        event_type: "message".to_string(),
        payload: response_payload.clone(),
        timestamp: now.clone(),
    };

    for bot_id in member_bot_ids {
        ws_manager
            .broadcast_message(&bot_id, ws_event.clone())
            .await;
    }

    tracing::info!(
        "✅ 消息发送成功 - 消息ID: {}, 群聊ID: {}, 发送者: {}",
        msg_id,
        msg_req.group_id,
        sender_bot.bot_id
    );

    Ok(HttpResponse::Created().json(response_payload))
}
