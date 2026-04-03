use crate::models::ApiErrorResponse;
use thiserror::Error;

/// Common result type used by the SDK.
///
/// SDK 使用的统一结果类型。
pub type SdkResult<T> = Result<T, SdkError>;

/// Error type returned by the SDK.
///
/// SDK 返回的错误类型。
#[derive(Debug, Error)]
pub enum SdkError {
    #[error("请求构建失败: {0}")]
    RequestBuild(String),

    #[error("网络请求失败: {0}")]
    Transport(String),

    #[error("服务端返回错误: {code} - {message} (status={status})")]
    Api {
        code: String,
        message: String,
        status: u16,
    },

    #[error("HTTP 状态异常: status={status}, body={body}")]
    HttpStatus { status: u16, body: String },

    #[error("序列化或反序列化失败: {0}")]
    Serde(String),

    #[error("缺少用户 token，请先登录")]
    MissingUserToken,

    #[error("缺少 Bot token，请先选择或设置 Bot")]
    MissingBotToken,

    #[error("URL 无效: {0}")]
    InvalidUrl(String),

    #[cfg(feature = "ws")]
    #[error("WebSocket 错误: {0}")]
    WebSocket(String),
}

impl SdkError {
    /// Convert a structured backend API error into an SDK error.
    ///
    /// 将后端结构化 API 错误转换为 SDK 错误。
    pub fn from_api(value: ApiErrorResponse) -> Self {
        Self::Api {
            code: value.code,
            message: value.message,
            status: value.status,
        }
    }
}
