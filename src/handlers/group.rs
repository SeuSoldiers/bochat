use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;
use serde::Deserialize;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateGroupRequest, GroupMemberResponse, GroupResponse, JoinGroupRequest};
use crate::utils::{generate_group_id, verify_token};

#[derive(Debug, Deserialize)]
pub struct GroupMessagesQuery {
    pub bot_id: Option<String>,
}

/// 创建新群聊（仅用户可创建）
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 解析 token 获取 bot_id
/// 3. 查询 bot 找到其所有者（用户）
/// 4. 验证 token
/// 5. 创建新群聊并将创建者加入
#[tracing::instrument(skip(pool))]
pub async fn create_group(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    req: web::Json<CreateGroupRequest>,
) -> AppResult<HttpResponse> {
    tracing::info!("=== 开始创建新群聊 ===");
    tracing::debug!("请求数据: 群名称={}, 群描述={}", req.name, req.description.as_deref().unwrap_or("无"));

    // 从 Authorization 头提取 Bearer token
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::warn!("创建群聊失败: 缺少 Authorization header");
            AppError::Unauthorized
        })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        tracing::warn!("创建群聊失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    }
    let requester_bot_id = parts[0];
    tracing::debug!("从 token 解析出 Bot ID: {}", requester_bot_id);

    // 查询 bot 找到其所有者
    tracing::debug!("正在查询 Bot 所有者...");
    let user_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("Bot 不存在: {}", requester_bot_id);
        AppError::BotNotFound
    })?;

    tracing::debug!("Bot 查询成功，所有者（用户）ID: {}", user_bot.owner_id);

    // 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(token, &user_bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    let user_id = user_bot.owner_id.clone();

    let member_bot_id = if let Some(target_bot_id) = req.bot_id.as_ref() {
        let target_bot: crate::models::Bot = sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
        )
        .bind(target_bot_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("查询目标 Bot 时数据库错误: {}", e);
            AppError::DatabaseError(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!("目标 Bot 不存在: {}", target_bot_id);
            AppError::BotNotFound
        })?;

        if target_bot.owner_id != user_id {
            tracing::warn!(
                "创建群聊失败: 目标 Bot 不属于当前用户, owner_id={}, user_id={}",
                target_bot.owner_id,
                user_id
            );
            return Err(AppError::BotPermissionDenied);
        }

        if target_bot.status != "active" {
            tracing::warn!("创建群聊失败: 目标 Bot 未激活: {}", target_bot.bot_id);
            return Err(AppError::BadRequest("目标 Bot 未激活".to_string()));
        }

        target_bot.bot_id
    } else {
        if user_bot.status != "active" {
            tracing::warn!("创建群聊失败: 当前认证 Bot 未激活: {}", user_bot.bot_id);
            return Err(AppError::BadRequest("当前 Bot 未激活".to_string()));
        }
        requester_bot_id.to_string()
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
    .execute(pool.get_ref())
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
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("添加 Bot 到群聊成员时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("✅ 群聊创建成功 - 群聊ID: {}, 创建者: {}, Bot已自动加入: {}", group_id, user_id, member_bot_id);

    Ok(HttpResponse::Created().json(json!({
        "group_id": group_id,
        "group_code": req.group_code,
        "creator_id": user_id,
        "name": req.name,
        "description": req.description,
        "status": "active",
        "created_at": now,
        "updated_at": now,
    })))
}

/// 列出已认证用户创建的群聊
///
/// 返回该用户创建的所有群聊列表
#[tracing::instrument(skip(pool))]
pub async fn list_user_groups(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    tracing::info!("=== 开始查询用户群聊列表 ===");

    // 从 Authorization 头提取 Bearer token
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::warn!("查询群聊列表失败: 缺少 Authorization header");
            AppError::Unauthorized
        })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        tracing::warn!("查询群聊列表失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    }
    let requester_bot_id = parts[0];
    tracing::debug!("从 token 解析出 Bot ID: {}", requester_bot_id);

    // 查询 bot 找到其所有者
    tracing::debug!("正在查询 Bot...");
    let user_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("Bot 不存在: {}", requester_bot_id);
        AppError::BotNotFound
    })?;

    tracing::debug!("Bot 查询成功，所有者 ID: {}", user_bot.owner_id);

    // 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(token, &user_bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    // 查询当前用户创建或其 Bot 已加入的群聊
    tracing::info!("正在查询当前用户可管理/已加入的群聊");

    let groups: Vec<crate::models::Group> = sqlx::query_as(
        r#"
        SELECT DISTINCT g.group_id, g.group_code, g.creator_id, g.name, g.description, g.status, g.created_at, g.updated_at
        FROM groups g
        LEFT JOIN group_members gm ON gm.group_id = g.group_id
        LEFT JOIN bots b ON b.bot_id = gm.member_id
        WHERE g.creator_id = ? OR b.owner_id = ?
        ORDER BY g.created_at DESC
        "#
    )
    .bind(&user_bot.owner_id)
    .bind(&user_bot.owner_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询群聊列表时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("✅ 查询成功，共找到 {} 个群聊", groups.len());
    tracing::debug!("群聊列表: {:?}", groups.iter().map(|g| &g.group_id).collect::<Vec<_>>());

    let responses: Vec<GroupResponse> = groups.into_iter().map(|g| g.into()).collect();

    Ok(HttpResponse::Ok().json(json!({
        "groups": responses,
    })))
}

