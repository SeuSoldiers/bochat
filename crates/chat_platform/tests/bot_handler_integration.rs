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
            file_storage_path: temp_dir.path().join("files").to_string_lossy().to_string(),
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
    let temp_dir = TempDir::new().expect("temp dir");
    let config = test_config(&temp_dir);
    let pool = db::init_pool(&config.database).await.expect("db pool");
    db::init_schema(&pool).await.expect("init schema");

    let redis_client = redis::Client::open(config.redis.url.as_str()).expect("redis url");
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");
    let redis_cache = RedisMessageCache::new(redis_conn);
    let message_record_manager = MessageRecordManager::new(pool.clone(), redis_cache)
        .await
        .expect("msg mgr");

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
        Some(json!({"name":name,"account":account,"password":"Pass2026!"})),
    )
    .await;
    assert_eq!(register_status, StatusCode::CREATED);
    let (login_status, login_body) = send_json(
        app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({"account":account,"password":"Pass2026!"})),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK);
    body_str(&login_body, "token").to_string()
}

#[tokio::test]
async fn bot_handlers_core_paths_work() {
    let unique = chrono::Utc::now().timestamp_millis();
    let (app, _tmp) = setup_app().await;
    let alice_token = register_and_login(&app, &format!("alice_bot_{}", unique), "Alice").await;
    let bob_token = register_and_login(&app, &format!("bob_bot_{}", unique), "Bob").await;

    let (bad_create_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/bots",
        Some(&alice_token),
        Some(json!({"name":""})),
    )
    .await;
    assert_eq!(bad_create_status, StatusCode::BAD_REQUEST);

    let (create_status, create_body) = send_json(
        &app,
        "POST",
        "/api/v1/bots",
        Some(&alice_token),
        Some(json!({"name":"Alice Helper","description":"d1"})),
    )
    .await;
    assert_eq!(create_status, StatusCode::CREATED);
    let created_bot_id = body_str(&create_body, "bot_id").to_string();

    let (list_status, list_body) = send_json(&app, "GET", "/api/v1/bots", Some(&alice_token), None).await;
    assert_eq!(list_status, StatusCode::OK);
    assert!(list_body["bots"]
        .as_array()
        .expect("bots arr")
        .iter()
        .any(|b| b["bot_id"] == created_bot_id));

    let (get_status, get_body) = send_json(
        &app,
        "GET",
        &format!("/api/v1/bots/{created_bot_id}"),
        None,
        None,
    )
    .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_body["bot_id"], created_bot_id);

    let (search_status, search_body) = send_json(
        &app,
        "GET",
        &format!("/api/v1/bots/search?bot_id={created_bot_id}"),
        Some(&alice_token),
        None,
    )
    .await;
    assert_eq!(search_status, StatusCode::OK);
    assert_eq!(
        search_body["bots"].as_array().expect("bots arr")[0]["bot_id"],
        created_bot_id
    );

    let (update_status, update_body) = send_json(
        &app,
        "PUT",
        &format!("/api/v1/bots/{created_bot_id}"),
        Some(&alice_token),
        Some(json!({"name":"Alice Helper 2","description":"d2","avatar_url":"http://x"})),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(update_body["name"], "Alice Helper 2");

    let (alice_bots_status, alice_bots_body) =
        send_json(&app, "GET", "/api/v1/bots", Some(&alice_token), None).await;
    assert_eq!(alice_bots_status, StatusCode::OK);
    let alice_default_bot_token = body_str(&alice_bots_body["bots"][0], "token").to_string();
    let (profile_status, profile_body) =
        send_json(&app, "GET", "/api/v1/bot/profile", Some(&alice_default_bot_token), None).await;
    assert_eq!(profile_status, StatusCode::OK);
    assert!(profile_body["bot_id"].is_string());

    let (forbidden_delete_status, _) = send_json(
        &app,
        "DELETE",
        &format!("/api/v1/bots/{created_bot_id}"),
        Some(&bob_token),
        None,
    )
    .await;
    assert_eq!(forbidden_delete_status, StatusCode::FORBIDDEN);

    let (delete_status, _) = send_json(
        &app,
        "DELETE",
        &format!("/api/v1/bots/{created_bot_id}"),
        Some(&alice_token),
        None,
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK);
}
