use axum::http::HeaderMap;

use crate::error::{AppError, AppResult};

pub fn bearer_token(headers: &HeaderMap) -> AppResult<String> {
    headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(ToOwned::to_owned)
        .ok_or(AppError::Unauthorized)
}

pub fn token_bot_id(token: &str) -> AppResult<&str> {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }

    Ok(parts[0])
}
