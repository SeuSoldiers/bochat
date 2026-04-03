use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::auth::AuthApi;
use crate::bots::BotsApi;
use crate::error::{SdkError, SdkResult};
use crate::files::FilesApi;
use crate::groups::GroupsApi;
use crate::messages::MessagesApi;
use crate::models::ApiErrorResponse;
use crate::retry::RetryPolicy;
#[cfg(feature = "ws")]
use crate::ws::WsSessionBuilder;

#[derive(Clone)]
pub struct BochatClient {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    base_url: String,
    http: reqwest::Client,
    user_token: RwLock<Option<String>>,
    bot_token: RwLock<Option<String>>,
    retry_policy: RetryPolicy,
}

#[derive(Clone, Copy)]
pub(crate) enum AuthKind {
    None,
    User,
    Bot,
}

pub struct BochatClientBuilder {
    base_url: String,
    timeout_secs: u64,
    user_agent: String,
    user_token: Option<String>,
    bot_token: Option<String>,
    retry_policy: RetryPolicy,
}

impl BochatClientBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            timeout_secs: 15,
            user_agent: "bochat-sdk/0.1.0".to_string(),
            user_token: None,
            bot_token: None,
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn user_token(mut self, token: impl Into<String>) -> Self {
        self.user_token = Some(token.into());
        self
    }

    pub fn bot_token(mut self, token: impl Into<String>) -> Self {
        self.bot_token = Some(token.into());
        self
    }

    pub fn retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    pub fn build(self) -> SdkResult<BochatClient> {
        let base_url = self.base_url.trim_end_matches('/').to_string();
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .user_agent(self.user_agent)
            .build()
            .map_err(|e| SdkError::RequestBuild(e.to_string()))?;

        Ok(BochatClient {
            inner: Arc::new(ClientInner {
                base_url,
                http,
                user_token: RwLock::new(self.user_token),
                bot_token: RwLock::new(self.bot_token),
                retry_policy: self.retry_policy,
            }),
        })
    }
}

impl BochatClient {
    pub fn builder(base_url: impl Into<String>) -> BochatClientBuilder {
        BochatClientBuilder::new(base_url)
    }

    pub fn auth(&self) -> AuthApi {
        AuthApi::new(self.clone())
    }

    pub fn bots(&self) -> BotsApi {
        BotsApi::new(self.clone())
    }

    pub fn messages(&self) -> MessagesApi {
        MessagesApi::new(self.clone())
    }

    pub fn groups(&self) -> GroupsApi {
        GroupsApi::new(self.clone())
    }

    pub fn files(&self) -> FilesApi {
        FilesApi::new(self.clone())
    }

    #[cfg(feature = "ws")]
    pub fn ws(&self) -> WsSessionBuilder {
        WsSessionBuilder::new(self.clone())
    }

    pub async fn set_user_token(&self, token: Option<String>) {
        *self.inner.user_token.write().await = token;
    }

    pub async fn set_bot_token(&self, token: Option<String>) {
        *self.inner.bot_token.write().await = token;
    }

    pub async fn user_token(&self) -> Option<String> {
        self.inner.user_token.read().await.clone()
    }

    pub async fn bot_token(&self) -> Option<String> {
        self.inner.bot_token.read().await.clone()
    }

    pub fn base_url(&self) -> &str {
        &self.inner.base_url
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        auth: AuthKind,
    ) -> SdkResult<T> {
        let url = self.path_to_url(path)?;
        let method = Method::GET;
        let mut last_err: Option<SdkError> = None;

        for attempt in 0..self.inner.retry_policy.max_attempts {
            let req = self
                .inner
                .http
                .request(method.clone(), &url)
                .headers(self.auth_headers(auth).await?);

            match req.send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return resp
                            .json::<T>()
                            .await
                            .map_err(|e| SdkError::Serde(e.to_string()));
                    }

                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();

                    if RetryPolicy::should_retry_method(&method)
                        && RetryPolicy::should_retry_status(status)
                        && attempt + 1 < self.inner.retry_policy.max_attempts
                    {
                        sleep(self.inner.retry_policy.next_delay(attempt)).await;
                        continue;
                    }

                    return Err(self.parse_http_error(status, text));
                }
                Err(err) => {
                    last_err = Some(SdkError::Transport(err.to_string()));
                    if attempt + 1 < self.inner.retry_policy.max_attempts {
                        sleep(self.inner.retry_policy.next_delay(attempt)).await;
                        continue;
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(|| SdkError::Transport("请求失败".to_string())))
    }