/// Get group details
#[tracing::instrument(skip(pool))]
pub async fn get_group(
    pool: web::Data<DbPool>,
    group_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let group: crate::models::Group = sqlx::query_as(
        "SELECT group_id, group_code, creator_id, name, description, status, created_at, updated_at FROM groups WHERE group_id = ?"
    )
    .bind(group_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    let response: GroupResponse = group.into();
    Ok(HttpResponse::Ok().json(response))
}

/// Join a bot to a group (支持群ID或群号)
#[tracing::instrument(skip(pool))]
pub async fn join_group(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    req: web::Json<JoinGroupRequest>,
) -> AppResult<HttpResponse> {
    tracing::info!("=== 开始加入群聊 ===");
    tracing::debug!("请求参数: group_id={:?}, group_code={:?}", req.group_id, req.group_code);

    // 至少需要一个标识符
    if req.group_id.is_none() && req.group_code.is_none() {
        tracing::warn!("加入群聊失败: 既没有 group_id 也没有 group_code");
        return Err(AppError::BadRequest("必须提供 group_id 或 group_code".to_string()));
    }

    // Extract token from Authorization header
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::warn!("加入群聊失败: 缺少 Authorization header");
            AppError::Unauthorized
        })?;

    // Parse token to get bot_id
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        tracing::warn!("加入群聊失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    }
    let requester_bot_id = parts[0];
    tracing::debug!("从 token 解析出请求者 Bot ID: {}", requester_bot_id);

    // Get the requester bot
    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("Bot 不存在: {}", requester_bot_id);
        AppError::BotNotFound
    })?;

    // Verify token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(token, &requester_bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    let target_bot_id = if let Some(bot_id) = req.bot_id.as_ref() {
        let target_bot: crate::models::Bot = sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
        )
        .bind(bot_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("查询目标 Bot 时数据库错误: {}", e);
            AppError::DatabaseError(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!("目标 Bot 不存在: {}", bot_id);
            AppError::BotNotFound
        })?;

        if target_bot.owner_id != requester_bot.owner_id {
            tracing::warn!(
                "加入群聊失败: 目标 Bot 不属于当前用户, owner_id={}, requester_owner={}",
                target_bot.owner_id,
                requester_bot.owner_id
            );
            return Err(AppError::BotPermissionDenied);
        }

        if target_bot.status != "active" {
            tracing::warn!("加入群聊失败: 目标 Bot 未激活: {}", target_bot.bot_id);
            return Err(AppError::BadRequest("目标 Bot 未激活".to_string()));
        }

        target_bot.bot_id
    } else {
        if requester_bot.status != "active" {
            tracing::warn!("加入群聊失败: 当前认证 Bot 未激活: {}", requester_bot.bot_id);
            return Err(AppError::BadRequest("当前 Bot 未激活".to_string()));
        }
        requester_bot.bot_id.clone()
    };

    // 查找群聊
    let group_id_str = if let Some(ref gid) = req.group_id {
        tracing::debug!("使用 group_id 查找群聊: {}", gid);
        gid.clone()
    } else {
        let gcode = req.group_code.as_ref().unwrap();
        tracing::debug!("使用 group_code 查找群聊: {}", gcode);

        let found_group: Option<String> = sqlx::query_scalar(
            "SELECT group_id FROM groups WHERE group_code = ?"
        )
        .bind(gcode)
        .fetch_optional(pool.get_ref())
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
            .fetch_one(pool.get_ref())
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
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("添加 Bot 到群聊时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("✅ Bot {} 已加入群聊 {}", target_bot_id, group_id_str);

    Ok(HttpResponse::Ok().json(json!({
        "message": "成功加入群聊",
        "group_id": group_id_str,
        "bot_id": target_bot_id,
    })))
}

/// Leave a group
#[tracing::instrument(skip(pool))]
pub async fn leave_group(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    group_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let group_id_str = group_id.into_inner();

    // Extract token from Authorization header
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    // Parse token to get bot_id
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }
    let bot_id = parts[0];

    // Get the bot
    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Verify token
    let _token_payload = verify_token(token, &bot.secret, 86400)?;

    // Remove bot from group
    sqlx::query("DELETE FROM group_members WHERE group_id = ? AND member_id = ?")
        .bind(&group_id_str)
        .bind(bot_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Bot {} left group {}", bot_id, group_id_str);

    Ok(HttpResponse::Ok().json(json!({
        "message": "Successfully left group",
        "group_id": group_id_str,
        "bot_id": bot_id,
    })))
}

/// Remove a specific owned bot from a group
#[tracing::instrument(skip(pool))]
pub async fn remove_group_member(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    path: web::Path<(String, String)>,
) -> AppResult<HttpResponse> {
    let (group_id_str, target_bot_id) = path.into_inner();

    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }
    let requester_bot_id = parts[0];

    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    let _token_payload = verify_token(token, &requester_bot.secret, 86400)?;

    let target_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(&target_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    if target_bot.owner_id != requester_bot.owner_id {
        return Err(AppError::BotPermissionDenied);
    }

    sqlx::query("DELETE FROM group_members WHERE group_id = ? AND member_id = ?")
        .bind(&group_id_str)
        .bind(&target_bot_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Bot removed from group successfully",
        "group_id": group_id_str,
        "bot_id": target_bot_id,
    })))
}

