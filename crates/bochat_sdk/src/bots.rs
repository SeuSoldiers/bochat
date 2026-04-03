use reqwest::Method;

use crate::client::{AuthKind, BochatClient};
use crate::error::{SdkError, SdkResult};
use crate::models::{BotInfo, BotListResponse, CreateBotRequest, UpdateBotRequest};

#[derive(Clone)]
pub struct BotsApi {
    client: BochatClient,
}

impl BotsApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> SdkResult<Vec<BotInfo>> {
        let resp: BotListResponse = self.client.get_json("/api/v1/bots", AuthKind::User).await?;
        Ok(resp.bots)
    }

    pub async fn create(&self, req: CreateBotRequest) -> SdkResult<BotInfo> {
        self.client
            .request_json(Method::POST, "/api/v1/bots", AuthKind::User, &req)
            .await
    }

    pub async fn get(&self, bot_id: &str) -> SdkResult<BotInfo> {
        self.client
            .get_json(&format!("/api/v1/bots/{}", bot_id), AuthKind::None)
            .await
    }

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

    pub async fn delete(&self, bot_id: &str) -> SdkResult<()> {
        self.client
            .request_empty(
                Method::DELETE,
                &format!("/api/v1/bots/{}", bot_id),
                AuthKind::User,
            )
            .await
    }

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
