use reqwest::Method;

use crate::client::{AuthKind, BochatClient};
use crate::error::SdkResult;
use crate::models::{CreateGroupRequest, GroupInfo};

#[derive(Clone)]
pub struct GroupsApi {
    client: BochatClient,
}

impl GroupsApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, req: CreateGroupRequest) -> SdkResult<GroupInfo> {
        self.client
            .request_json(Method::POST, "/api/v1/groups", AuthKind::User, &req)
            .await
    }

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
