use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("User not found")]
    UserNotFound,

    #[error("Bot not found")]
    BotNotFound,

    #[error("Invalid ID number")]
    InvalidIdNumber,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Token invalid or expired")]
    InvalidToken,

    #[error("User token required")]
    UserTokenRequired,

    #[error("Bot token required")]
    BotTokenRequired,

    #[error("Invalid user token")]
    InvalidUserToken,

    #[error("Invalid bot token")]
    InvalidBotToken,

    #[error("ID number already exists")]
    IdNumberConflict,

    #[error("Account already exists")]
    AccountConflict,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Message not found")]
    MessageNotFound,

    #[error("File not found")]
    FileNotFound,

    #[error("Invalid file format")]
    InvalidFileFormat,

    #[error("File too large")]
    FileTooLarge,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Redis error: {0}")]
    RedisError(String),

    #[error("Bot permission denied")]
    BotPermissionDenied,

    #[error("Bot ownership mismatch")]
    BotOwnershipMismatch,

    #[error("Bot inactive")]
    BotInactive,

    #[error("No available bot")]
    NoAvailableBot,

    #[error("Bot is not a member of this group")]
    BotNotInGroup,

    #[error("{0}")]
    Forbidden(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::UserNotFound
            | AppError::BotNotFound
            | AppError::MessageNotFound
            | AppError::FileNotFound => StatusCode::NOT_FOUND,
            AppError::InvalidIdNumber
            | AppError::InvalidToken
            | AppError::InvalidUserToken
            | AppError::InvalidBotToken
            | AppError::BotInactive
            | AppError::NoAvailableBot
            | AppError::BotNotInGroup => StatusCode::BAD_REQUEST,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::Unauthorized
            | AppError::UserTokenRequired
            | AppError::BotTokenRequired
            | AppError::BotPermissionDenied
            | AppError::BotOwnershipMismatch
            | AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::IdNumberConflict | AppError::AccountConflict => StatusCode::CONFLICT,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::BadRequest(_) | AppError::InvalidFileFormat => StatusCode::BAD_REQUEST,
            AppError::FileTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            AppError::DatabaseError(_) => "database_error",
            AppError::UserNotFound => "user_not_found",
            AppError::BotNotFound => "bot_not_found",
            AppError::InvalidIdNumber => "invalid_id_number",
            AppError::Unauthorized => "unauthorized",
            AppError::InvalidToken => "invalid_token",
            AppError::UserTokenRequired => "user_token_required",
            AppError::BotTokenRequired => "bot_token_required",
            AppError::InvalidUserToken => "invalid_user_token",
            AppError::InvalidBotToken => "invalid_bot_token",
            AppError::IdNumberConflict => "id_number_conflict",
            AppError::AccountConflict => "account_conflict",
            AppError::InvalidCredentials => "invalid_credentials",
            AppError::MessageNotFound => "message_not_found",
            AppError::FileNotFound => "file_not_found",
            AppError::InvalidFileFormat => "invalid_file_format",
            AppError::FileTooLarge => "file_too_large",
            AppError::RateLimitExceeded => "rate_limit_exceeded",
            AppError::BadRequest(_) => "bad_request",
            AppError::InternalError(_) => "internal_error",
            AppError::RedisError(_) => "redis_error",
            AppError::BotPermissionDenied => "bot_permission_denied",
            AppError::BotOwnershipMismatch => "bot_ownership_mismatch",
            AppError::BotInactive => "bot_inactive",
            AppError::NoAvailableBot => "no_available_bot",
            AppError::BotNotInGroup => "bot_not_in_group",
            AppError::Forbidden(_) => "forbidden",
        }
    }

    fn user_message(&self) -> String {
        match self {
            AppError::DatabaseError(e) => {
                error!("Database error: {}", e);
                "数据库操作失败，请稍后重试".to_string()
            }
            AppError::UserNotFound => "用户不存在或登录信息不正确".to_string(),
            AppError::BotNotFound => "Bot 不存在".to_string(),
            AppError::InvalidIdNumber => "身份证号格式不正确".to_string(),
            AppError::Unauthorized => "未授权访问".to_string(),
            AppError::InvalidToken => "登录状态已失效，请重新登录".to_string(),
            AppError::UserTokenRequired => "该操作需要用户登录后再执行".to_string(),
            AppError::BotTokenRequired => "该操作需要选择一个可用的 Bot".to_string(),
            AppError::InvalidUserToken => "用户登录已过期，请重新登录".to_string(),
            AppError::InvalidBotToken => "Bot 凭证已失效，请重新选择 Bot".to_string(),
            AppError::IdNumberConflict => "该身份证号已注册".to_string(),
            AppError::AccountConflict => "该账号已注册".to_string(),
            AppError::InvalidCredentials => "账号或密码错误".to_string(),
            AppError::MessageNotFound => "消息不存在".to_string(),
            AppError::FileNotFound => "文件不存在".to_string(),
            AppError::InvalidFileFormat => "文件格式不支持".to_string(),
            AppError::FileTooLarge => "文件过大，请上传更小的文件".to_string(),
            AppError::RateLimitExceeded => "请求过于频繁，请稍后再试".to_string(),
            AppError::BadRequest(message) => message.clone(),
            AppError::InternalError(_) => "服务器内部错误，请稍后重试".to_string(),
            AppError::RedisError(_) => "系统暂时不可用，请稍后重试".to_string(),
            AppError::BotPermissionDenied => "没有权限执行该 Bot 操作".to_string(),
            AppError::BotOwnershipMismatch => "只能操作自己名下的 Bot".to_string(),
            AppError::BotInactive => "目标 Bot 未激活".to_string(),
            AppError::NoAvailableBot => "当前用户没有可用的 Bot".to_string(),
            AppError::BotNotInGroup => "该 Bot 还不在当前群里".to_string(),
            AppError::Forbidden(message) => message.clone(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code();
        let message = self.user_message();
        (
            status,
            Json(json!({
                "code": code,
                "message": message,
                "status": status.as_u16(),
            })),
        )
            .into_response()
    }
}

pub fn json_response<T: Serialize>(status: StatusCode, payload: T) -> Response {
    (status, Json(payload)).into_response()
}

pub fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(json!({
            "message": message.into(),
            "status": status.as_u16(),
        })),
    )
        .into_response()
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
