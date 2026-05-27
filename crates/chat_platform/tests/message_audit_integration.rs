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

async fn send_plain(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
) -> (StatusCode, String, String) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).expect("build request"))
        .await
        .expect("request should succeed");
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let body_str = String::from_utf8_lossy(&body).to_string();
    (status, content_type, body_str)
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

#[tokio::test]
async fn message_idempotency_and_audit_access_work() {
    let unique = chrono::Utc::now().timestamp_millis();
    let super_admin_account = format!("super_admin_case_{}", unique);
    unsafe {
        std::env::set_var("SUPER_ADMIN_ACCOUNT", &super_admin_account);
    }

    let (app, _tmp) = setup_app().await;
    let alice_token = register_and_login(&app, &format!("alice_ma_{}", unique), "Alice").await;
    let super_admin_token = register_and_login(&app, &super_admin_account, "SuperAdmin").await;

    let (bots_status, bots_body) = send_json(&app, "GET", "/api/v1/bots", Some(&alice_token), None).await;
    assert_eq!(bots_status, StatusCode::OK);
    let alice_bot_token = body_str(&bots_body["bots"][0], "token").to_string();

    let group_code = format!("MSGA{}", unique);
    let (group_status, group_body) = send_json(
        &app,
        "POST",
        "/api/v1/groups",
        Some(&alice_token),
        Some(json!({
            "name": "Message Audit Group",
            "group_code": group_code,
            "is_public": true
        })),
    )
    .await;
    assert_eq!(group_status, StatusCode::CREATED);
    let group_id = body_str(&group_body, "group_id").to_string();

    let first_payload = json!({
        "group_id": group_id,
        "content": {"text":"hello idempotent"},
        "msg_type": "text",
        "idempotency_key": "idem-key-1"
    });
    let (first_status, first_body) = send_json(
        &app,
        "POST",
        "/api/v1/message/send",
        Some(&alice_bot_token),
        Some(first_payload.clone()),
    )
    .await;
    assert_eq!(first_status, StatusCode::CREATED);
    let first_msg_id = first_body["msg_id"].as_i64().expect("msg id");

    let (second_status, second_body) = send_json(
        &app,
        "POST",
        "/api/v1/message/send",
        Some(&alice_bot_token),
        Some(first_payload),
    )
    .await;
    assert_eq!(second_status, StatusCode::OK);
    let second_msg_id = second_body["msg_id"].as_i64().expect("msg id");
    assert_eq!(first_msg_id, second_msg_id);

    let (bad_status, bad_body) = send_json(
        &app,
        "POST",
        "/api/v1/message/send",
        Some(&alice_bot_token),
        Some(json!({
            "group_id": group_id,
            "content": {"text":"bad"},
            "msg_type": "text",
            "idempotency_key": "   "
        })),
    )
    .await;
    assert_eq!(bad_status, StatusCode::BAD_REQUEST);
    assert_ne!(bad_body, json!({}));

    let (normal_audit_status, _) =
        send_json(&app, "GET", "/api/v1/audit/logs?limit=20", Some(&alice_token), None).await;
    assert_eq!(normal_audit_status, StatusCode::FORBIDDEN);

    let (admin_audit_status, admin_audit_body) = send_json(
        &app,
        "GET",
        "/api/v1/audit/logs?action=message.send&limit=50",
        Some(&super_admin_token),
        None,
    )
    .await;
    assert_eq!(admin_audit_status, StatusCode::OK);
    let logs = admin_audit_body["logs"].as_array().expect("logs array");
    assert!(!logs.is_empty());

    let (csv_status, csv_content_type, csv_body) = send_plain(
        &app,
        "GET",
        "/api/v1/audit/logs/export?action=message.send&limit=50",
        Some(&super_admin_token),
    )
    .await;
    assert_eq!(csv_status, StatusCode::OK);
    assert!(csv_content_type.starts_with("text/csv"));
    assert!(csv_body.contains("log_id,created_at,actor_type"));
}

#[tokio::test]
async fn message_send_rejects_non_member_and_unknown_group() {
    let unique = chrono::Utc::now().timestamp_millis();
    let (app, _tmp) = setup_app().await;

    let alice_token = register_and_login(&app, &format!("alice_msg_{}", unique), "Alice").await;
    let bob_token = register_and_login(&app, &format!("bob_msg_{}", unique), "Bob").await;

    let (alice_bots_status, alice_bots_body) =
        send_json(&app, "GET", "/api/v1/bots", Some(&alice_token), None).await;
    assert_eq!(alice_bots_status, StatusCode::OK);
    let alice_bot_token = body_str(&alice_bots_body["bots"][0], "token").to_string();

    let (bob_bots_status, bob_bots_body) =
        send_json(&app, "GET", "/api/v1/bots", Some(&bob_token), None).await;
    assert_eq!(bob_bots_status, StatusCode::OK);
    let bob_bot_token = body_str(&bob_bots_body["bots"][0], "token").to_string();

    let group_code = format!("MEM{}", unique);
    let (group_status, group_body) = send_json(
        &app,
        "POST",
        "/api/v1/groups",
        Some(&alice_token),
        Some(json!({
            "name": "Membership Group",
            "group_code": group_code,
            "is_public": true
        })),
    )
    .await;
    assert_eq!(group_status, StatusCode::CREATED);
    let group_id = body_str(&group_body, "group_id").to_string();

    let (unknown_group_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/message/send",
        Some(&alice_bot_token),
        Some(json!({
            "group_id": "g_missing_group",
            "content": {"text":"should fail"},
            "msg_type": "text",
            "idempotency_key": "missing-group-1"
        })),
    )
    .await;
    assert_eq!(unknown_group_status, StatusCode::BAD_REQUEST);

    let (non_member_status, non_member_body) = send_json(
        &app,
        "POST",
        "/api/v1/message/send",
        Some(&bob_bot_token),
        Some(json!({
            "group_id": group_id,
            "content": {"text":"should fail"},
            "msg_type": "text",
            "idempotency_key": "non-member-1"
        })),
    )
    .await;
    assert_eq!(non_member_status, StatusCode::BAD_REQUEST);
    assert_eq!(non_member_body["code"], "bot_not_in_group");
}
