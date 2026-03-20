use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::{json_response, AppError, AppResult},
    http::{bearer_token, token_bot_id},
    AppState,
};
use crate::models::{BotResponse, CreateBotRequest, UpdateBotRequest};
use crate::utils::{generate_bot_id, generate_token, verify_token};

/// 为已认证的用户创建新 Bot
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 解析 token 获取请求者的 bot_id
/// 3. 从数据库获取请求者 bot 的 secret
/// 4. 使用 secret 验证 token
/// 5. 创建新 bot 并返回 bot 信息
#[tracing::instrument(skip(state))]
pub async fn create_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateBotRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始创建新 Bot ===");

    // 从 Authorization 头提取 Bearer token
    let token = bearer_token(&headers).map_err(|_| {
        tracing::warn!("创建 Bot 失败: 缺少 Authorization header");
        AppError::Unauthorized
    })?;

    tracing::debug!("Token 提取成功 (前30位): {}", &token[..30.min(token.len())]);

    // 解析 token 获取 bot_id（先不验证）
    let requester_bot_id = if let Ok(bot_id) = token_bot_id(&token) {
        bot_id
    } else {
        tracing::warn!(
            "创建 Bot 失败: Token 格式无效"
        );
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出请求者 Bot ID: {}", requester_bot_id);

    // 从数据库获取请求者 bot（以获取 secret）
    tracing::debug!("正在从数据库查询请求者 Bot...");
    let owner_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询请求者 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("请求者 Bot 不存在: {}", requester_bot_id);
        AppError::BotNotFound
    })?;

    tracing::debug!("请求者 Bot 查询成功，所有者 ID: {}", owner_bot.owner_id);

    // 使用 bot 的 secret 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(&token, &owner_bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    let owner_id = owner_bot.owner_id;

    // 验证输入
    if req.name.is_empty() {
        tracing::warn!("创建 Bot 失败: Bot 名称不能为空");
        return Err(AppError::BadRequest("Bot 名称是必需的".to_string()));
    }

    tracing::info!("输入验证通过，Bot 名称: {}", req.name);

    // 创建新 bot
    let new_bot_id = generate_bot_id();
    let bot_secret = Uuid::new_v4().to_string();
    let bot_token = generate_token(&new_bot_id, &bot_secret)?;
    let now = chrono::Utc::now().to_rfc3339();

    tracing::debug!("生成新 Bot 信息:");
    tracing::debug!("  新 Bot ID: {}", new_bot_id);
    tracing::debug!(
        "  Bot Secret (前16位): {}",
        &bot_secret[..16.min(bot_secret.len())]
    );
    tracing::debug!("  时间戳: {}", now);

    tracing::info!("正在数据库中插入新 Bot 记录: {}", new_bot_id);
    sqlx::query(
        r#"
        INSERT INTO bots (bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&new_bot_id)
    .bind(&owner_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.avatar_url)
    .bind("active")
    .bind(&bot_token)
    .bind(&bot_secret)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("创建 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!(
        "✅ Bot 创建成功 - Bot ID: {}, 所有者: {}",
        new_bot_id,
        owner_id
    );

    Ok(json_response(StatusCode::CREATED, json!({
        "bot_id": new_bot_id,
        "owner_id": owner_id,
        "name": req.name,
        "description": req.description,
        "avatar_url": req.avatar_url,
        "status": "active",
        "token": bot_token,
        "secret": bot_secret,
        "created_at": now,
        "updated_at": now,
    })))
}

/// 列出已认证用户的所有 Bot
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 验证 token 有效性
/// 3. 查询该用户的所有 bot
/// 4. 返回 bot 列表
#[tracing::instrument(skip(state))]
pub async fn list_bots(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Response> {
    tracing::info!("=== 开始查询 Bot 列表 ===");

    // 从 Authorization 头提取 Bearer token
    let token = bearer_token(&headers).map_err(|_| {
        tracing::warn!("查询 Bot 列表失败: 缺少 Authorization header");
        AppError::Unauthorized
    })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id（先不验证）
    let requester_bot_id = if let Ok(bot_id) = token_bot_id(&token) {
        bot_id
    } else {
        tracing::warn!("查询 Bot 列表失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出 Bot ID: {}", requester_bot_id);

    // 从数据库获取请求者 bot（以获取 owner_id 和 secret）
    tracing::debug!("正在查询请求者 Bot...");
    let owner_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询请求者 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("请求者 Bot 不存在: {}", requester_bot_id);
        AppError::BotNotFound
    })?;

    tracing::debug!("请求者 Bot 查询成功，所有者 ID: {}", owner_bot.owner_id);

    // 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(&token, &owner_bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    // 查询该用户的所有 bot
    let owner_id = owner_bot.owner_id;
    tracing::info!("正在查询用户所有 Bot: {}", owner_id);

    let bots: Vec<crate::models::Bot> = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE owner_id = ? ORDER BY created_at DESC"
    )
    .bind(&owner_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询 Bot 列表时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("✅ 查询成功，共找到 {} 个 Bot", bots.len());
    tracing::debug!(
        "Bot 列表: {:?}",
        bots.iter().map(|b| &b.bot_id).collect::<Vec<_>>()
    );

    let responses: Vec<BotResponse> = bots.into_iter().map(|b| b.into()).collect();

    Ok(json_response(StatusCode::OK, json!({
        "bots": responses,
    })))
}

