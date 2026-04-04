use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::models::{CreateGroupRequest, GroupMemberResponse, GroupResponse, JoinGroupRequest};
use crate::services::authz::{
    bot_has_global_group_access, can_manage_target_user, ensure_user_exists, user_is_super_admin,
};
use crate::services::file_reference;
use crate::utils::{generate_group_id, verify_token, verify_user_token};
use crate::{
    error::{json_response, AppError, AppResult},
    http::{require_bot_bearer_token, require_user_bearer_token, token_bot_id, token_user_id},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct GroupMessagesQuery {
    pub base_id: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LeaveGroupQuery {
    pub bot_id: String,
}

/// 创建新群聊（仅用户可创建）
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 解析 token 获取 bot_id
/// 3. 查询 bot 找到其所有者（用户）
/// 4. 验证 token
/// 5. 创建新群聊并将创建者加入
#[tracing::instrument(skip_all)]
pub async fn create_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateGroupRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始创建新群聊 ===");
    tracing::debug!(
        "请求数据: 群名称={}, 群描述={}",
        req.name,
        req.description.as_deref().unwrap_or("无")
    );

    // 从 Authorization 头提取 Bearer token
    let token = require_user_bearer_token(&headers).map_err(|err| {
        tracing::warn!("创建群聊失败: {}", err);
        err
    })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id
    let user_id = if let Ok(user_id) = token_user_id(&token) {
        user_id.to_string()
    } else {
        tracing::warn!("创建群聊失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出用户 ID: {}", user_id);
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &user_id).await?;

    let member_bot_id = if let Some(target_bot_id) = req.bot_id.as_ref() {
        let target_bot: crate::models::Bot = sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
        )
        .bind(target_bot_id)
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

        if !can_manage_target_user(&state.pool, &user_id, &target_bot.owner_id).await? {
            tracing::warn!(
                "创建群聊失败: 目标 Bot 不属于当前用户, owner_id={}, user_id={}",
                target_bot.owner_id,
                user_id
            );
            return Err(AppError::BotOwnershipMismatch);
        }

        if target_bot.status != "active" {
            tracing::warn!("创建群聊失败: 目标 Bot 未激活: {}", target_bot.bot_id);
            return Err(AppError::BotInactive);
        }

        target_bot.bot_id
    } else {
        let default_bot_id: Option<String> = sqlx::query_scalar(
            "SELECT bot_id FROM bots WHERE owner_id = ? AND status = 'active' ORDER BY created_at ASC LIMIT 1"
        )
        .bind(&user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        default_bot_id.ok_or(AppError::NoAvailableBot)?
    };

    // 验证输入
    if req.name.is_empty() {
        tracing::warn!("创建群聊失败: 群名称不能为空");
        return Err(AppError::BadRequest("群名称是必需的".to_string()));
    }

    tracing::info!("输入验证通过");

    // 创建新群聊
    let group_id = generate_group_id();
    let now = chrono::Utc::now().to_rfc3339();

    tracing::debug!("生成群聊 ID: {}", group_id);
    tracing::debug!("时间戳: {}", now);

    tracing::info!("正在数据库中插入群聊记录: {}", group_id);
    sqlx::query(
        r#"
        INSERT INTO groups (group_id, group_code, creator_id, name, description, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&group_id)
    .bind(&req.group_code)
    .bind(&user_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind("active")
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("创建群聊时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("群聊创建成功，正在添加指定 Bot 到群聊成员...");

    // 将创建者的 Bot 自动添加到群聊成员中
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO group_members (group_id, member_id, member_type, joined_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&group_id)
    .bind(&member_bot_id)
    .bind("bot")
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("添加 Bot 到群聊成员时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!(
        "✅ 群聊创建成功 - 群聊ID: {}, 创建者: {}, Bot已自动加入: {}",
        group_id,
        user_id,
        member_bot_id
    );

    Ok(json_response(
        StatusCode::CREATED,
        json!({
            "group_id": group_id,
            "group_code": req.group_code,
            "creator_id": user_id,
            "name": req.name,
            "description": req.description,
            "status": "active",
            "created_at": now,
            "updated_at": now,
        }),
    ))
}

/// 列出已认证用户创建的群聊
///
/// 返回该用户创建的所有群聊列表
#[tracing::instrument(skip_all)]
pub async fn list_user_groups(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Response> {
    tracing::info!("=== 开始查询用户群聊列表 ===");

    // 从 Authorization 头提取 Bearer token
    let token = require_user_bearer_token(&headers).map_err(|err| {
        tracing::warn!("查询群聊列表失败: {}", err);
        err
    })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id
    let user_id = if let Ok(user_id) = token_user_id(&token) {
        user_id.to_string()
    } else {
        tracing::warn!("查询群聊列表失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出用户 ID: {}", user_id);
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &user_id).await?;

    let is_super_admin = user_is_super_admin(&state.pool, &user_id).await?;

    // 超级管理员可查看所有群；普通用户仅可查看自己创建或自己 Bot 加入的群。
    tracing::info!("正在查询群聊列表，is_super_admin={}", is_super_admin);

    let groups: Vec<crate::models::Group> = if is_super_admin {
        sqlx::query_as(
            r#"
            SELECT g.group_id, g.group_code, g.creator_id, g.name, g.description, g.status, g.created_at, g.updated_at
            FROM groups g
            ORDER BY g.created_at DESC
            "#,
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("查询全部群聊时数据库错误: {}", e);
            AppError::DatabaseError(e.to_string())
        })?
    } else {
        sqlx::query_as(
            r#"
            SELECT DISTINCT g.group_id, g.group_code, g.creator_id, g.name, g.description, g.status, g.created_at, g.updated_at
            FROM groups g
            LEFT JOIN group_members gm ON gm.group_id = g.group_id
            LEFT JOIN bots b ON b.bot_id = gm.member_id
            WHERE g.creator_id = ? OR b.owner_id = ?
            ORDER BY g.created_at DESC
            "#,
        )
        .bind(&user_id)
        .bind(&user_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("查询群聊列表时数据库错误: {}", e);
            AppError::DatabaseError(e.to_string())
        })?
    };

    tracing::info!("✅ 查询成功，共找到 {} 个群聊", groups.len());
    tracing::debug!(
        "群聊列表: {:?}",
        groups.iter().map(|g| &g.group_id).collect::<Vec<_>>()
    );

    let responses: Vec<GroupResponse> = groups.into_iter().map(|g| g.into()).collect();

    Ok(json_response(
        StatusCode::OK,
        json!({
            "groups": responses,
        }),
    ))
}

/// Get group details
#[tracing::instrument(skip_all)]
pub async fn get_group(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
) -> AppResult<Response> {
    let group: crate::models::Group = sqlx::query_as(
        "SELECT group_id, group_code, creator_id, name, description, status, created_at, updated_at FROM groups WHERE group_id = ?"
    )
    .bind(group_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    let response: GroupResponse = group.into();
    Ok(json_response(StatusCode::OK, response))
}

/// Join a bot to a group (支持群ID或群号)
#[tracing::instrument(skip_all)]
pub async fn join_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<JoinGroupRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始加入群聊 ===");
    tracing::debug!(
        "请求参数: group_id={:?}, group_code={:?}",
        req.group_id,
        req.group_code
    );

    // 至少需要一个标识符
    if req.group_id.is_none() && req.group_code.is_none() {
        tracing::warn!("加入群聊失败: 既没有 group_id 也没有 group_code");
        return Err(AppError::BadRequest(
            "必须提供 group_id 或 group_code".to_string(),
        ));
    }

    // Extract token from Authorization header
    let token = require_user_bearer_token(&headers).map_err(|err| {
        tracing::warn!("加入群聊失败: {}", err);
        err
    })?;

    // Parse token to get bot_id
    let requester_user_id = if let Ok(user_id) = token_user_id(&token) {
        user_id.to_string()
    } else {
        tracing::warn!("加入群聊失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    };
    tracing::debug!("从 token 解析出请求者用户 ID: {}", requester_user_id);
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &requester_user_id).await?;

    let target_bot_id = if let Some(bot_id) = req.bot_id.as_ref() {
        let target_bot: crate::models::Bot = sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
        )
        .bind(bot_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("查询目标 Bot 时数据库错误: {}", e);
            AppError::DatabaseError(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!("目标 Bot 不存在: {}", bot_id);
            AppError::BotNotFound
        })?;

        if !can_manage_target_user(&state.pool, &requester_user_id, &target_bot.owner_id).await? {
            tracing::warn!(
                "加入群聊失败: 目标 Bot 不属于当前用户, owner_id={}, requester_owner={}",
                target_bot.owner_id,
                requester_user_id
            );
            return Err(AppError::BotOwnershipMismatch);
        }

        if target_bot.status != "active" {
            tracing::warn!("加入群聊失败: 目标 Bot 未激活: {}", target_bot.bot_id);
            return Err(AppError::BotInactive);
        }

        target_bot.bot_id
    } else {
        let default_bot_id: Option<String> = sqlx::query_scalar(
            "SELECT bot_id FROM bots WHERE owner_id = ? AND status = 'active' ORDER BY created_at ASC LIMIT 1"
        )
        .bind(&requester_user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        default_bot_id.ok_or(AppError::NoAvailableBot)?
    };

    // 查找群聊
    let group_id_str = if let Some(ref gid) = req.group_id {
        tracing::debug!("使用 group_id 查找群聊: {}", gid);
        gid.clone()
    } else {
        let gcode = req.group_code.as_ref().unwrap();
        tracing::debug!("使用 group_code 查找群聊: {}", gcode);

        let found_group: Option<String> =
            sqlx::query_scalar("SELECT group_id FROM groups WHERE group_code = ?")
                .bind(gcode)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| {
                    tracing::error!("查询群聊时数据库错误: {}", e);
                    AppError::DatabaseError(e.to_string())
                })?;

        found_group.ok_or_else(|| {
            tracing::warn!("群号不存在: {}", gcode);
            AppError::BadRequest("群号不存在".to_string())
        })?
    };

    tracing::debug!("群聊 ID: {}", group_id_str);

    // Verify group exists
    let group_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE group_id = ?)")
            .bind(&group_id_str)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("验证群聊存在时数据库错误: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

    if !group_exists {
        tracing::warn!("群聊不存在: {}", group_id_str);
        return Err(AppError::BadRequest("群聊不存在".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Add bot to group (ignore if already a member)
    sqlx::query(
        "INSERT OR IGNORE INTO group_members (group_id, member_id, member_type, joined_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&group_id_str)
    .bind(&target_bot_id)
    .bind("bot")
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("添加 Bot 到群聊时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("✅ Bot {} 已加入群聊 {}", target_bot_id, group_id_str);

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "成功加入群聊",
            "group_id": group_id_str,
            "bot_id": target_bot_id,
        }),
    ))
}

/// Leave a group
#[tracing::instrument(skip_all)]
pub async fn leave_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id_str): Path<String>,
    Query(query): Query<LeaveGroupQuery>,
) -> AppResult<Response> {
    let token = require_user_bearer_token(&headers)?;
    let requester_user_id = token_user_id(&token)?.to_string();
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &requester_user_id).await?;
    let bot_id = query.bot_id;

    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    if !can_manage_target_user(&state.pool, &requester_user_id, &bot.owner_id).await? {
        return Err(AppError::BotOwnershipMismatch);
    }

    sqlx::query("DELETE FROM group_members WHERE group_id = ? AND member_id = ?")
        .bind(&group_id_str)
        .bind(&bot_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Bot {} left group {}", bot_id, group_id_str);

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "Successfully left group",
            "group_id": group_id_str,
            "bot_id": bot_id,
        }),
    ))
}

