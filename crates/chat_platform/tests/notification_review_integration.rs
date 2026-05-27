use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use chat_platform::{
    AppState, app_router,
    cache::RedisMessageCache,
    config::{
        Config, DatabaseConfig, LoggingConfig, RedisConfig, SecurityConfig, ServerConfig,
        StorageConfig,
    },
    db,
    services::message_record_manager::MessageRecordManager,
    ws::WsManager,
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::util::ServiceExt;

fn test_config(temp_dir: &TempDir) -> Config {
    let file_storage_path = temp_dir.path().join("files");

    Config {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            workers: 1,
        },
        database: DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://chat_user:chat_password@127.0.0.1:50032/chat_db".to_string()
            }),
            max_connections: 2,
            min_connections: 1,
        },
        redis: RedisConfig {
            url: std::env::var("TEST_REDIS_URL")
                .unwrap_or_else(|_| "redis://:redis_password@127.0.0.1:50079/0".to_string()),
        },
        security: SecurityConfig {
            jwt_secret: "test-secret".to_string(),
            token_expiry_secs: 86_400,
            max_file_size_mb: 10,
            rate_limit_per_second: 100,
        },
        storage: StorageConfig {
            file_storage_path: file_storage_path.to_string_lossy().to_string(),
        },
        logging: LoggingConfig {
            dir: temp_dir.path().join("logs").to_string_lossy().to_string(),
            file_prefix: "chat_platform_test".to_string(),
        },
    }
}

fn body_str<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key]
        .as_str()
        .unwrap_or_else(|| panic!("missing string field `{key}` in {value}"))
}

async fn send_json(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    payload: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    let body = payload
        .map(|value| Body::from(value.to_string()))
        .unwrap_or_else(Body::empty);

    let response = app
        .clone()
        .oneshot(builder.body(body).expect("build request"))
        .await
        .expect("request should succeed");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let value = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));
    (status, value)
}

async fn setup_app() -> (Router, TempDir) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config = test_config(&temp_dir);
    let pool = db::init_pool(&config.database).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let redis_client =
        redis::Client::open(config.redis.url.as_str()).expect("invalid Redis URL in test");
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("connect Redis in test");
    let redis_cache = RedisMessageCache::new(redis_conn);
    let message_record_manager = MessageRecordManager::new(pool.clone(), redis_cache)
        .await
        .expect("init message record manager");

    let app = app_router(AppState {
        config,
        pool,
        ws_manager: WsManager::new(),
        message_record_manager,
    });
    (app, temp_dir)
}

async fn register_and_login(app: &Router, account: &str, name: &str) -> String {
    let (register_status, _) = send_json(
        app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({
            "name": name,
            "account": account,
            "password": "Pass2026!"
        })),
    )
    .await;
    assert_eq!(register_status, StatusCode::CREATED);

    let (login_status, login_body) = send_json(
        app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({
            "account": account,
            "password": "Pass2026!"
        })),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK);
    body_str(&login_body, "token").to_string()
}

async fn user_default_bot_id(app: &Router, user_token: &str) -> String {
    let (bots_status, bots_body) = send_json(app, "GET", "/api/v1/bots", Some(user_token), None).await;
    assert_eq!(bots_status, StatusCode::OK);
    body_str(&bots_body["bots"][0], "bot_id").to_string()
}

