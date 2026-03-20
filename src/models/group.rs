use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Group {
    pub group_id: String,
    pub group_code: Option<String>, // 可选的群号，用户可以输入这个号码加入群聊
    pub creator_id: String, // 创建者的User ID
    pub name: String,
    pub description: Option<String>,
    pub status: String, // active / inactive
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub group_code: Option<String>, // 可选，如果不提供则不设置
    pub bot_id: Option<String>, // 可选，指定自动加入群聊的 Bot
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupResponse {
    pub group_id: String,
    pub group_code: Option<String>,
    pub creator_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Group> for GroupResponse {
    fn from(group: Group) -> Self {
        GroupResponse {
            group_id: group.group_id,
            group_code: group.group_code,
            creator_id: group.creator_id,
            name: group.name,
            description: group.description,
            status: group.status,
            created_at: group.created_at,
            updated_at: group.updated_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupMember {
    pub group_id: String,
    pub member_id: String,   // Bot ID
    pub member_type: String, // bot
    pub joined_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupMemberResponse {
    pub group_id: String,
    pub member_id: String,
    pub member_type: String,
    pub joined_at: String,
    pub bot_name: Option<String>,
    pub owner_id: Option<String>,
}

impl From<GroupMember> for GroupMemberResponse {
    fn from(member: GroupMember) -> Self {
        GroupMemberResponse {
            group_id: member.group_id,
            member_id: member.member_id,
            member_type: member.member_type,
            joined_at: member.joined_at,
            bot_name: None,
            owner_id: None,
        }
    }
}

/// 加入群聊请求
///
/// 用户可以通过群ID或群号加入群聊
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinGroupRequest {
    /// 群聊ID（group_id）- 优先使用这个
    pub group_id: Option<String>,
    /// 群号（group_code）- 如果没有 group_id，可以用这个
    pub group_code: Option<String>,
    /// 要加入群聊的 Bot ID，必须属于当前认证用户
    pub bot_id: Option<String>,
}
