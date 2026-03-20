use actix_web::{
    http::{header, StatusCode},
    test, web, App,
};
use chat_platform::{
    config::{Config, DatabaseConfig, SecurityConfig, ServerConfig, StorageConfig},
    configure_routes, db,
    ws::WsManager,
};
use serde_json::{json, Value};
use tempfile::TempDir;

fn test_config(temp_dir: &TempDir) -> Config {
    let file_storage_path = temp_dir.path().join("files");

    Config {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            workers: 1,
        },
        database: DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            min_connections: 1,
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
    }
}

fn auth_header(token: &str) -> (header::HeaderName, String) {
    (header::AUTHORIZATION, format!("Bearer {token}"))
}

fn body_str<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key]
        .as_str()
        .unwrap_or_else(|| panic!("missing string field `{key}` in {value}"))
}

#[actix_web::test]
async fn chat_flow_from_python_script_is_covered_by_integration_test() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let config = test_config(&temp_dir);
    let pool = db::init_pool(&config.database).await.expect("init db pool");
    db::run_migrations(&pool).await.expect("run migrations");

    let config_data = web::Data::new(config);
    let pool_data = web::Data::new(pool);
    let ws_manager_data = web::Data::new(WsManager::new());

    let app = test::init_service(
        App::new()
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(ws_manager_data.clone())
            .configure(configure_routes),
    )
    .await;

    let health_response = test::call_service(&app, test::TestRequest::get().uri("/health").to_request()).await;
    assert_eq!(health_response.status(), StatusCode::OK);

    let alice_register = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "name": "Alice",
                "id_number": "110101199003071234",
                "phone": "13800138001"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(alice_register.status(), StatusCode::CREATED);

    let bob_register = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "name": "Bob",
                "id_number": "110101199003071235",
                "phone": "13800138002"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(bob_register.status(), StatusCode::CREATED);

    let alice_login = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({
                "id_number": "110101199003071234",
                "phone": "13800138001"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(alice_login.status(), StatusCode::OK);
    let alice_login_body: Value = test::read_body_json(alice_login).await;
    let alice_token = body_str(&alice_login_body, "bot_token").to_string();

    let bob_login = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({
                "id_number": "110101199003071235",
                "phone": "13800138002"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(bob_login.status(), StatusCode::OK);
    let bob_login_body: Value = test::read_body_json(bob_login).await;
    let bob_token = body_str(&bob_login_body, "bot_token").to_string();

    let alice_bots_response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/bots")
            .insert_header(auth_header(&alice_token))
            .to_request(),
    )
    .await;
    assert_eq!(alice_bots_response.status(), StatusCode::OK);
    let alice_bots_body: Value = test::read_body_json(alice_bots_response).await;
    let alice_default_bot_id = body_str(&alice_bots_body["bots"][0], "bot_id").to_string();

    let bob_bots_response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/bots")
            .insert_header(auth_header(&bob_token))
            .to_request(),
    )
    .await;
    assert_eq!(bob_bots_response.status(), StatusCode::OK);
    let bob_bots_body: Value = test::read_body_json(bob_bots_response).await;
    let bob_default_bot_id = body_str(&bob_bots_body["bots"][0], "bot_id").to_string();

    let alice_group_response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/groups")
            .insert_header(auth_header(&alice_token))
            .set_json(json!({
                "name": "技术讨论组",
                "description": "讨论技术问题的群聊",
                "group_code": "TECH001"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(alice_group_response.status(), StatusCode::CREATED);
    let alice_group_body: Value = test::read_body_json(alice_group_response).await;
    let tech_group_id = body_str(&alice_group_body, "group_id").to_string();

    let bob_group_response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/groups")
            .insert_header(auth_header(&bob_token))
            .set_json(json!({
                "name": "产品反馈组",
                "description": "收集产品反馈的群聊",
                "group_code": "PROD001"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(bob_group_response.status(), StatusCode::CREATED);
    let bob_group_body: Value = test::read_body_json(bob_group_response).await;
    let product_group_id = body_str(&bob_group_body, "group_id").to_string();

    let bob_join_tech = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/groups/join")
            .insert_header(auth_header(&bob_token))
            .set_json(json!({ "group_code": "TECH001" }))
            .to_request(),
    )
    .await;
    assert_eq!(bob_join_tech.status(), StatusCode::OK);

    let alice_join_product = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/groups/join")
            .insert_header(auth_header(&alice_token))
            .set_json(json!({ "group_code": "PROD001" }))
            .to_request(),
    )
    .await;
    assert_eq!(alice_join_product.status(), StatusCode::OK);

    let alice_second_bot_response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/bots")
            .insert_header(auth_header(&alice_token))
            .set_json(json!({
                "name": "Alice的AI助手",
                "description": "帮助Alice处理任务"
            }))
            .to_request(),
    )
    .await;
    assert_eq!(alice_second_bot_response.status(), StatusCode::CREATED);
    let alice_second_bot_body: Value = test::read_body_json(alice_second_bot_response).await;
    let alice_second_bot_id = body_str(&alice_second_bot_body, "bot_id").to_string();
    let alice_second_bot_token = body_str(&alice_second_bot_body, "token").to_string();

    let second_bot_join_tech = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/groups/join")
            .insert_header(auth_header(&alice_second_bot_token))
            .set_json(json!({ "group_code": "TECH001" }))
            .to_request(),
    )
    .await;
    assert_eq!(second_bot_join_tech.status(), StatusCode::OK);

    for (token, payload) in [
        (
            alice_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "大家好！这是技术讨论组，欢迎加入！" },
                "msg_type": "text"
            }),
        ),
        (
            bob_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "感谢Alice邀请我加入！我们可以讨论什么话题呢？" },
                "msg_type": "text"
            }),
        ),
        (
            alice_token.as_str(),
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
            bob_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "太棒了！我最近在学习Rust" },
                "msg_type": "text"
            }),
        ),
        (
            alice_token.as_str(),
            json!({
                "group_id": tech_group_id,
                "content": { "text": "Rust是个很强大的语言，我们一起探讨吧！" },
                "msg_type": "text"
            }),
        ),
        (
            bob_token.as_str(),
            json!({
                "group_id": product_group_id,
                "content": { "text": "欢迎来到产品反馈组！" },
                "msg_type": "text"
            }),
        ),
        (
            alice_token.as_str(),
            json!({
                "group_id": product_group_id,
                "content": { "text": "感谢邀请，我有一些关于用户界面的反馈" },
                "msg_type": "text"
            }),
        ),
        (
            bob_token.as_str(),
            json!({
                "group_id": product_group_id,
                "content": { "text": "请继续，我们很想听听你的意见" },
                "msg_type": "text"
            }),
        ),
        (
            alice_token.as_str(),
            json!({
                "group_id": product_group_id,
                "content": { "text": "我觉得可以添加暗黑模式和多语言支持" },
                "msg_type": "text"
            }),
        ),
    ] {
        let send_message_response = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/message/send")
                .insert_header(auth_header(token))
                .set_json(payload)
                .to_request(),
        )
        .await;
        assert_eq!(send_message_response.status(), StatusCode::CREATED);
    }

    let tech_messages_response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!(
                "/api/v1/groups/{tech_group_id}/messages?limit=100&offset=0"
            ))
            .insert_header(auth_header(&bob_token))
            .to_request(),
    )
    .await;
    assert_eq!(tech_messages_response.status(), StatusCode::OK);
    let tech_messages_body: Value = test::read_body_json(tech_messages_response).await;
    let tech_messages = tech_messages_body["messages"]
        .as_array()
        .expect("messages should be an array");
    assert_eq!(tech_messages_body["limit"], 100);
    assert_eq!(tech_messages_body["offset"], 0);
    assert_eq!(tech_messages.len(), 7);
    assert_eq!(tech_messages[0]["sender_id"], alice_default_bot_id);
    assert_eq!(tech_messages[1]["sender_id"], bob_default_bot_id);
    assert_eq!(tech_messages[3]["sender_id"], alice_second_bot_id);

    let product_messages_response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!(
                "/api/v1/groups/{product_group_id}/messages?limit=100&offset=0"
            ))
            .insert_header(auth_header(&alice_token))
            .to_request(),
    )
    .await;
    assert_eq!(product_messages_response.status(), StatusCode::OK);
    let product_messages_body: Value = test::read_body_json(product_messages_response).await;
    let product_messages = product_messages_body["messages"]
        .as_array()
        .expect("messages should be an array");
    assert_eq!(product_messages.len(), 4);

    let tech_members_response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/groups/{tech_group_id}/members"))
            .insert_header(auth_header(&alice_token))
            .to_request(),
    )
    .await;
    assert_eq!(tech_members_response.status(), StatusCode::OK);
    let tech_members_body: Value = test::read_body_json(tech_members_response).await;
    let tech_members = tech_members_body["members"]
        .as_array()
        .expect("members should be an array");
    assert_eq!(tech_members.len(), 3);

    let product_members_response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!("/api/v1/groups/{product_group_id}/members"))
            .insert_header(auth_header(&bob_token))
            .to_request(),
    )
    .await;
    assert_eq!(product_members_response.status(), StatusCode::OK);
    let product_members_body: Value = test::read_body_json(product_members_response).await;
    let product_members = product_members_body["members"]
        .as_array()
        .expect("members should be an array");
    assert_eq!(product_members.len(), 2);

    let bob_groups_response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/groups")
            .insert_header(auth_header(&bob_token))
            .to_request(),
    )
    .await;
    assert_eq!(bob_groups_response.status(), StatusCode::OK);
    let bob_groups_body: Value = test::read_body_json(bob_groups_response).await;
    let bob_groups = bob_groups_body["groups"]
        .as_array()
        .expect("groups should be an array");
    assert_eq!(bob_groups.len(), 2);
}
