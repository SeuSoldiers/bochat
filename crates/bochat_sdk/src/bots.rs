use reqwest::Method;

use crate::client::{AuthKind, BochatClient};
use crate::error::{SdkError, SdkResult};
use crate::models::{BotInfo, BotListResponse, CreateBotRequest, UpdateBotRequest};

/// Bot management API facade.
///
/// Bot 管理 API 门面。
#[derive(Clone)]
pub struct BotsApi {
    client: BochatClient,
}

impl BotsApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    /// List bots visible to the current user token.
    ///
    /// 列出当前用户 token 可见的 Bot。
    pub async fn list(&self) -> SdkResult<Vec<BotInfo>> {
        let resp: BotListResponse = self.client.get_json("/api/v1/bots", AuthKind::User).await?;
        Ok(resp.bots)
    }

    /// Create a new bot owned by the current user.
    ///
    /// 为当前用户创建新的 Bot。
    pub async fn create(&self, req: CreateBotRequest) -> SdkResult<BotInfo> {
        self.client
            .request_json(Method::POST, "/api/v1/bots", AuthKind::User, &req)
            .await
    }

    /// Fetch a bot by ID using the public detail endpoint.
    ///
    /// 通过公开详情接口按 ID 获取 Bot。
    pub async fn get(&self, bot_id: &str) -> SdkResult<BotInfo> {
        self.client
            .get_json(&format!("/api/v1/bots/{}", bot_id), AuthKind::None)
            .await
    }

    /// Update a bot owned by the current user.
    ///
    /// 更新当前用户名下的 Bot。
    pub async fn update(&self, bot_id: &str, req: UpdateBotRequest) -> SdkResult<BotInfo> {
        self.client
            .request_json(
                Method::PUT,
                &format!("/api/v1/bots/{}", bot_id),
                AuthKind::User,
                &req,
            )
            .await
    }

    /// Delete a bot owned by the current user.
    ///
    /// 删除当前用户名下的 Bot。
    pub async fn delete(&self, bot_id: &str) -> SdkResult<()> {
        self.client
            .request_empty(
                Method::DELETE,
                &format!("/api/v1/bots/{}", bot_id),
                AuthKind::User,
            )
            .await
    }

    /// Select a bot from the current bot list and store its token on the client.
    ///
    /// 从当前 Bot 列表中选中一个 Bot，并把它的 token 写回客户端。
    ///
    /// If `bot_id` is `None`, the first active bot is selected.
    ///
    /// 若 `bot_id` 为 `None`，则自动选择第一个处于 `active` 状态的 Bot。
    pub async fn use_bot_token(&self, bot_id: Option<&str>) -> SdkResult<String> {
        let bots = self.list().await?;
        let chosen = if let Some(target) = bot_id {
            bots.into_iter().find(|b| b.bot_id == target)
        } else {
            bots.into_iter().find(|b| b.status == "active")
        }
        .ok_or_else(|| SdkError::Api {
            code: "no_available_bot".to_string(),
            message: "没有可用的 Bot 可设置 token".to_string(),
            status: 400,
        })?;

        self.client.set_bot_token(Some(chosen.token.clone())).await;
        Ok(chosen.token)
    }
}
