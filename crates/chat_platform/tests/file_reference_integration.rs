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

#[tokio::test]
async fn delete_group_cleans_message_file_references() {
    let unique = chrono::Utc::now().timestamp_millis();
    let account = format!("alice_ref_case_{}", unique);
    let group_code = format!("REFCNT{}", unique);
    let file_id = format!("f_test_msg_ref_{}", unique);
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
        pool: pool.clone(),
        ws_manager: WsManager::new(),
        message_record_manager,
    });

    let (register_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({
            "name": "Alice",
            "account": account,
            "password": "Alice2026!"
        })),
    )
    .await;
    assert_eq!(register_status, StatusCode::CREATED);

    let (login_status, login_body) = send_json(
        &app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({
            "account": account,
            "password": "Alice2026!"
        })),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK);
    let user_token = body_str(&login_body, "token").to_string();

    let (bots_status, bots_body) =
        send_json(&app, "GET", "/api/v1/bots", Some(&user_token), None).await;
    assert_eq!(bots_status, StatusCode::OK);
    let bot_id = body_str(&bots_body["bots"][0], "bot_id").to_string();
    let bot_token = body_str(&bots_body["bots"][0], "token").to_string();

    let (group_status, group_body) = send_json(
        &app,
        "POST",
        "/api/v1/groups",
        Some(&user_token),
        Some(json!({
            "name": "引用计数测试群",
            "group_code": group_code
        })),
    )
    .await;
    assert_eq!(group_status, StatusCode::CREATED);
    let group_id = body_str(&group_body, "group_id").to_string();

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO files (file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&file_id)
    .bind(&bot_id)
    .bind("hash-msg-ref")
    .bind("spec.txt")
    .bind(10_i64)
    .bind("text/plain")
    .bind(temp_dir.path().join("files/spec.txt").to_string_lossy().to_string())
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert file");

    sqlx::query("INSERT INTO file_uploaders (file_id, uploader_id, created_at) VALUES ($1, $2, $3)")
        .bind(&file_id)
        .bind(&bot_id)
        .bind(&now)
        .execute(&pool)
        .await
        .expect("insert uploader");

    let file_url = format!(
        "http://127.0.0.1:8080/api/v1/file/download/{}/spec.txt",
        file_id
    );
    let (send_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/message/send",
        Some(&bot_token),
        Some(json!({
            "group_id": group_id,
            "content": {
                "url": file_url,
                "filename": "spec.txt"
            },
            "msg_type": "file",
            "idempotency_key": "file-ref-msg-1"
        })),
    )
    .await;
    assert_eq!(send_status, StatusCode::CREATED);

    let reference_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM file_references WHERE file_id = $1 AND reference_type = 'message'",
    )
    .bind(&file_id)
    .fetch_one(&pool)
    .await
    .expect("count file refs");
    assert_eq!(reference_count, 1);

    let (delete_group_status, _) = send_json(
        &app,
        "DELETE",
        &format!("/api/v1/groups/{group_id}"),
        Some(&user_token),
        None,
    )
    .await;
    assert_eq!(delete_group_status, StatusCode::OK);

    let remaining_reference_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM file_references WHERE file_id = $1 AND reference_type = 'message'",
    )
    .bind(&file_id)
    .fetch_one(&pool)
    .await
    .expect("count remaining refs");
    assert_eq!(remaining_reference_count, 0);

    let file_exists_after_delete_group: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM files WHERE file_id = $1)")
            .bind(&file_id)
            .fetch_one(&pool)
            .await
            .expect("check file existence");
    assert!(!file_exists_after_delete_group);
}

#[tokio::test]
async fn bot_avatar_reset_reduces_file_reference() {
    let unique = chrono::Utc::now().timestamp_millis();
    let account = format!("bob_ref_case_{}", unique);
    let file_id = format!("f_test_avatar_ref_{}", unique);
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
        pool: pool.clone(),
        ws_manager: WsManager::new(),
        message_record_manager,
    });

    let (register_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({
            "name": "Bob",
            "account": account,
            "password": "Bob2026!"
        })),
    )
    .await;
    assert_eq!(register_status, StatusCode::CREATED);

    let (login_status, login_body) = send_json(
        &app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({
            "account": account,
            "password": "Bob2026!"
        })),
    )
    .await;
    assert_eq!(login_status, StatusCode::OK);
    let user_token = body_str(&login_body, "token").to_string();

    let (bots_status, bots_body) =
        send_json(&app, "GET", "/api/v1/bots", Some(&user_token), None).await;
    assert_eq!(bots_status, StatusCode::OK);
    let bot_id = body_str(&bots_body["bots"][0], "bot_id").to_string();
    let bot_name = body_str(&bots_body["bots"][0], "name").to_string();

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO files (file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&file_id)
    .bind(&bot_id)
    .bind("hash-avatar-ref")
    .bind("avatar.png")
    .bind(128_i64)
    .bind("image/png")
    .bind(temp_dir.path().join("files/avatar.png").to_string_lossy().to_string())
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert file");

    sqlx::query("INSERT INTO file_uploaders (file_id, uploader_id, created_at) VALUES ($1, $2, $3)")
        .bind(&file_id)
        .bind(&bot_id)
        .bind(&now)
        .execute(&pool)
        .await
        .expect("insert uploader");

    let avatar_url = format!(
        "http://127.0.0.1:8080/api/v1/file/download/{}/avatar.png",
        file_id
    );
    let (set_avatar_status, _) = send_json(
        &app,
        "PUT",
        &format!("/api/v1/bots/{bot_id}"),
        Some(&user_token),
        Some(json!({
            "name": bot_name,
            "description": "avatar ref test",
            "avatar_url": avatar_url
        })),
    )
    .await;
    assert_eq!(set_avatar_status, StatusCode::OK);

    let ref_count_after_set: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM file_references WHERE file_id = $1 AND reference_type = 'bot_avatar' AND reference_id = $2",
    )
    .bind(&file_id)
    .bind(&bot_id)
    .fetch_one(&pool)
    .await
    .expect("count avatar refs");
    assert_eq!(ref_count_after_set, 1);

    let (reset_avatar_status, _) = send_json(
        &app,
        "PUT",
        &format!("/api/v1/bots/{bot_id}"),
        Some(&user_token),
        Some(json!({
            "name": "reset-avatar",
            "description": "avatar ref reset",
            "avatar_url": null
        })),
    )
    .await;
    assert_eq!(reset_avatar_status, StatusCode::OK);

    let ref_count_after_reset: i64 = sqlx::query_scalar(
        "SELECT COUNT(1) FROM file_references WHERE file_id = $1 AND reference_type = 'bot_avatar' AND reference_id = $2",
    )
    .bind(&file_id)
    .bind(&bot_id)
    .fetch_one(&pool)
    .await
    .expect("count avatar refs after reset");
    assert_eq!(ref_count_after_reset, 0);

    let file_exists_after_reset: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM files WHERE file_id = $1)")
            .bind(&file_id)
            .fetch_one(&pool)
            .await
            .expect("check file existence after reset");
    assert!(!file_exists_after_reset);
}
