use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::models::{BotResponse, CreateBotRequest, UpdateBotRequest};
use crate::repositories::{BotRepository, NewBot};
use crate::services::authz::{can_manage_target_user, ensure_user_exists, user_is_super_admin};
use crate::services::FileService;
use crate::utils::{generate_bot_id, generate_token, verify_user_token};
use crate::{
    error::{json_response, AppError, AppResult},
    http::{require_user_bearer_token, token_user_id},
    AppState,
};

/// 为已认证的用户创建新 Bot
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 解析 token 获取请求者的 bot_id
/// 3. 从数据库获取请求者 bot 的 secret
/// 4. 使用 secret 验证 token
/// 5. 创建新 bot 并返回 bot 信息
#[tracing::instrument(skip_all)]
pub async fn create_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateBotRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始创建新 Bot ===");

    // 从 Authorization 头提取 Bearer token
    let token = require_user_bearer_token(&headers).map_err(|err| {
        tracing::warn!("创建 Bot 失败: {}", err);
        err
    })?;

    tracing::debug!("Token 提取成功 (前30位): {}", &token[..30.min(token.len())]);

    let owner_id = if let Ok(user_id) = token_user_id(&token) {
        user_id.to_string()
    } else {
        tracing::warn!("创建 Bot 失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出用户 ID: {}", owner_id);
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &owner_id).await?;

    // 验证输入
    if req.name.is_empty() {
        tracing::warn!("创建 Bot 失败: Bot 名称不能为空");
        return Err(AppError::BadRequest("Bot 名称是必需的".to_string()));
    }

    tracing::trace!("输入验证通过，Bot 名称: {}", req.name);

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
    let new_bot = NewBot {
        bot_id: &new_bot_id,
        owner_id: &owner_id,
        name: &req.name,
        description: req.description.as_deref(),
        avatar_url: req.avatar_url.as_deref(),
        status: "active",
        token: &bot_token,
        secret: &bot_secret,
        now: &now,
    };
    BotRepository::insert(&state.pool, &new_bot)
    .await
    .map_err(|e| {
        tracing::error!("创建 Bot 时数据库错误: {}", e);
        e
    })?;

    FileService::on_bot_avatar_changed(&state.pool, &new_bot_id, None, req.avatar_url.as_deref())
        .await?;

    tracing::info!(
        "✅ Bot 创建成功 - Bot ID: {}, 所有者: {}",
        new_bot_id,
        owner_id
    );

    Ok(json_response(
        StatusCode::CREATED,
        json!({
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
        }),
    ))
}

/// 列出已认证用户的所有 Bot
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 验证 token 有效性
/// 3. 查询该用户的所有 bot
/// 4. 返回 bot 列表
#[tracing::instrument(skip_all)]
pub async fn list_bots(State(state): State<AppState>, headers: HeaderMap) -> AppResult<Response> {
    tracing::info!("=== 开始查询 Bot 列表 ===");

    // 从 Authorization 头提取 Bearer token
    let token = require_user_bearer_token(&headers).map_err(|err| {
        tracing::warn!("查询 Bot 列表失败: {}", err);
        err
    })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id（先不验证）
    let owner_id = if let Ok(user_id) = token_user_id(&token) {
        user_id.to_string()
    } else {
        tracing::warn!("查询 Bot 列表失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出用户 ID: {}", owner_id);
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &owner_id).await?;

    let is_super_admin = user_is_super_admin(&state.pool, &owner_id).await?;

    // 超级管理员可查看全量 Bot；普通用户仅查看自己名下 Bot。
    tracing::info!(
        "正在查询 Bot 列表，owner_id={}, is_super_admin={}",
        owner_id,
        is_super_admin
    );

    let bots: Vec<crate::models::Bot> = if is_super_admin {
        BotRepository::list_all(&state.pool).await.map_err(|e| {
            tracing::error!("查询全量 Bot 列表时数据库错误: {}", e);
            e
        })?
    } else {
        BotRepository::list_by_owner(&state.pool, &owner_id)
            .await
            .map_err(|e| {
                tracing::error!("查询 Bot 列表时数据库错误: {}", e);
                e
            })?
    };

    tracing::info!("✅ 查询成功，共找到 {} 个 Bot", bots.len());
    tracing::debug!(
        "Bot 列表: {:?}",
        bots.iter().map(|b| &b.bot_id).collect::<Vec<_>>()
    );

    let responses: Vec<BotResponse> = bots.into_iter().map(|b| b.into()).collect();

    Ok(json_response(
        StatusCode::OK,
        json!({
            "bots": responses,
        }),
    ))
}

