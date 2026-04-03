use reqwest::Method;

use crate::client::{AuthKind, BochatClient};
use crate::error::SdkResult;
use crate::models::{
    AuthResponse, LoginRequest, RegisterRequest, UpdateProfileRequest, UserProfile,
};

#[derive(Clone)]
pub struct AuthApi {
    client: BochatClient,
}

impl AuthApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    pub fn register(&self) -> RegisterBuilder {
        RegisterBuilder {
            client: self.client.clone(),
            account: None,
            password: None,
            nickname: None,
        }
    }

    pub fn login(&self) -> LoginBuilder {
        LoginBuilder {
            client: self.client.clone(),
            account: None,
            password: None,
        }
    }

    pub async fn me(&self) -> SdkResult<UserProfile> {
        self.client
            .get_json("/api/v1/users/me", AuthKind::User)
            .await
    }

    pub async fn update_profile(&self, req: UpdateProfileRequest) -> SdkResult<UserProfile> {
        self.client
            .request_json(Method::PUT, "/api/v1/users/me", AuthKind::User, &req)
            .await
    }

    pub async fn delete_account(&self) -> SdkResult<()> {
        self.client
            .request_empty(Method::DELETE, "/api/v1/users/delete", AuthKind::User)
            .await
    }
}

pub struct RegisterBuilder {
    client: BochatClient,
    account: Option<String>,
    password: Option<String>,
    nickname: Option<String>,
}

impl RegisterBuilder {
    pub fn account(mut self, account: impl Into<String>) -> Self {
        self.account = Some(account.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nickname = Some(nickname.into());
        self
    }

    pub async fn send(self) -> SdkResult<AuthResponse> {
        let req = RegisterRequest {
            account: self.account.unwrap_or_default(),
            password: self.password.unwrap_or_default(),
            name: self.nickname,
        };

        let resp: AuthResponse = self
            .client
            .request_json(Method::POST, "/api/v1/auth/register", AuthKind::None, &req)
            .await?;
        self.client.set_user_token(Some(resp.token.clone())).await;
        Ok(resp)
    }
}

pub struct LoginBuilder {
    client: BochatClient,
    account: Option<String>,
    password: Option<String>,
}

impl LoginBuilder {
    pub fn account(mut self, account: impl Into<String>) -> Self {
        self.account = Some(account.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub async fn send(self) -> SdkResult<AuthResponse> {
        let req = LoginRequest {
            account: self.account.unwrap_or_default(),
            password: self.password.unwrap_or_default(),
        };

        let resp: AuthResponse = self
            .client
            .request_json(Method::POST, "/api/v1/auth/login", AuthKind::None, &req)
            .await?;
        self.client.set_user_token(Some(resp.token.clone())).await;
        Ok(resp)
    }
}