/// Remove a specific owned bot from a group
#[tracing::instrument(skip_all)]
pub async fn remove_group_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((group_id_str, target_bot_id)): Path<(String, String)>,
) -> AppResult<Response> {
    let token = require_user_bearer_token(&headers)?;
    let requester_user_id = token_user_id(&token)?.to_string();
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &requester_user_id).await?;

    let target_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&target_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    if !can_manage_target_user(&state.pool, &requester_user_id, &target_bot.owner_id).await? {
        return Err(AppError::BotOwnershipMismatch);
    }

    sqlx::query("DELETE FROM group_members WHERE group_id = ? AND member_id = ?")
        .bind(&group_id_str)
        .bind(&target_bot_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "Bot removed from group successfully",
            "group_id": group_id_str,
            "bot_id": target_bot_id,
        }),
    ))
}

/// Delete a group (only creator can delete)
#[tracing::instrument(skip_all)]
pub async fn delete_group(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id_str): Path<String>,
) -> AppResult<Response> {
    let token = require_user_bearer_token(&headers)?;
    let requester_user_id = token_user_id(&token)?.to_string();
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &requester_user_id).await?;

    // Get group
    let group: crate::models::Group = sqlx::query_as(
        "SELECT group_id, group_code, creator_id, name, description, status, created_at, updated_at FROM groups WHERE group_id = ?"
    )
    .bind(&group_id_str)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    // 超级管理员可删除任意群。
    if !can_manage_target_user(&state.pool, &requester_user_id, &group.creator_id).await? {
        return Err(AppError::Forbidden("只有群创建者才能删除该群".to_string()));
    }

    let affected_file_ids =
        file_reference::remove_message_references_for_group(&state.pool, &group_id_str).await?;

    // 删除群消息，避免 groups 删除时触发 messages 的外键约束错误。
    sqlx::query("DELETE FROM messages WHERE group_id = ?")
        .bind(&group_id_str)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Delete group members
    sqlx::query("DELETE FROM group_members WHERE group_id = ?")
        .bind(&group_id_str)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Delete group
    sqlx::query("DELETE FROM groups WHERE group_id = ?")
        .bind(&group_id_str)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    for file_id in affected_file_ids {
        let _ = file_reference::cleanup_file_if_unreferenced(&state.pool, &file_id).await?;
    }

    tracing::info!("Group deleted: {}", group_id_str);

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "Group deleted successfully",
            "group_id": group_id_str,
        }),
    ))
}

