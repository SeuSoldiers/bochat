use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPayload {
    pub bot_id: String,
    pub timestamp: i64,
}

pub fn generate_token(bot_id: &str, secret: &str) -> AppResult<String> {
    let timestamp = Utc::now().timestamp();
    let payload = format!("{}:{}", bot_id, timestamp);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::InternalError("Failed to create HMAC".to_string()))?;

    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    Ok(format!("{}:{}", payload, signature))
}

pub fn verify_token(token: &str, secret: &str, max_age_secs: u64) -> AppResult<TokenPayload> {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }

    let bot_id = parts[0];
    let timestamp_str = parts[1];
    let signature = parts[2];

    let timestamp = timestamp_str
        .parse::<i64>()
        .map_err(|_| AppError::InvalidToken)?;

    // Check token age
    let now = Utc::now().timestamp();
    if now - timestamp > max_age_secs as i64 {
        return Err(AppError::InvalidToken);
    }

    // Verify signature
    let payload = format!("{}:{}", bot_id, timestamp);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::InternalError("Failed to create HMAC".to_string()))?;

    mac.update(payload.as_bytes());
    let expected_signature = hex::encode(mac.finalize().into_bytes());

    if signature != expected_signature {
        return Err(AppError::InvalidToken);
    }

    Ok(TokenPayload {
        bot_id: bot_id.to_string(),
        timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_verification() {
        let secret = "test-secret";
        let bot_id = "b_test123";

        let token = generate_token(bot_id, secret).expect("Failed to generate token");
        let payload = verify_token(&token, secret, 3600).expect("Failed to verify token");

        assert_eq!(payload.bot_id, bot_id);
    }

    #[test]
    fn test_token_verification_fails_with_wrong_secret() {
        let secret = "test-secret";
        let bot_id = "b_test123";

        let token = generate_token(bot_id, secret).expect("Failed to generate token");
        let result = verify_token(&token, "wrong-secret", 3600);

        assert!(result.is_err());
    }
}
