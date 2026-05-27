use chat_platform::utils::token::{
    generate_token, generate_user_token, verify_token, verify_user_token,
};
use std::time::Duration;

#[test]
fn test_token_generation_and_verification() {
    let secret = "test-secret";
    let bot_id = "b_test123";

    let token = generate_token(bot_id, secret).expect("Failed to generate token");
    println!("Generated token: {}", token);

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

#[test]
fn test_token_format() {
    let secret = "test-secret";
    let bot_id = "b_test123";

    let token = generate_token(bot_id, secret).expect("Failed to generate token");

    // Token should have 3 parts separated by colons
    let parts: Vec<&str> = token.split(':').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], bot_id);

    // Middle part should be a valid timestamp
    let _timestamp = parts[1]
        .parse::<i64>()
        .expect("Timestamp should be a valid i64");

    // Last part should be hex string (signature)
    assert!(parts[2].chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_token_verification_fails_with_invalid_format() {
    let secret = "test-secret";
    let result = verify_token("invalid-token-without-colons", secret, 3600);
    assert!(result.is_err());
}

#[test]
fn test_token_verification_fails_with_invalid_timestamp() {
    let secret = "test-secret";
    let bot_id = "b_test123";
    let token = format!("{bot_id}:not-a-timestamp:deadbeef");
    let result = verify_token(&token, secret, 3600);
    assert!(result.is_err());
}

#[test]
fn test_token_verification_fails_when_expired() {
    let secret = "test-secret";
    let bot_id = "b_test123";
    let token = generate_token(bot_id, secret).expect("Failed to generate token");
    std::thread::sleep(Duration::from_secs(2));
    let result = verify_token(&token, secret, 0);
    assert!(result.is_err());
}

#[test]
fn test_user_token_generation_and_verification() {
    let secret = "test-secret";
    let user_id = "u_test123";
    let token = generate_user_token(user_id, secret).expect("Failed to generate user token");
    let payload = verify_user_token(&token, secret, 3600).expect("Failed to verify user token");
    assert_eq!(payload.user_id, user_id);
}

#[test]
fn test_user_token_verification_fails_with_invalid_prefix() {
    let secret = "test-secret";
    let token = "x:u_test:123:deadbeef";
    let result = verify_user_token(token, secret, 3600);
    assert!(result.is_err());
}

#[test]
fn test_user_token_verification_fails_with_wrong_secret() {
    let secret = "test-secret";
    let user_id = "u_test123";
    let token = generate_user_token(user_id, secret).expect("Failed to generate user token");
    let result = verify_user_token(&token, "wrong-secret", 3600);
    assert!(result.is_err());
}