/// Delete a group (only creator can delete)
#[tracing::instrument(skip(pool))]
pub async fn delete_group(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    group_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let group_id_str = group_id.into_inner();

    // Extract token from Authorization header
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    // Parse token to get bot_id
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }
    let bot_id = parts[0];

    // Get the bot to find its owner
    let user_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    // Verify token
    let _token_payload = verify_token(token, &user_bot.secret, 86400)?;

    // Get group
    let group: crate::models::Group = sqlx::query_as(
        "SELECT group_id, group_code, creator_id, name, description, status, created_at, updated_at FROM groups WHERE group_id = ?"
    )
    .bind(&group_id_str)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    // Check if requester is the creator
    if group.creator_id != user_bot.owner_id {
        return Err(AppError::BotPermissionDenied);
    }

    // Delete group members
    sqlx::query("DELETE FROM group_members WHERE group_id = ?")
        .bind(&group_id_str)
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Delete group
    sqlx::query("DELETE FROM groups WHERE group_id = ?")
        .bind(&group_id_str)
        .execute(pool.get_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Group deleted: {}", group_id_str);

    Ok(HttpResponse::Ok().json(json!({
        "message": "Group deleted successfully",
        "group_id": group_id_str,
    })))
}

/// 获取群聊消息历史
#[tracing::instrument(skip(pool))]
/// 获取群聊消息历史（需要身份验证，只有群内的Bot可以查看）
///
/// 流程:
/// 1. 从请求头提取 Bearer token
/// 2. 解析 token 获取 bot_id
/// 3. 查询 bot 信息并验证 token
/// 4. 验证 bot 是否在该群聊中
/// 5. 返回群聊的消息历史
#[tracing::instrument(skip(pool))]
pub async fn get_group_messages(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    group_id: web::Path<String>,
    query: web::Query<GroupMessagesQuery>,
) -> AppResult<HttpResponse> {
    let group_id_str = group_id.into_inner();

    tracing::info!("=== 获取群聊消息历史 ===");
    tracing::debug!("群聊 ID: {}", group_id_str);

    // 从 Authorization 头提取 Bearer token
    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| {
            tracing::warn!("获取消息失败: 缺少 Authorization header");
            AppError::Unauthorized
        })?;

    tracing::debug!("Token 提取成功");

    // 解析 token 获取 bot_id
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        tracing::warn!("获取消息失败: Token 格式无效");
        return Err(AppError::InvalidToken);
    }
    let requester_bot_id = parts[0];
    tracing::debug!("从 token 解析出 Bot ID: {}", requester_bot_id);

    // 查询 bot 信息
    tracing::debug!("正在查询 Bot 信息...");
    let bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询 Bot 时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| {
        tracing::warn!("Bot 不存在: {}", requester_bot_id);
        AppError::BotNotFound
    })?;

    // 验证 token
    tracing::debug!("正在验证 token...");
    let _token_payload = verify_token(token, &bot.secret, 86400)?;
    tracing::debug!("Token 验证成功");

    // 验证群聊存在
    tracing::debug!("正在验证群聊存在...");
    let group_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE group_id = ?)")
            .bind(&group_id_str)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                tracing::error!("验证群聊存在时数据库错误: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

    if !group_exists {
        tracing::warn!("群聊不存在: {}", group_id_str);
        return Err(AppError::BadRequest("群聊不存在".to_string()));
    }

    // 验证 Bot 是否在群内
    tracing::debug!("正在验证 Bot 是否在群内...");
    let access_bot_id = if let Some(target_bot_id) = query.bot_id.as_ref() {
        let target_bot: crate::models::Bot = sqlx::query_as(
            "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
        )
        .bind(target_bot_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| {
            tracing::error!("查询目标 Bot 时数据库错误: {}", e);
            AppError::DatabaseError(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!("目标 Bot 不存在: {}", target_bot_id);
            AppError::BotNotFound
        })?;

        if target_bot.owner_id != bot.owner_id {
            tracing::warn!(
                "拉取消息失败: 目标 Bot 不属于当前用户, owner_id={}, requester_owner={}",
                target_bot.owner_id,
                bot.owner_id
            );
            return Err(AppError::BotPermissionDenied);
        }

        target_bot.bot_id
    } else {
        bot.bot_id.clone()
    };

    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = ? AND member_id = ?)"
    )
    .bind(&group_id_str)
    .bind(&access_bot_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("检查群组成员时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    if !is_member {
        tracing::warn!("Bot {} 不是群 {} 的成员，无权查看消息", access_bot_id, group_id_str);
        return Err(AppError::Forbidden("只有群内的Bot才能查看消息".to_string()));
    }

    tracing::debug!("✅ Bot 是群内成员，继续获取消息");

    // 获取群聊的所有消息，按创建时间升序排列
    let messages: Vec<crate::models::Message> = sqlx::query_as(
        "SELECT msg_id, group_id, sender_id, content, msg_type, created_at FROM messages WHERE group_id = ? ORDER BY created_at ASC"
    )
    .bind(&group_id_str)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("查询消息历史时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("✅ 成功获取 {} 条消息", messages.len());

    let responses: Vec<crate::models::MessageResponse> = messages
        .into_iter()
        .map(|m| m.into())
        .collect();

    Ok(HttpResponse::Ok().json(json!({
        "group_id": group_id_str,
        "messages": responses,
    })))
}

/// List group members
#[tracing::instrument(skip(pool))]
pub async fn list_group_members(
    pool: web::Data<DbPool>,
    http_req: HttpRequest,
    group_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let group_id_str = group_id.into_inner();

    let token = http_req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }
    let requester_bot_id = parts[0];

    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    let _token_payload = verify_token(token, &requester_bot.secret, 86400)?;

    let can_view: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM groups g
            LEFT JOIN group_members gm ON gm.group_id = g.group_id
            LEFT JOIN bots b ON b.bot_id = gm.member_id
            WHERE g.group_id = ? AND (g.creator_id = ? OR b.owner_id = ?)
        )
        "#
    )
    .bind(&group_id_str)
    .bind(&requester_bot.owner_id)
    .bind(&requester_bot.owner_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !can_view {
        return Err(AppError::Forbidden("无权查看该群成员".to_string()));
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
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(json!({
        "members": members,
    })))
}
