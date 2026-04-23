use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use crate::models::{CreateGroupRequest, GroupMemberResponse, GroupResponse, JoinGroupRequest};
use crate::repositories::{
    BotRepository, GroupMemberLink, GroupMessagesQuery as RepoGroupMessagesQuery, GroupMessagesRow,
    GroupRepository, NewGroup, NewGroupMember,
};
use crate::services::authz::{
    bot_has_global_group_access, can_manage_target_user, user_is_super_admin,
};
use crate::services::FileService;
use crate::utils::generate_group_id;
use crate::{
    error::{json_response, AppError, AppResult},
    middlewares::{BotAuth, UserAuth},
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
    Extension(auth): Extension<UserAuth>,
    Json(req): Json<CreateGroupRequest>,
) -> AppResult<Response> {
    tracing::info!("=== 开始创建新群聊 ===");
    tracing::debug!(
        "请求数据: 群名称={}, 群描述={}",
        req.name,
        req.description.as_deref().unwrap_or("无")
    );

    let user_id = auth.user_id;
    tracing::debug!("从 token 解析出用户 ID: {}", user_id);

    let member_bot_id = if let Some(target_bot_id) = req.bot_id.as_ref() {
        let target_bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, target_bot_id)
            .await
            .map_err(|e| {
                tracing::error!("查询目标 Bot 时数据库错误: {}", e);
                e
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
        let default_bot_id: Option<String> =
            BotRepository::find_default_active_bot_id(&state.pool, &user_id).await?;

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
    let new_group = NewGroup {
        group_id: &group_id,
        group_code: req.group_code.as_deref(),
        creator_id: &user_id,
        name: &req.name,
        description: req.description.as_deref(),
        status: "active",
        now: &now,
    };
    GroupRepository::insert_group(&state.pool, &new_group)
    .await
    .map_err(|e| {
        tracing::error!("创建群聊时数据库错误: {}", e);
        e
    })?;

    tracing::info!("群聊创建成功，正在添加指定 Bot 到群聊成员...");

    // 将创建者的 Bot 自动添加到群聊成员中
    GroupRepository::add_member_ignore(
        &state.pool,
        &NewGroupMember {
            group_id: &group_id,
            member_id: &member_bot_id,
            member_type: "bot",
            joined_at: &now,
        },
    )
        .await
        .map_err(|e| {
            tracing::error!("添加 Bot 到群聊成员时数据库错误: {}", e);
            e
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
    Extension(auth): Extension<UserAuth>,
) -> AppResult<Response> {
    tracing::info!("=== 开始查询用户群聊列表 ===");

    let user_id = auth.user_id;
    tracing::debug!("从 token 解析出用户 ID: {}", user_id);

    let is_super_admin = user_is_super_admin(&state.pool, &user_id).await?;

    // 超级管理员可查看所有群；普通用户仅可查看自己创建或自己 Bot 加入的群。
    tracing::info!("正在查询群聊列表，is_super_admin={}", is_super_admin);

    let groups: Vec<crate::models::Group> = if is_super_admin {
        GroupRepository::list_all(&state.pool).await.map_err(|e| {
            tracing::error!("查询全部群聊时数据库错误: {}", e);
            e
        })?
    } else {
        GroupRepository::list_visible_by_user(&state.pool, &user_id)
            .await
            .map_err(|e| {
                tracing::error!("查询群聊列表时数据库错误: {}", e);
                e
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
    let group: crate::models::Group = GroupRepository::find_by_id(&state.pool, &group_id)
        .await?
        .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    let response: GroupResponse = group.into();
    Ok(json_response(StatusCode::OK, response))
}

/// Join a bot to a group (支持群ID或群号)
#[tracing::instrument(skip_all)]
pub async fn join_group(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
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

    let requester_user_id = auth.user_id;
    tracing::debug!("从 token 解析出请求者用户 ID: {}", requester_user_id);

    let target_bot_id = if let Some(bot_id) = req.bot_id.as_ref() {
        let target_bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, bot_id)
            .await
            .map_err(|e| {
                tracing::error!("查询目标 Bot 时数据库错误: {}", e);
                e
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
        let default_bot_id: Option<String> =
            BotRepository::find_default_active_bot_id(&state.pool, &requester_user_id).await?;

        default_bot_id.ok_or(AppError::NoAvailableBot)?
    };

    // 查找群聊
    let group_id_str = if let Some(ref gid) = req.group_id {
        tracing::debug!("使用 group_id 查找群聊: {}", gid);
        gid.clone()
    } else {
        let gcode = req.group_code.as_ref().unwrap();
        tracing::debug!("使用 group_code 查找群聊: {}", gcode);

        let found_group: Option<String> = GroupRepository::find_group_id_by_code(&state.pool, gcode)
            .await
            .map_err(|e| {
                tracing::error!("查询群聊时数据库错误: {}", e);
                e
            })?;

        found_group.ok_or_else(|| {
            tracing::warn!("群号不存在: {}", gcode);
            AppError::BadRequest("群号不存在".to_string())
        })?
    };

    tracing::debug!("群聊 ID: {}", group_id_str);

    // Verify group exists
    let group_exists: bool = GroupRepository::exists(&state.pool, &group_id_str)
        .await
        .map_err(|e| {
            tracing::error!("验证群聊存在时数据库错误: {}", e);
            e
        })?;

    if !group_exists {
        tracing::warn!("群聊不存在: {}", group_id_str);
        return Err(AppError::BadRequest("群聊不存在".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Add bot to group (ignore if already a member)
    GroupRepository::add_member_ignore(
        &state.pool,
        &NewGroupMember {
            group_id: &group_id_str,
            member_id: &target_bot_id,
            member_type: "bot",
            joined_at: &now,
        },
    )
        .await
        .map_err(|e| {
            tracing::error!("添加 Bot 到群聊时数据库错误: {}", e);
            e
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
    Extension(auth): Extension<UserAuth>,
    Path(group_id_str): Path<String>,
    Query(query): Query<LeaveGroupQuery>,
) -> AppResult<Response> {
    let requester_user_id = auth.user_id;
    let bot_id = query.bot_id;

    let bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, &bot_id)
        .await?
        .ok_or(AppError::BotNotFound)?;

    if !can_manage_target_user(&state.pool, &requester_user_id, &bot.owner_id).await? {
        return Err(AppError::BotOwnershipMismatch);
    }

    GroupRepository::remove_member(
        &state.pool,
        &GroupMemberLink {
            group_id: &group_id_str,
            member_id: &bot_id,
        },
    )
    .await?;

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
    Extension(auth): Extension<UserAuth>,
    Path((group_id_str, target_bot_id)): Path<(String, String)>,
) -> AppResult<Response> {
    let requester_user_id = auth.user_id;

    let target_bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, &target_bot_id)
        .await?
        .ok_or(AppError::BotNotFound)?;

    if !can_manage_target_user(&state.pool, &requester_user_id, &target_bot.owner_id).await? {
        return Err(AppError::BotOwnershipMismatch);
    }

    GroupRepository::remove_member(
        &state.pool,
        &GroupMemberLink {
            group_id: &group_id_str,
            member_id: &target_bot_id,
        },
    )
    .await?;

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
    Extension(auth): Extension<UserAuth>,
    Path(group_id_str): Path<String>,
) -> AppResult<Response> {
    let requester_user_id = auth.user_id;

    // Get group
    let group: crate::models::Group = GroupRepository::find_by_id(&state.pool, &group_id_str)
        .await?
        .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    // 超级管理员可删除任意群。
    if !can_manage_target_user(&state.pool, &requester_user_id, &group.creator_id).await? {
        return Err(AppError::Forbidden("只有群创建者才能删除该群".to_string()));
    }

    FileService::on_group_deleted(&state.pool, &group_id_str).await?;

    // 删除群消息，避免 groups 删除时触发 messages 的外键约束错误。
    GroupRepository::delete_messages_by_group(&state.pool, &group_id_str).await?;

    // Delete group members
    GroupRepository::delete_members_by_group(&state.pool, &group_id_str).await?;

    // Delete group
    GroupRepository::delete_group(&state.pool, &group_id_str).await?;

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
    Extension(auth): Extension<BotAuth>,
    Path(group_id_str): Path<String>,
    Query(query): Query<GroupMessagesQuery>,
) -> AppResult<Response> {
    tracing::info!("=== 获取群聊消息历史 ===");
    tracing::debug!("群聊 ID: {}", group_id_str);

    let requester_bot_id = auth.bot_id;
    tracing::debug!("从 token 解析出 Bot ID: {}", requester_bot_id);

    // 验证群聊存在
    tracing::debug!("正在验证群聊存在...");
    let group_exists: bool = GroupRepository::exists(&state.pool, &group_id_str)
        .await
        .map_err(|e| {
            tracing::error!("验证群聊存在时数据库错误: {}", e);
            e
        })?;

    if !group_exists {
        tracing::warn!("群聊不存在: {}", group_id_str);
        return Err(AppError::BadRequest("群聊不存在".to_string()));
    }

    let is_member: bool = GroupRepository::is_member(
        &state.pool,
        &GroupMemberLink {
            group_id: &group_id_str,
            member_id: &requester_bot_id,
        },
    )
        .await
        .map_err(|e| {
            tracing::error!("检查群组成员时数据库错误: {}", e);
            e
        })?;

    if !is_member && !bot_has_global_group_access(&state.pool, &requester_bot_id).await? {
        tracing::warn!(
            "Bot {} 不是群 {} 的成员，无权查看消息",
            requester_bot_id,
            group_id_str
        );
        return Err(AppError::BotNotInGroup);
    }

    tracing::debug!("✅ Bot 是群内成员，继续获取消息");

    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let base_id = query.base_id.unwrap_or(i64::MAX);

    let messages: Vec<GroupMessagesRow> = GroupRepository::list_messages(
        &state.pool,
        &RepoGroupMessagesQuery {
            group_id: &group_id_str,
            base_id,
            limit,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!("查询消息历史时数据库错误: {}", e);
        e
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
    Extension(auth): Extension<UserAuth>,
    Path(group_id_str): Path<String>,
) -> AppResult<Response> {
    let requester_user_id = auth.user_id;

    let can_view: bool =
        GroupRepository::can_user_view_members(&state.pool, &group_id_str, &requester_user_id)
            .await?;

    if !can_view && !user_is_super_admin(&state.pool, &requester_user_id).await? {
        return Err(AppError::Forbidden(
            "只有群创建者或群内 Bot 的拥有者可以查看成员".to_string(),
        ));
    }

    let members: Vec<GroupMemberResponse> = GroupRepository::list_members(&state.pool, &group_id_str).await?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "members": members,
        }),
    ))
}