/// 获取指定的 Bot 信息
#[tracing::instrument(skip(state))]
pub async fn get_bot(
    State(state): State<AppState>,
    Path(requested_bot_id): Path<String>,
) -> AppResult<Response> {
    tracing::info!("=== 查询 Bot 详情: {} ===", requested_bot_id);

    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&requested_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("Bot 不存在: {}", requested_bot_id);
        AppError::BotNotFound
    })?;

    tracing::info!("✅ Bot 查询成功: {}", requested_bot_id);
    tracing::debug!(
        "Bot 信息: 名称={}, 所有者={}, 状态={}",
        bot.name,
        bot.owner_id,
        bot.status
    );

    let response: BotResponse = bot.into();
    Ok(json_response(StatusCode::OK, response))
}

/// 删除 Bot（仅所有者可删除）
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 解析 token 获取请求者 bot_id
/// 3. 验证请求者身份
/// 4. 检查请求者是否是目标 bot 的所有者
/// 5. 删除 bot（消息会被保留，因为没有外键约束）
#[tracing::instrument(skip(state))]
pub async fn delete_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_bot_id): Path<String>,
) -> AppResult<Response> {
    tracing::info!("=== 开始删除 Bot: {} ===", target_bot_id);

    // 从 Authorization 头提取 Bearer token
    let token = bearer_token(&headers).map_err(|_| {
        tracing::warn!("删除 Bot 失败: 缺少 Authorization header");
        AppError::Unauthorized
    })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取请求者 bot_id
    let requester_bot_id = if let Ok(bot_id) = token_bot_id(&token) {
        bot_id
    } else {
        tracing::warn!("删除 Bot 失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出请求者 Bot ID: {}", requester_bot_id);

    // 查询请求者 bot
    tracing::debug!("正在查询请求者 Bot...");
    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询请求者 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("请求者 Bot 不存在: {}", requester_bot_id);
        AppError::BotNotFound
    })?;

    tracing::debug!("请求者 Bot 查询成功，所有者: {}", requester_bot.owner_id);

    // 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(&token, &requester_bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    // 查询目标 bot
    tracing::debug!("正在查询目标 Bot: {}", target_bot_id);
    let target_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&target_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询目标 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("目标 Bot 不存在: {}", target_bot_id);
        AppError::BotNotFound
    })?;

    tracing::debug!("目标 Bot 查询成功，所有者: {}", target_bot.owner_id);

    // 检查请求者是否是目标 bot 的所有者
    if requester_bot.owner_id != target_bot.owner_id {
        tracing::warn!(
            "删除 Bot 权限检查失败: 请求者所有者={}, 目标所有者={}",
            requester_bot.owner_id,
            target_bot.owner_id
        );
        return Err(AppError::BotPermissionDenied);
    }

    tracing::info!("权限检查通过，开始删除 Bot 记录");

    // 删除 bot（消息会被保留，因为没有外键约束）
    sqlx::query("DELETE FROM bots WHERE bot_id = ?")
        .bind(&target_bot_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("删除 Bot 时数据库错误: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

    tracing::info!(
        "✅ Bot 删除成功 - Bot ID: {}, 所有者: {}",
        target_bot_id,
        requester_bot.owner_id
    );

    Ok(json_response(StatusCode::OK, json!({
        "message": "Bot 删除成功",
        "bot_id": target_bot_id,
    })))
}

#[tracing::instrument(skip(state))]
pub async fn update_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_bot_id): Path<String>,
    Json(req): Json<UpdateBotRequest>,
) -> AppResult<Response> {

    let token = bearer_token(&headers)?;
    let requester_bot_id = token_bot_id(&token)?;

    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    let _token_payload = verify_token(&token, &requester_bot.secret, 86400)?;

    let target_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&target_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    if requester_bot.owner_id != target_bot.owner_id {
        return Err(AppError::BotPermissionDenied);
    }

    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("Bot 名称是必需的".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE bots
        SET name = ?, description = ?, avatar_url = ?, updated_at = ?
        WHERE bot_id = ?
        "#,
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.avatar_url)
    .bind(&now)
    .bind(&target_bot_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let updated_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&target_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    Ok(json_response(StatusCode::OK, BotResponse::from(updated_bot)))
}
