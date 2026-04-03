use reqwest::Method;

use crate::client::{AuthKind, BochatClient};
use crate::error::SdkResult;
use crate::models::{CreateGroupRequest, GroupInfo};

/// Group management API facade.
///
/// 群管理 API 门面。
#[derive(Clone)]
pub struct GroupsApi {
    client: BochatClient,
}

impl GroupsApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    /// Create a group with the current user token.
    ///
    /// 使用当前用户 token 创建群聊。
    ///
    /// The backend can auto-select one of the user's active bots if `bot_id` is
    /// omitted in the request.
    ///
    /// 如果请求中省略 `bot_id`，后端会自动选择当前用户的一个活跃 Bot 入群。
    pub async fn create(&self, req: CreateGroupRequest) -> SdkResult<GroupInfo> {
        self.client
            .request_json(Method::POST, "/api/v1/groups", AuthKind::User, &req)
            .await
    }

    /// Delete a group.
    ///
    /// 删除群聊。
    pub async fn delete(&self, group_id: &str) -> SdkResult<()> {
        self.client
            .request_empty(
                Method::DELETE,
                &format!("/api/v1/groups/{}", group_id),
                AuthKind::User,
            )
            .await
    }
}
