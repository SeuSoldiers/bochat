use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    Router,
};
use chat_platform::{
    app_router,
    cache::RedisMessageCache,
    config::{
        Config, DatabaseConfig, LoggingConfig, RedisConfig, SecurityConfig, ServerConfig,
        StorageConfig,
    },
    db,
    services::message_record_manager::MessageRecordManager,
    ws::WsManager,
    AppState,
};
use serde_json::{json, Value};
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
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://chat_user:chat_pass@localhost:5432/chat_platform_test".to_string()),
            max_connections: 2,
            min_connections: 1,
        },
        redis: RedisConfig {
            url: std::env::var("TEST_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
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
async fn chat_flow_from_python_script_is_covered_by_integration_test() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config = test_config(&temp_dir);
    let pool = db::init_pool(&config.database).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let redis_client = redis::Client::open(config.redis.url.as_str())
        .expect("invalid Redis URL in test");
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

    let (health_status, _) = send_json(&app, "GET", "/health", None, None).await;
    assert_eq!(health_status, StatusCode::OK);

    let (alice_register_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({
            "name": "Alice",
            "account": "alice_1001",
            "password": "Alice2026!"
        })),
    )
    .await;
    assert_eq!(alice_register_status, StatusCode::CREATED);

    let (bob_register_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/auth/register",
        None,
        Some(json!({
            "name": "Bob",
            "account": "bob_1002",
            "password": "Bob2026!"
        })),
    )
    .await;
    assert_eq!(bob_register_status, StatusCode::CREATED);

    let (alice_login_status, alice_login_body) = send_json(
        &app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({
            "account": "alice_1001",
            "password": "Alice2026!"
        })),
    )
    .await;
    assert_eq!(alice_login_status, StatusCode::OK);
    let alice_token = body_str(&alice_login_body, "token").to_string();

    let (bob_login_status, bob_login_body) = send_json(
        &app,
        "POST",
        "/api/v1/auth/login",
        None,
        Some(json!({
            "account": "bob_1002",
            "password": "Bob2026!"
        })),
    )
    .await;
    assert_eq!(bob_login_status, StatusCode::OK);
    let bob_token = body_str(&bob_login_body, "token").to_string();

    let (alice_bots_status, alice_bots_body) =
        send_json(&app, "GET", "/api/v1/bots", Some(&alice_token), None).await;
    assert_eq!(alice_bots_status, StatusCode::OK);
    let alice_default_bot_id = body_str(&alice_bots_body["bots"][0], "bot_id").to_string();
    let alice_default_bot_token = body_str(&alice_bots_body["bots"][0], "token").to_string();

    let (bob_bots_status, bob_bots_body) =
        send_json(&app, "GET", "/api/v1/bots", Some(&bob_token), None).await;
    assert_eq!(bob_bots_status, StatusCode::OK);
    let bob_default_bot_id = body_str(&bob_bots_body["bots"][0], "bot_id").to_string();
    let bob_default_bot_token = body_str(&bob_bots_body["bots"][0], "token").to_string();

    let (alice_group_status, alice_group_body) = send_json(
        &app,
        "POST",
        "/api/v1/groups",
        Some(&alice_token),
        Some(json!({
            "name": "技术讨论组",
            "description": "讨论技术问题的群聊",
            "group_code": "TECH001"
        })),
    )
    .await;
    assert_eq!(alice_group_status, StatusCode::CREATED);
    let tech_group_id = body_str(&alice_group_body, "group_id").to_string();

    let (bob_group_status, bob_group_body) = send_json(
        &app,
        "POST",
        "/api/v1/groups",
        Some(&bob_token),
        Some(json!({
            "name": "产品反馈组",
            "description": "收集产品反馈的群聊",
            "group_code": "PROD001"
        })),
    )
    .await;
    assert_eq!(bob_group_status, StatusCode::CREATED);
    let product_group_id = body_str(&bob_group_body, "group_id").to_string();

    let (bob_join_tech_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/groups/join",
        Some(&bob_token),
        Some(json!({ "group_code": "TECH001" })),
    )
    .await;
    assert_eq!(bob_join_tech_status, StatusCode::OK);

    let (alice_join_product_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/groups/join",
        Some(&alice_token),
        Some(json!({ "group_code": "PROD001" })),
    )
    .await;
    assert_eq!(alice_join_product_status, StatusCode::OK);

    let (alice_second_bot_status, alice_second_bot_body) = send_json(
        &app,
        "POST",
        "/api/v1/bots",
        Some(&alice_token),
        Some(json!({
            "name": "Alice的AI助手",
            "description": "帮助Alice处理任务"
        })),
    )
    .await;
    assert_eq!(alice_second_bot_status, StatusCode::CREATED);
    let alice_second_bot_id = body_str(&alice_second_bot_body, "bot_id").to_string();
    let alice_second_bot_token = body_str(&alice_second_bot_body, "token").to_string();

    let (second_bot_join_status, _) = send_json(
        &app,
        "POST",
        "/api/v1/groups/join",
        Some(&alice_token),
        Some(json!({
            "group_code": "TECH001",
            "bot_id": alice_second_bot_id
        })),
    )
    .await;
    assert_eq!(second_bot_join_status, StatusCode::OK);

    for (index, (token, payload)) in [
        (
            alice_default_bot_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "大家好！这是技术讨论组，欢迎加入！" },
                "msg_type": "text"
            }),
        ),
        (
            bob_default_bot_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "感谢Alice邀请我加入！我们可以讨论什么话题呢？" },
                "msg_type": "text"
            }),
        ),
        (
            alice_default_bot_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "我们可以讨论Rust、Python、区块链等话题" },
                "msg_type": "text"
            }),
        ),
        (
            alice_second_bot_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "我是Alice的AI助手，很高兴认识大家！" },
                "msg_type": "text"
            }),
        ),
        (
            alice_second_bot_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "有什么我可以帮助的吗？" },
                "msg_type": "text"
            }),
        ),
        (
            bob_default_bot_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "太棒了！我最近在学习Rust" },
                "msg_type": "text"
            }),
        ),
        (
            alice_default_bot_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "Rust是个很强大的语言，我们一起探讨吧！" },
                "msg_type": "text"
            }),
        ),
        (
            bob_default_bot_token.as_str(),
            json!({
                "group_id": product_group_id,
                "content": { "text": "欢迎来到产品反馈组！" },
                "msg_type": "text"
            }),
        ),
        (
            alice_default_bot_token.as_str(),
            json!({
                "group_id": product_group_id,
                "content": { "text": "感谢邀请，我有一些关于用户界面的反馈" },
                "msg_type": "text"
            }),
        ),
        (
            bob_default_bot_token.as_str(),
            json!({
                "group_id": product_group_id,
                "content": { "text": "请继续，我们很想听听你的意见" },
                "msg_type": "text"
            }),
        ),
        (
            alice_default_bot_token.as_str(),
            json!({
                "group_id": product_group_id,
                "content": { "text": "我觉得可以添加暗黑模式和多语言支持" },
                "msg_type": "text"
            }),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let payload_with_idempotency = match payload {
            serde_json::Value::Object(mut map) => {
                map.insert(
                    "idempotency_key".to_string(),
                    serde_json::Value::String(format!("msg-{index}")),
                );
                serde_json::Value::Object(map)
            }
            other => other,
        };

        let (status, _) = send_json(
            &app,
            "POST",
            "/api/v1/message/send",
            Some(token),
            Some(payload_with_idempotency),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let duplicate_payload = json!({
        "group_id": tech_group_id,
        "content": { "text": "幂等测试消息" },
        "msg_type": "text",
        "idempotency_key": "dedupe-tech-1"
    });
    let (first_duplicate_status, first_duplicate_body) = send_json(
        &app,
        "POST",
        "/api/v1/message/send",
        Some(&alice_default_bot_token),
        Some(duplicate_payload.clone()),
    )
    .await;
    assert_eq!(first_duplicate_status, StatusCode::CREATED);

    let (second_duplicate_status, second_duplicate_body) = send_json(
        &app,
        "POST",
        "/api/v1/message/send",
        Some(&alice_default_bot_token),
        Some(duplicate_payload),
    )
    .await;
    assert_eq!(second_duplicate_status, StatusCode::OK);
    assert_eq!(
        first_duplicate_body["msg_id"],
        second_duplicate_body["msg_id"]
    );

    let (tech_messages_status, tech_messages_body) = send_json(
        &app,
        "GET",
        &format!("/api/v1/groups/{tech_group_id}/messages?limit=100"),
        Some(&bob_default_bot_token),
        None,
    )
    .await;
    assert_eq!(tech_messages_status, StatusCode::OK);
    let tech_messages = tech_messages_body["messages"]
        .as_array()
        .expect("messages should be an array");
    assert_eq!(tech_messages_body["limit"], 100);
    assert_eq!(tech_messages.len(), 8);
    assert_eq!(tech_messages[0]["sender_id"], alice_default_bot_id);
    assert_eq!(tech_messages[1]["sender_id"], bob_default_bot_id);
    assert_eq!(tech_messages[3]["sender_id"], alice_second_bot_id);

    let (product_messages_status, product_messages_body) = send_json(
        &app,
        "GET",
        &format!("/api/v1/groups/{product_group_id}/messages?limit=100"),
        Some(&alice_default_bot_token),
        None,
    )
    .await;
    assert_eq!(product_messages_status, StatusCode::OK);
    let product_messages = product_messages_body["messages"]
        .as_array()
        .expect("messages should be an array");
    assert_eq!(product_messages.len(), 4);

    let (tech_members_status, tech_members_body) = send_json(
        &app,
        "GET",
        &format!("/api/v1/groups/{tech_group_id}/members"),
        Some(&alice_token),
        None,
    )
    .await;
    assert_eq!(tech_members_status, StatusCode::OK);
    let tech_members = tech_members_body["members"]
        .as_array()
        .expect("members should be an array");
    assert_eq!(tech_members.len(), 3);

    let (product_members_status, product_members_body) = send_json(
        &app,
        "GET",
        &format!("/api/v1/groups/{product_group_id}/members"),
        Some(&bob_token),
        None,
    )
    .await;
    assert_eq!(product_members_status, StatusCode::OK);
    let product_members = product_members_body["members"]
        .as_array()
        .expect("members should be an array");
    assert_eq!(product_members.len(), 2);

    let (bob_groups_status, bob_groups_body) =
        send_json(&app, "GET", "/api/v1/groups", Some(&bob_token), None).await;
    assert_eq!(bob_groups_status, StatusCode::OK);
    let bob_groups = bob_groups_body["groups"]
        .as_array()
        .expect("groups should be an array");
    assert_eq!(bob_groups.len(), 2);
}