    pub(crate) async fn request_json<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        auth: AuthKind,
        body: &B,
    ) -> SdkResult<T> {
        let url = self.path_to_url(path)?;
        let req = self
            .inner
            .http
            .request(method, url)
            .headers(self.auth_headers(auth).await?)
            .json(body);

        let resp = req
            .send()
            .await
            .map_err(|e| SdkError::Transport(e.to_string()))?;
        self.parse_json_response::<T>(resp).await
    }

    pub(crate) async fn request_empty(
        &self,
        method: Method,
        path: &str,
        auth: AuthKind,
    ) -> SdkResult<()> {
        let url = self.path_to_url(path)?;
        let resp = self
            .inner
            .http
            .request(method, url)
            .headers(self.auth_headers(auth).await?)
            .send()
            .await
            .map_err(|e| SdkError::Transport(e.to_string()))?;

        if resp.status().is_success() {
            return Ok(());
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(self.parse_http_error(status, text))
    }

    pub(crate) async fn request_multipart<T: DeserializeOwned>(
        &self,
        path: &str,
        auth: AuthKind,
        form: reqwest::multipart::Form,
    ) -> SdkResult<T> {
        let url = self.path_to_url(path)?;
        let resp = self
            .inner
            .http
            .post(url)
            .headers(self.auth_headers(auth).await?)
            .multipart(form)
            .send()
            .await
            .map_err(|e| SdkError::Transport(e.to_string()))?;

        self.parse_json_response::<T>(resp).await
    }

    pub(crate) async fn parse_json_response<T: DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> SdkResult<T> {
        if resp.status().is_success() {
            return resp
                .json::<T>()
                .await
                .map_err(|e| SdkError::Serde(e.to_string()));
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(self.parse_http_error(status, text))
    }

    fn path_to_url(&self, path: &str) -> SdkResult<String> {
        if path.starts_with("http://") || path.starts_with("https://") {
            return Ok(path.to_string());
        }

        let normalized = if path.starts_with('/') {
            format!("{}{}", self.inner.base_url, path)
        } else {
            format!("{}/{}", self.inner.base_url, path)
        };

        url::Url::parse(&normalized).map_err(|e| SdkError::InvalidUrl(e.to_string()))?;
        Ok(normalized)
    }

    async fn auth_headers(&self, auth: AuthKind) -> SdkResult<HeaderMap> {
        let mut headers = HeaderMap::new();
        match auth {
            AuthKind::None => {}
            AuthKind::User => {
                let token = self
                    .inner
                    .user_token
                    .read()
                    .await
                    .clone()
                    .ok_or(SdkError::MissingUserToken)?;
                let value = HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| SdkError::RequestBuild(e.to_string()))?;
                headers.insert(AUTHORIZATION, value);
            }
            AuthKind::Bot => {
                let token = self
                    .inner
                    .bot_token
                    .read()
                    .await
                    .clone()
                    .ok_or(SdkError::MissingBotToken)?;
                let value = HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| SdkError::RequestBuild(e.to_string()))?;
                headers.insert(AUTHORIZATION, value);
            }
        }
        Ok(headers)
    }

    fn parse_http_error(&self, status: StatusCode, body: String) -> SdkError {
        if let Ok(api_err) = serde_json::from_str::<ApiErrorResponse>(&body) {
            return SdkError::from_api(api_err);
        }

        SdkError::HttpStatus {
            status: status.as_u16(),
            body,
        }
    }
}