/// 获取指定的 Bot 信息
#[tracing::instrument(skip_all)]
pub async fn get_bot(
    State(state): State<AppState>,
    Path(requested_bot_id): Path<String>,
) -> AppResult<Response> {
    tracing::info!("=== 查询 Bot 详情: {} ===", requested_bot_id);

    let bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, &requested_bot_id)
        .await
        .map_err(|e| {
            tracing::error!("查询 Bot 时数据库错误: {}", e);
            e
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
#[tracing::instrument(skip_all)]
pub async fn delete_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_bot_id): Path<String>,
) -> AppResult<Response> {
    tracing::info!("=== 开始删除 Bot: {} ===", target_bot_id);

    // 从 Authorization 头提取 Bearer token
    let token = require_user_bearer_token(&headers).map_err(|err| {
        tracing::warn!("删除 Bot 失败: {}", err);
        err
    })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取请求者 bot_id
    let requester_user_id = if let Ok(user_id) = token_user_id(&token) {
        user_id.to_string()
    } else {
        tracing::warn!("删除 Bot 失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出请求者用户 ID: {}", requester_user_id);
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &requester_user_id).await?;

    // 查询目标 bot
    tracing::debug!("正在查询目标 Bot: {}", target_bot_id);
    let target_bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, &target_bot_id)
        .await
        .map_err(|e| {
            tracing::error!("查询目标 Bot 时数据库错误: {}", e);
            e
        })?
        .ok_or_else(|| {
            tracing::warn!("目标 Bot 不存在: {}", target_bot_id);
            AppError::BotNotFound
        })?;

    tracing::debug!("目标 Bot 查询成功，所有者: {}", target_bot.owner_id);

    // 超级管理员可管理任意 Bot。
    if !can_manage_target_user(&state.pool, &requester_user_id, &target_bot.owner_id).await? {
        tracing::warn!(
            "删除 Bot 权限检查失败: 请求者所有者={}, 目标所有者={}",
            requester_user_id,
            target_bot.owner_id
        );
        return Err(AppError::BotOwnershipMismatch);
    }

    tracing::info!("权限检查通过，开始删除 Bot 记录");

    // 删除 bot（消息会被保留，因为没有外键约束）
    BotRepository::delete_by_id(&state.pool, &target_bot_id)
        .await
        .map_err(|e| {
            tracing::error!("删除 Bot 时数据库错误: {}", e);
            e
        })?;

    FileService::on_bot_avatar_changed(
        &state.pool,
        &target_bot_id,
        target_bot.avatar_url.as_deref(),
        None,
    )
    .await?;

    tracing::info!(
        "✅ Bot 删除成功 - Bot ID: {}, 所有者: {}",
        target_bot_id,
        requester_user_id
    );

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "Bot 删除成功",
            "bot_id": target_bot_id,
        }),
    ))
}

#[tracing::instrument(skip_all)]
pub async fn update_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_bot_id): Path<String>,
    Json(req): Json<UpdateBotRequest>,
) -> AppResult<Response> {
    let token = require_user_bearer_token(&headers)?;
    let requester_user_id = token_user_id(&token)?;
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, requester_user_id).await?;

    let target_bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, &target_bot_id)
        .await?
        .ok_or(AppError::BotNotFound)?;

    if !can_manage_target_user(&state.pool, &requester_user_id, &target_bot.owner_id).await? {
        return Err(AppError::BotOwnershipMismatch);
    }

    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("Bot 名称是必需的".to_string()));
    }

    let old_avatar_url = target_bot.avatar_url.clone();
    let now = chrono::Utc::now().to_rfc3339();

    BotRepository::update_profile(
        &state.pool,
        &target_bot_id,
        &req.name,
        req.description.as_deref(),
        req.avatar_url.as_deref(),
        &now,
    )
    .await?;

    FileService::on_bot_avatar_changed(
        &state.pool,
        &target_bot_id,
        old_avatar_url.as_deref(),
        req.avatar_url.as_deref(),
    )
    .await?;

    let updated_bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, &target_bot_id)
        .await?
        .ok_or(AppError::BotNotFound)?;

    Ok(json_response(
        StatusCode::OK,
        BotResponse::from(updated_bot),
    ))
}
