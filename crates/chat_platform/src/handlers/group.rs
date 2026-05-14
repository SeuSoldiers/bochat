use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Response,
};
use serde::Deserialize;
use serde_json::json;

use crate::models::{
    CreateGroupRequest, GroupMemberResponse, GroupResponse, JoinGroupRequest, UpdateGroupRequest,
};
use crate::repositories::{
    BotRepository, GroupJoinRequestRepository, GroupMemberLink, GroupRepository, NewGroup,
    NewGroupJoinRequest, NewGroupMember, UpdateGroupProfile, UserRepository,
};
use crate::services::GroupService;
use crate::services::audit::{AuditRecord, record_best_effort};
use crate::services::authz::{
    bot_has_global_group_access, can_manage_target_user, user_is_super_admin,
};
use crate::services::notification::{
    NotificationRecord, create_best_effort as create_notification_best_effort,
};
use crate::utils::{generate_group_id, generate_group_join_request_id};
use crate::{
    AppState,
    error::{AppError, AppResult, json_response},
    middlewares::{BotAuth, UserAuth},
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

#[derive(Debug, Deserialize)]
pub struct SearchGroupQuery {
    pub group_code: String,
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
        avatar_url: req.avatar_url.as_deref(),
        is_public: req.is_public.unwrap_or(false),
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

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &user_id,
            user_id: Some(&user_id),
            bot_id: Some(&member_bot_id),
            group_id: Some(&group_id),
            action: "group.create",
            resource_type: "group",
            resource_id: Some(&group_id),
            details: Some(json!({ "name": req.name, "is_public": req.is_public.unwrap_or(false) })),
        },
    )
    .await;

    Ok(json_response(
        StatusCode::CREATED,
        json!({
            "group_id": group_id,
            "group_code": req.group_code,
            "creator_id": user_id,
            "name": req.name,
            "description": req.description,
            "avatar_url": req.avatar_url,
            "is_public": req.is_public.unwrap_or(false),
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

#[tracing::instrument(skip_all)]
pub async fn list_bot_groups(
    State(state): State<AppState>,
    Extension(auth): Extension<BotAuth>,
) -> AppResult<Response> {
    let groups = GroupRepository::list_by_bot_member(&state.pool, &auth.bot_id).await?;
    let responses: Vec<GroupResponse> = groups.into_iter().map(GroupResponse::from).collect();

    Ok(json_response(
        StatusCode::OK,
        json!({
            "groups": responses,
        }),
    ))
}

/// 按群号前缀搜索群信息
#[tracing::instrument(skip_all)]
pub async fn search_group_by_code(
    State(state): State<AppState>,
    Query(query): Query<SearchGroupQuery>,
) -> AppResult<Response> {
    let group_code = query.group_code.trim();
    let groups: Vec<crate::models::Group> = if group_code.is_empty() {
        GroupRepository::list_public(&state.pool).await?
    } else {
        GroupRepository::find_by_code_prefix(&state.pool, group_code).await?
    };

    let responses: Vec<GroupResponse> = groups.into_iter().map(|group| group.into()).collect();
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

#[tracing::instrument(skip_all)]
pub async fn update_group(
    State(state): State<AppState>,
    Extension(auth): Extension<UserAuth>,
    Path(group_id): Path<String>,
    Json(req): Json<UpdateGroupRequest>,
) -> AppResult<Response> {
    let requester_user_id = auth.user_id;

    let target_group: crate::models::Group = GroupRepository::find_by_id(&state.pool, &group_id)
        .await?
        .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    if !can_manage_target_user(&state.pool, &requester_user_id, &target_group.creator_id).await? {
        return Err(AppError::Forbidden("只有群创建者才能编辑该群".to_string()));
    }

    let next_name = req.name.trim();
    if next_name.is_empty() {
        return Err(AppError::BadRequest("群名称是必需的".to_string()));
    }

    let next_group_code = req
        .group_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(code) = next_group_code
        && let Some(existing_group_id) =
            GroupRepository::find_group_id_by_code(&state.pool, code).await?
        && existing_group_id != group_id
    {
        return Err(AppError::BadRequest("群号已存在".to_string()));
    }

    let next_description = req
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let next_avatar_url = req
        .avatar_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let next_is_public = req.is_public.unwrap_or(target_group.is_public);
    let now = chrono::Utc::now().to_rfc3339();

    GroupRepository::update_profile(
        &state.pool,
        &UpdateGroupProfile {
            group_id: &group_id,
            name: next_name,
            group_code: next_group_code,
            description: next_description,
            avatar_url: next_avatar_url,
            is_public: next_is_public,
            updated_at: &now,
        },
    )
    .await?;

    let updated_group: crate::models::Group = GroupRepository::find_by_id(&state.pool, &group_id)
        .await?
        .ok_or(AppError::BadRequest("Group not found".to_string()))?;

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &requester_user_id,
            user_id: Some(&requester_user_id),
            bot_id: None,
            group_id: Some(&group_id),
            action: "group.update",
            resource_type: "group",
            resource_id: Some(&group_id),
            details: Some(json!({ "name": next_name, "is_public": next_is_public })),
        },
    )
    .await;

    Ok(json_response(
        StatusCode::OK,
        GroupResponse::from(updated_group),
    ))
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

    // 查找群聊
    let group_id_str = if let Some(ref gid) = req.group_id {
        tracing::debug!("使用 group_id 查找群聊: {}", gid);
        gid.clone()
    } else {
        let gcode = req.group_code.as_ref().unwrap();
        tracing::debug!("使用 group_code 查找群聊: {}", gcode);

        let found_group: Option<String> =
            GroupRepository::find_group_id_by_code(&state.pool, gcode)
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

    let target_group: crate::models::Group =
        GroupRepository::find_by_id(&state.pool, &group_id_str)
            .await
            .map_err(|e| {
                tracing::error!("查询群聊时数据库错误: {}", e);
                e
            })?
            .ok_or_else(|| {
                tracing::warn!("群聊不存在: {}", group_id_str);
                AppError::BadRequest("群聊不存在".to_string())
            })?;

    let target_bot: crate::models::Bot = if let Some(bot_id) = req.bot_id.as_ref() {
        BotRepository::find_by_id(&state.pool, bot_id)
            .await
            .map_err(|e| {
                tracing::error!("查询目标 Bot 时数据库错误: {}", e);
                e
            })?
            .ok_or_else(|| {
                tracing::warn!("目标 Bot 不存在: {}", bot_id);
                AppError::BotNotFound
            })?
    } else {
        let default_bot_id: Option<String> =
            BotRepository::find_default_active_bot_id(&state.pool, &requester_user_id).await?;
        let bot_id = default_bot_id.ok_or(AppError::NoAvailableBot)?;
        BotRepository::find_by_id(&state.pool, &bot_id)
            .await
            .map_err(|e| {
                tracing::error!("查询默认 Bot 时数据库错误: {}", e);
                e
            })?
            .ok_or(AppError::BotNotFound)?
    };

    if target_bot.status != "active" {
        tracing::warn!("加入群聊失败: 目标 Bot 未激活: {}", target_bot.bot_id);
        return Err(AppError::BotInactive);
    }

    let target_bot_id = target_bot.bot_id.clone();
    let target_bot_owner_id = target_bot.owner_id.clone();

    let requester_can_manage_bot =
        can_manage_target_user(&state.pool, &requester_user_id, &target_bot_owner_id).await?;
    let requester_can_manage_group =
        can_manage_target_user(&state.pool, &requester_user_id, &target_group.creator_id).await?;

    if target_group.is_public {
        if !requester_can_manage_bot {
            return Err(AppError::Forbidden(
                "公开群仅允许 Bot 管理员发起入群".to_string(),
            ));
        }
        let joined_at = chrono::Utc::now().to_rfc3339();

        GroupRepository::add_member_ignore(
            &state.pool,
            &NewGroupMember {
                group_id: &group_id_str,
                member_id: &target_bot_id,
                member_type: "bot",
                joined_at: &joined_at,
            },
        )
        .await?;

        record_best_effort(
            &state.pool,
            AuditRecord {
                actor_type: "user",
                actor_id: &requester_user_id,
                user_id: Some(&requester_user_id),
                bot_id: Some(&target_bot_id),
                group_id: Some(&group_id_str),
                action: "group.join_public",
                resource_type: "group_member",
                resource_id: Some(&group_id_str),
                details: Some(json!({ "bot_id": target_bot_id })),
            },
        )
        .await;

        return Ok(json_response(
            StatusCode::OK,
            json!({
                "message": "公开群已直接加入",
                "group_id": group_id_str,
                "bot_id": target_bot_id,
                "result_status": "joined",
            }),
        ));
    }

    if !requester_can_manage_bot && !requester_can_manage_group {
        tracing::warn!(
            "加入群聊失败: 既不是 Bot 所有者也不是群创建者, bot_owner={}, group_creator={}, requester={}",
            target_bot_owner_id,
            target_group.creator_id,
            requester_user_id
        );
        return Err(AppError::Forbidden(
            "只能申请自己的 Bot 入群，或由群创建者发起邀请".to_string(),
        ));
    }

    let already_member = GroupRepository::is_member(
        &state.pool,
        &GroupMemberLink {
            group_id: &group_id_str,
            member_id: &target_bot_id,
        },
    )
    .await?;

    if already_member {
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "message": "Bot 已在群聊中",
                "group_id": group_id_str,
                "bot_id": target_bot_id,
                "result_status": "joined",
            }),
        ));
    }

    let now = chrono::Utc::now().to_rfc3339();

    if requester_can_manage_bot && requester_can_manage_group {
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
        record_best_effort(
            &state.pool,
            AuditRecord {
                actor_type: "user",
                actor_id: &requester_user_id,
                user_id: Some(&requester_user_id),
                bot_id: Some(&target_bot_id),
                group_id: Some(&group_id_str),
                action: "group.join_direct",
                resource_type: "group_member",
                resource_id: Some(&group_id_str),
                details: Some(json!({ "bot_id": target_bot_id })),
            },
        )
        .await;
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "message": "成功加入群聊",
                "group_id": group_id_str,
                "bot_id": target_bot_id,
                "result_status": "joined",
            }),
        ));
    }

    let (approver_user_id, request_type) =
        if requester_can_manage_group && !requester_can_manage_bot {
            (target_bot_owner_id.as_str(), "bot_owner_approval")
        } else {
            (target_group.creator_id.as_str(), "group_owner_approval")
        };

    let request_reason = req
        .request_reason
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or(AppError::BadRequest("申请理由不能为空".to_string()))?;

    if let Some(pending) = GroupJoinRequestRepository::find_pending(
        &state.pool,
        &group_id_str,
        &target_bot_id,
        approver_user_id,
        request_type,
    )
    .await?
    {
        return Ok(json_response(
            StatusCode::OK,
            json!({
                "message": "申请已存在，等待对方处理",
                "group_id": group_id_str,
                "bot_id": target_bot_id,
                "result_status": "pending_approval",
                "request_id": pending.request_id,
                "approver_user_id": approver_user_id,
                "request_type": request_type,
            }),
        ));
    }

    let request_id = generate_group_join_request_id();
    GroupJoinRequestRepository::insert(
        &state.pool,
        &NewGroupJoinRequest {
            request_id: &request_id,
            group_id: &group_id_str,
            bot_id: &target_bot_id,
            requester_user_id: &requester_user_id,
            approver_user_id,
            request_type,
            request_reason,
            now: &now,
        },
    )
    .await?;

    let requester_display_name = UserRepository::find_by_id(&state.pool, &requester_user_id)
        .await?
        .map(|user| user.name)
        .unwrap_or_else(|| requester_user_id.clone());

    create_notification_best_effort(
        &state.pool,
        NotificationRecord {
            recipient_user_id: approver_user_id,
            kind: "group_invite_approval",
            title: "群邀请审批",
            content: &format!(
                "{} 申请将 Bot {} 加入群 {}，理由：{}",
                requester_display_name, target_bot.name, target_group.name, request_reason
            ),
            requires_action: true,
            action_payload: Some(json!({
                "request_id": request_id,
                "group_id": group_id_str,
                "group_name": target_group.name,
                "bot_id": target_bot_id,
                "bot_name": target_bot.name,
                "requester_user_id": requester_user_id,
                "request_type": request_type,
                "request_reason": request_reason
            })),
            related_request_id: Some(&request_id),
            related_group_id: Some(&group_id_str),
            related_bot_id: Some(&target_bot_id),
        },
    )
    .await;

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &requester_user_id,
            user_id: Some(&requester_user_id),
            bot_id: Some(&target_bot_id),
            group_id: Some(&group_id_str),
            action: "group.join_request.create",
            resource_type: "group_join_request",
            resource_id: Some(&request_id),
            details: Some(
                json!({ "request_type": request_type, "approver_user_id": approver_user_id }),
            ),
        },
    )
    .await;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "message": "申请已提交，等待对方同意",
            "group_id": group_id_str,
            "bot_id": target_bot_id,
            "result_status": "pending_approval",
            "request_id": request_id,
            "approver_user_id": approver_user_id,
            "request_type": request_type,
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

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &requester_user_id,
            user_id: Some(&requester_user_id),
            bot_id: Some(&bot_id),
            group_id: Some(&group_id_str),
            action: "group.leave",
            resource_type: "group_member",
            resource_id: Some(&group_id_str),
            details: None,
        },
    )
    .await;

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
    let now = chrono::Utc::now().to_rfc3339();

    let target_bot: crate::models::Bot = BotRepository::find_by_id(&state.pool, &target_bot_id)
        .await?
        .ok_or(AppError::BotNotFound)?;
    let group: crate::models::Group = GroupRepository::find_by_id(&state.pool, &group_id_str)
        .await?
        .ok_or(AppError::BadRequest("Group not found".to_string()))?;

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

    if requester_user_id != target_bot.owner_id {
        create_notification_best_effort(
            &state.pool,
            NotificationRecord {
                recipient_user_id: &target_bot.owner_id,
                kind: "bot_removed_from_group",
                title: "Bot 已被移出群聊",
                content: &format!(
                    "Bot {} 已被用户 {} 移出群 {}",
                    target_bot.name, requester_user_id, group.name
                ),
                requires_action: false,
                action_payload: Some(json!({
                    "group_id": group_id_str,
                    "group_name": group.name,
                    "bot_id": target_bot.bot_id,
                    "bot_name": target_bot.name,
                    "operator_user_id": requester_user_id
                })),
                related_request_id: None,
                related_group_id: Some(&group_id_str),
                related_bot_id: Some(&target_bot_id),
            },
        )
        .await;
    }

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &requester_user_id,
            user_id: Some(&requester_user_id),
            bot_id: Some(&target_bot_id),
            group_id: Some(&group_id_str),
            action: "group.member.remove",
            resource_type: "group_member",
            resource_id: Some(&group_id_str),
            details: Some(json!({ "removed_at": now })),
        },
    )
    .await;

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

    GroupService::on_group_deleting(&state.pool, &group_id_str).await?;

    // 删除群消息，避免 groups 删除时触发 messages 的外键约束错误。
    GroupRepository::delete_messages_by_group(&state.pool, &group_id_str).await?;

    // Delete group members
    GroupRepository::delete_members_by_group(&state.pool, &group_id_str).await?;

    // Delete group
    GroupRepository::delete_group(&state.pool, &group_id_str).await?;

    tracing::info!("Group deleted: {}", group_id_str);

    record_best_effort(
        &state.pool,
        AuditRecord {
            actor_type: "user",
            actor_id: &requester_user_id,
            user_id: Some(&requester_user_id),
            bot_id: None,
            group_id: Some(&group_id_str),
            action: "group.delete",
            resource_type: "group",
            resource_id: Some(&group_id_str),
            details: None,
        },
    )
    .await;

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

    let messages = state
        .message_record_manager
        .get_group_messages(&group_id_str, base_id, limit)
        .await
        .map_err(|e| {
            tracing::error!("查询消息历史时错误: {}", e);
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

    let members: Vec<GroupMemberResponse> =
        GroupRepository::list_members(&state.pool, &group_id_str).await?;

    Ok(json_response(
        StatusCode::OK,
        json!({
            "members": members,
        }),
    ))
}