#[tokio::test]
async fn notification_approve_join_request_flow_works() {
    let (app, _tmp) = setup_app().await;
    let unique = chrono::Utc::now().timestamp_millis();
    let alice_token = register_and_login(&app, &format!("alice_n_{}", unique), "Alice").await;
    let bob_token = register_and_login(&app, &format!("bob_n_{}", unique), "Bob").await;

    let bob_bot_id = user_default_bot_id(&app, &bob_token).await;
    let group_code = format!("NAPP{}", unique);

    let (create_group_status, create_group_body) = send_json(
        &app,
        "POST",
        "/api/v1/groups",
        Some(&alice_token),
        Some(json!({
            "name": "Need Approval Group",
            "group_code": group_code,
            "is_public": false
        })),
    )
    .await;
    assert_eq!(create_group_status, StatusCode::CREATED);
    let group_id = body_str(&create_group_body, "group_id").to_string();

    let (join_status, join_body) = send_json(
        &app,
        "POST",
        "/api/v1/groups/join",
        Some(&bob_token),
        Some(json!({
            "group_id": group_id,
            "bot_id": bob_bot_id,
            "request_reason": "please let me join"
        })),
    )
    .await;
    assert_eq!(join_status, StatusCode::OK);
    assert_eq!(join_body["result_status"], "pending_approval");

    let (list_status, list_body) = send_json(
        &app,
        "GET",
        "/api/v1/notifications?status=pending&kind=group_invite_approval",
        Some(&alice_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let notifications = list_body["notifications"].as_array().expect("notifications array");
    let notification = notifications
        .iter()
        .find(|n| n["kind"] == "group_invite_approval")
        .expect("approval notification exists");
    let notification_id = body_str(notification, "notification_id").to_string();

    let (approve_status, _) = send_json(
        &app,
        "POST",
        &format!("/api/v1/notifications/{notification_id}/approve"),
        Some(&alice_token),
        Some(json!({ "note": "approved in test" })),
    )
    .await;
    assert_eq!(approve_status, StatusCode::OK);

    let (members_status, members_body) = send_json(
        &app,
        "GET",
        &format!("/api/v1/groups/{group_id}/members"),
        Some(&alice_token),
        None,
    )
    .await;
    assert_eq!(members_status, StatusCode::OK);
    let members = members_body["members"].as_array().expect("members array");
    assert!(members.iter().any(|m| m["member_id"] == bob_bot_id));

    let (bob_notice_status, bob_notice_body) = send_json(
        &app,
        "GET",
        "/api/v1/notifications?kind=group_invite_result",
        Some(&bob_token),
        None,
    )
    .await;
    assert_eq!(bob_notice_status, StatusCode::OK);
    let bob_notifications = bob_notice_body["notifications"]
        .as_array()
        .expect("bob notifications array");
    let result_notice = bob_notifications
        .iter()
        .find(|n| n["kind"] == "group_invite_result")
        .expect("result notification exists");
    assert_eq!(result_notice["action_payload"]["status"], "approved");
}

#[tokio::test]
async fn notification_reject_join_request_flow_works() {
    let (app, _tmp) = setup_app().await;
    let unique = chrono::Utc::now().timestamp_millis();
    let alice_token = register_and_login(&app, &format!("alice_r_{}", unique), "Alice").await;
    let bob_token = register_and_login(&app, &format!("bob_r_{}", unique), "Bob").await;

    let bob_bot_id = user_default_bot_id(&app, &bob_token).await;
    let group_code = format!("NREJ{}", unique);

    let (create_group_status, create_group_body) = send_json(
        &app,
        "POST",
        "/api/v1/groups",
        Some(&alice_token),
        Some(json!({
            "name": "Reject Group",
            "group_code": group_code,
            "is_public": false
        })),
    )
    .await;
    assert_eq!(create_group_status, StatusCode::CREATED);
    let group_id = body_str(&create_group_body, "group_id").to_string();

    let (join_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/groups/join",
        Some(&bob_token),
        Some(json!({
            "group_id": group_id,
            "bot_id": bob_bot_id,
            "request_reason": "reject me in test"
        })),
    )
    .await;
    assert_eq!(join_status, StatusCode::OK);

    let (list_status, list_body) = send_json(
        &app,
        "GET",
        "/api/v1/notifications?status=pending&kind=group_invite_approval",
        Some(&alice_token),
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let notifications = list_body["notifications"].as_array().expect("notifications array");
    let notification = notifications
        .iter()
        .find(|n| n["kind"] == "group_invite_approval")
        .expect("approval notification exists");
    let notification_id = body_str(notification, "notification_id").to_string();

    let (reject_status, _) = send_json(
        &app,
        "POST",
        &format!("/api/v1/notifications/{notification_id}/reject"),
        Some(&alice_token),
        Some(json!({ "note": "rejected in test" })),
    )
    .await;
    assert_eq!(reject_status, StatusCode::OK);

    let (members_status, members_body) = send_json(
        &app,
        "GET",
        &format!("/api/v1/groups/{group_id}/members"),
        Some(&alice_token),
        None,
    )
    .await;
    assert_eq!(members_status, StatusCode::OK);
    let members = members_body["members"].as_array().expect("members array");
    assert!(!members.iter().any(|m| m["member_id"] == bob_bot_id));

    let (bob_notice_status, bob_notice_body) = send_json(
        &app,
        "GET",
        "/api/v1/notifications?kind=group_invite_result",
        Some(&bob_token),
        None,
    )
    .await;
    assert_eq!(bob_notice_status, StatusCode::OK);
    let bob_notifications = bob_notice_body["notifications"]
        .as_array()
        .expect("bob notifications array");
    let result_notice = bob_notifications
        .iter()
        .find(|n| n["kind"] == "group_invite_result")
        .expect("result notification exists");
    assert_eq!(result_notice["action_payload"]["status"], "rejected");
}
