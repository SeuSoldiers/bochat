use reqwest::Method;

use crate::client::{AuthKind, BochatClient};
use crate::error::SdkResult;
use crate::models::{
    AuthResponse, LoginRequest, RegisterRequest, UpdateProfileRequest, UserProfile,
};

/// Authentication and user-profile API facade.
///
/// 认证与用户资料 API 门面。
#[derive(Clone)]
pub struct AuthApi {
    client: BochatClient,
}

impl AuthApi {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self { client }
    }

    /// Start a register request builder.
    ///
    /// 开始构建注册请求。
    pub fn register(&self) -> RegisterBuilder {
        RegisterBuilder {
            client: self.client.clone(),
            account: None,
            password: None,
            nickname: None,
        }
    }

    /// Start a login request builder.
    ///
    /// 开始构建登录请求。
    pub fn login(&self) -> LoginBuilder {
        LoginBuilder {
            client: self.client.clone(),
            account: None,
            password: None,
        }
    }

    /// Fetch the profile of the current authenticated user.
    ///
    /// 获取当前已认证用户的资料。
    pub async fn me(&self) -> SdkResult<UserProfile> {
        self.client
            .get_json("/api/v1/users/me", AuthKind::User)
            .await
    }

    /// Update the current user's profile.
    ///
    /// 更新当前用户资料。
    pub async fn update_profile(&self, req: UpdateProfileRequest) -> SdkResult<UserProfile> {
        self.client
            .request_json(Method::PUT, "/api/v1/users/me", AuthKind::User, &req)
            .await
    }

    /// Delete the current user account and all owned bots.
    ///
    /// 删除当前用户账号及其名下全部 Bot。
    pub async fn delete_account(&self) -> SdkResult<()> {
        self.client
            .request_empty(Method::DELETE, "/api/v1/users/delete", AuthKind::User)
            .await
    }
}

/// Builder for `POST /api/v1/auth/register`.
///
/// `POST /api/v1/auth/register` 的请求构建器。
///
/// On success, the returned user token is automatically stored in the client.
///
/// 注册成功后，返回的用户 token 会自动写回客户端。
pub struct RegisterBuilder {
    client: BochatClient,
    account: Option<String>,
    password: Option<String>,
    nickname: Option<String>,
}

impl RegisterBuilder {
    /// Set the login account.
    ///
    /// 设置登录账号。
    pub fn account(mut self, account: impl Into<String>) -> Self {
        self.account = Some(account.into());
        self
    }

    /// Set the plaintext password.
    ///
    /// 设置明文密码。
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the optional nickname shown in the platform.
    ///
    /// 设置平台中展示的可选昵称。
    pub fn nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nickname = Some(nickname.into());
        self
    }

    /// Send the register request.
    ///
    /// 发送注册请求。
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

/// Builder for `POST /api/v1/auth/login`.
///
/// `POST /api/v1/auth/login` 的请求构建器。
///
/// On success, the returned user token is automatically stored in the client.
///
/// 登录成功后，返回的用户 token 会自动写回客户端。
pub struct LoginBuilder {
    client: BochatClient,
    account: Option<String>,
    password: Option<String>,
}

impl LoginBuilder {
    /// Set the login account.
    ///
    /// 设置登录账号。
    pub fn account(mut self, account: impl Into<String>) -> Self {
        self.account = Some(account.into());
        self
    }

    /// Set the plaintext password.
    ///
    /// 设置明文密码。
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Send the login request.
    ///
    /// 发送登录请求。
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
