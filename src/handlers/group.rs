use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateGroupRequest, GroupMemberResponse, GroupResponse};
use crate::utils::{generate_group_id, verify_token};

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

    let user_id = user_bot.owner_id;

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
        INSERT INTO groups (group_id, creator_id, name, description, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&group_id)
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

    tracing::info!("群聊创建成功，正在添加创建者 Bot 到群聊成员...");

    // 将创建者的 Bot 自动添加到群聊成员中
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO group_members (group_id, member_id, member_type, joined_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&group_id)
    .bind(requester_bot_id)
    .bind("bot")
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("添加 Bot 到群聊成员时数据库错误: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    tracing::info!("✅ 群聊创建成功 - 群聊ID: {}, 创建者: {}, 创建者Bot已自动加入", group_id, user_id);

    Ok(HttpResponse::Created().json(json!({
        "group_id": group_id,
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

    // 查询该用户创建的所有群聊
    let user_id = user_bot.owner_id;
    tracing::info!("正在查询用户创建的群聊: {}", user_id);

    let groups: Vec<crate::models::Group> = sqlx::query_as(
        "SELECT group_id, creator_id, name, description, status, created_at, updated_at FROM groups WHERE creator_id = ? ORDER BY created_at DESC"
    )
    .bind(&user_id)
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
        "SELECT group_id, creator_id, name, description, status, created_at, updated_at FROM groups WHERE group_id = ?"
    )
    .bind(group_id.into_inner())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    let response: GroupResponse = group.into();
    Ok(HttpResponse::Ok().json(response))
}

/// Join a bot to a group
#[tracing::instrument(skip(pool))]
pub async fn join_group(
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

    // Verify group exists
    let group_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM groups WHERE group_id = ?)")
            .bind(&group_id_str)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if !group_exists {
        return Err(AppError::BadRequest("Group not found".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Add bot to group (ignore if already a member)
    sqlx::query(
        "INSERT OR IGNORE INTO group_members (group_id, member_id, member_type, joined_at) VALUES (?, ?, ?, ?)"
    )
    .bind(&group_id_str)
    .bind(bot_id)
    .bind("bot")
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    tracing::info!("Bot {} joined group {}", bot_id, group_id_str);

    Ok(HttpResponse::Ok().json(json!({
        "message": "Successfully joined group",
        "group_id": group_id_str,
        "bot_id": bot_id,
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
        "SELECT group_id, creator_id, name, description, status, created_at, updated_at FROM groups WHERE group_id = ?"
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

/// List group members
#[tracing::instrument(skip(pool))]
pub async fn list_group_members(
    pool: web::Data<DbPool>,
    group_id: web::Path<String>,
) -> AppResult<HttpResponse> {
    let members: Vec<crate::models::GroupMember> = sqlx::query_as(
        "SELECT group_id, member_id, member_type, joined_at FROM group_members WHERE group_id = ? ORDER BY joined_at"
    )
    .bind(group_id.into_inner())
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let responses: Vec<GroupMemberResponse> = members.into_iter().map(|m| m.into()).collect();

    Ok(HttpResponse::Ok().json(json!({
        "members": responses,
    })))
}