/// 获取群聊消息历史
#[tracing::instrument(skip_all)]
/// 获取群聊消息历史（需要身份验证，只有群内的Bot可以查看）
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 解析 token 获取 bot_id
/// 3. 查询 bot 信息并验证 token
/// 4. 验证 bot 是否在该群聊中
/// 5. 返回群聊的消息历史
#[tracing::instrument(skip_all)]
pub async fn get_group_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id_str): Path<String>,
    Query(query): Query<GroupMessagesQuery>,
) -> AppResult<Response> {
    tracing::info!("=== 获取群聊消息历史 ===");
    tracing::debug!("群聊 ID: {}", group_id_str);

    // 从 Authorization 头提取 Bearer token
    let token = require_bot_bearer_token(&headers).map_err(|err| {
        tracing::warn!("获取消息失败: {}", err);
        err
    })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id
    let requester_bot_id = if let Ok(bot_id) = token_bot_id(&token) {
        bot_id
    } else {
        tracing::warn!("获取消息失败: Token 格式无效");
        return Err(AppError::InvalidBotToken);
    };
    tracing::debug!("从 token 解析出 Bot ID: {}", requester_bot_id);

    // 查询 bot 信息
    tracing::debug!("正在查询 Bot 信息...");
    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("Bot 不存在: {}", requester_bot_id);
        AppError::InvalidBotToken
    })?;

    // 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(&token, &bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    // 验证群聊存在
    tracing::debug!("正在验证群聊存在...");
    let group_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE group_id = ?)")
            .bind(&group_id_str)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!("验证群聊存在时数据库错误: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

    if !group_exists {
        tracing::warn!("群聊不存在: {}", group_id_str);
        return Err(AppError::BadRequest("群聊不存在".to_string()));
    }

    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = ? AND member_id = ?)",
    )
    .bind(&group_id_str)
    .bind(&bot.bot_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("检查群组成员时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    if !is_member && !bot_has_global_group_access(&state.pool, &bot.bot_id).await? {
        tracing::warn!(
            "Bot {} 不是群 {} 的成员，无权查看消息",
            bot.bot_id,
            group_id_str
        );
        return Err(AppError::BotNotInGroup);
    }

    tracing::debug!("✅ Bot 是群内成员，继续获取消息");

    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let base_id = query.base_id.unwrap_or(i64::MAX);

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

    let messages: Vec<MessageRow> = sqlx::query_as(
        r#"
        SELECT
            recent.msg_id,
            recent.group_id,
            recent.sender_id,
            b.name as sender_name,
            b.avatar_url as sender_avatar_url,
            recent.content,
            recent.msg_type,
            recent.created_at
        FROM (
            SELECT msg_id, group_id, sender_id, content, msg_type, created_at
            FROM messages
            WHERE group_id = ? AND msg_id < ?
            ORDER BY msg_id DESC
            LIMIT ?
        ) recent
        LEFT JOIN bots b ON b.bot_id = recent.sender_id
        ORDER BY recent.msg_id ASC
        "#,
    )
    .bind(&group_id_str)
    .bind(base_id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("查询消息历史时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("✅ 成功获取 {} 条消息", messages.len());

    let responses: Vec<crate::models::MessageResponse> = messages
        .into_iter()
        .map(|m| {
            let content = serde_json::from_str(&m.content)
                .unwrap_or_else(|_| serde_json::Value::String(m.content.clone()));

            crate::models::MessageResponse {
                msg_id: m.msg_id,
                group_id: m.group_id,
                sender_id: m.sender_id,
                sender_name: m.sender_name,
                sender_avatar_url: m.sender_avatar_url,
                content,
                msg_type: m.msg_type,
                created_at: m.created_at,
            }
        })
        .collect();

    Ok(json_response(
        StatusCode::OK,
        json!({
            "group_id": group_id_str,
            "base_id": if query.base_id.is_some() { Some(base_id) } else { None::<i64> },
            "limit": limit,
            "next_base_id": responses.first().map(|message| message.msg_id),
            "messages": responses,
        }),
    ))
}

/// List group members
#[tracing::instrument(skip_all)]
pub async fn list_group_members(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id_str): Path<String>,
) -> AppResult<Response> {
    let token = require_user_bearer_token(&headers)?;
    let requester_user_id = token_user_id(&token)?.to_string();
    let _token_payload = verify_user_token(&token, &state.config.security.jwt_secret, 86400)?;
    ensure_user_exists(&state.pool, &requester_user_id).await?;

    let can_view: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM groups g
            LEFT JOIN group_members gm ON gm.group_id = g.group_id
            LEFT JOIN bots b ON b.bot_id = gm.member_id
            WHERE g.group_id = ? AND (g.creator_id = ? OR b.owner_id = ?)
        )
        "#,
    )
    .bind(&group_id_str)
    .bind(&requester_user_id)
    .bind(&requester_user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !can_view && !user_is_super_admin(&state.pool, &requester_user_id).await? {
        return Err(AppError::Forbidden(
            "只有群创建者或群内 Bot 的拥有者可以查看成员".to_string(),
        ));
    }

    let members: Vec<GroupMemberResponse> = sqlx::query_as(
        r#"
        SELECT gm.group_id, gm.member_id, gm.member_type, gm.joined_at, b.name as bot_name, b.owner_id
        FROM group_members gm
        LEFT JOIN bots b ON b.bot_id = gm.member_id
        WHERE gm.group_id = ?
        ORDER BY gm.joined_at
        "#
    )
    .bind(&group_id_str)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "members": members,
        }),
    ))
}
