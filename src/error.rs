use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

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

    #[error("ID number already exists")]
    IdNumberConflict,

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
            AppError::InvalidIdNumber | AppError::InvalidToken => StatusCode::BAD_REQUEST,
            AppError::Unauthorized | AppError::BotPermissionDenied | AppError::Forbidden(_) => {
                StatusCode::FORBIDDEN
            }
            AppError::IdNumberConflict => StatusCode::CONFLICT,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::BadRequest(_) | AppError::InvalidFileFormat => StatusCode::BAD_REQUEST,
            AppError::FileTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        (
            status,
            Json(json!({
                "error": self.to_string(),
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
            "error": message.into(),
            "status": status.as_u16(),
        })),
    )
        .into_response()
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
