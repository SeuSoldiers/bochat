use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
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

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Token invalid or expired")]
    InvalidToken,

    #[error("Username already exists")]
    UsernameConflict,

    #[error("Email already exists")]
    EmailConflict,

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
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::UserNotFound
            | AppError::BotNotFound
            | AppError::MessageNotFound
            | AppError::FileNotFound => StatusCode::NOT_FOUND,
            AppError::InvalidCredentials | AppError::InvalidToken => StatusCode::UNAUTHORIZED,
            AppError::Unauthorized => StatusCode::FORBIDDEN,
            AppError::UsernameConflict | AppError::EmailConflict => StatusCode::CONFLICT,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::BadRequest(_) | AppError::InvalidFileFormat => StatusCode::BAD_REQUEST,
            AppError::FileTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        HttpResponse::build(status).json(json!({
            "error": self.to_string(),
            "status": status.as_u16(),
        }))
    }
}

// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
