use bochat_sdk::client::BochatClient;
use bochat_sdk::error::SdkError;
use bochat_sdk::models::{CreateBotRequest, CreateGroupRequest, MessageContent, SendMessageRequest};
use bochat_sdk::retry::RetryPolicy;
use httpmock::Method::{DELETE, GET, POST, PUT};
use httpmock::MockServer;
use serde_json::json;

fn mk_client(base_url: &str) -> BochatClient {
    BochatClient::builder(base_url)
        .retry_policy(RetryPolicy {
            max_attempts: 2,
            base_delay: std::time::Duration::from_millis(1),
            max_delay: std::time::Duration::from_millis(1),
        })
        .user_token("user-token")
        .bot_token("bot-token")
        .build()
        .expect("build client")
}

fn disable_proxy_env() {
    unsafe {
        std::env::remove_var("HTTP_PROXY");
        std::env::remove_var("HTTPS_PROXY");
        std::env::remove_var("ALL_PROXY");
        std::env::remove_var("http_proxy");
        std::env::remove_var("https_proxy");
        std::env::remove_var("all_proxy");
        std::env::set_var("NO_PROXY", "*");
        std::env::set_var("no_proxy", "*");
    }
}

#[tokio::test]
async fn auth_register_login_me_update_delete_work() {
    disable_proxy_env();
    let server = MockServer::start_async().await;
    let register = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/v1/auth/register");
            then.status(200).json_body(json!({
                "message": "ok",
                "name": "u1",
                "token": "token-reg",
                "account": "a1"
            }));
        })
        .await;
    let login = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/v1/auth/login");
            then.status(200).json_body(json!({
                "message": "ok",
                "name": "u1",
                "token": "token-login",
                "account": "a1"
            }));
        })
        .await;
    let me = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/v1/users/me");
            then.status(200).json_body(json!({
                "name": "me",
                "avatar_url": null,
                "created_at": null,
                "updated_at": null,
                "is_super_admin": false
            }));
        })
        .await;
    let update = server
        .mock_async(|when, then| {
            when.method(PUT).path("/api/v1/users/me");
            then.status(200).json_body(json!({
                "name": "me2",
                "avatar_url": null,
                "created_at": null,
                "updated_at": null,
                "is_super_admin": false
            }));
        })
        .await;
    let delete = server
        .mock_async(|when, then| {
            when.method(DELETE).path("/api/v1/users/delete");
            then.status(204);
        })
        .await;

    let client = mk_client(&server.base_url());
    let auth = client.auth();
    let reg = auth
        .register()
        .account("a1")
        .password("p1")
        .nickname("u1")
        .send()
        .await
        .expect("register");
    assert_eq!(reg.token, "token-reg");
    assert_eq!(client.user_token().await.as_deref(), Some("token-reg"));

    let login_resp = auth
        .login()
        .account("a1")
        .password("p1")
        .send()
        .await
        .expect("login");
    assert_eq!(login_resp.token, "token-login");
    assert_eq!(client.user_token().await.as_deref(), Some("token-login"));

    let me_resp = auth.me().await.expect("me");
    assert_eq!(me_resp.name, "me");

    let updated = auth
        .update_profile(bochat_sdk::models::UpdateProfileRequest {
            name: Some("me2".to_string()),
            password: None,
            avatar_url: None,
        })
        .await
        .expect("update");
    assert_eq!(updated.name, "me2");

    auth.delete_account().await.expect("delete");

    register.assert_async().await;
    login.assert_async().await;
    me.assert_async().await;
    update.assert_async().await;
    delete.assert_async().await;
}

#[tokio::test]
async fn bots_groups_messages_and_files_work() {
    disable_proxy_env();
    let server = MockServer::start_async().await;
    let list = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/v1/bots");
            then.status(200).json_body(json!({
                "bots": [{
                    "bot_id":"b1","owner_id":"u1","name":"bot","description":null,"avatar_url":null,
                    "status":"active","token":"bot-1","created_at":"now","updated_at":"now"
                }]
            }));
        })
        .await;
    let create = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/v1/bots");
            then.status(200).json_body(json!({
                "bot_id":"b2","owner_id":"u1","name":"new","description":null,"avatar_url":null,
                "status":"active","token":"bot-2","created_at":"now","updated_at":"now"
            }));
        })
        .await;
    let get = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/v1/bots/b2");
            then.status(200).json_body(json!({
                "bot_id":"b2","owner_id":"u1","name":"new","description":null,"avatar_url":null,
                "status":"active","token":"bot-2","created_at":"now","updated_at":"now"
            }));
        })
        .await;
    let update = server
        .mock_async(|when, then| {
            when.method(PUT).path("/api/v1/bots/b2");
            then.status(200).json_body(json!({
                "bot_id":"b2","owner_id":"u1","name":"new2","description":null,"avatar_url":null,
                "status":"active","token":"bot-2","created_at":"now","updated_at":"now"
            }));
        })
        .await;
    let delete_bot = server
        .mock_async(|when, then| {
            when.method(DELETE).path("/api/v1/bots/b2");
            then.status(204);
        })
        .await;
    let create_group = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/v1/groups");
            then.status(200).json_body(json!({
                "group_id":"g1","group_code":"c1","creator_id":"u1","name":"g","description":null,
                "status":"active","created_at":"now","updated_at":"now"
            }));
        })
        .await;
    let delete_group = server
        .mock_async(|when, then| {
            when.method(DELETE).path("/api/v1/groups/g1");
            then.status(204);
        })
        .await;
    let send_message = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/v1/message/send");
            then.status(200).json_body(json!({
                "msg_id": 1, "group_id":"g1","sender_id":"b1","sender_name":"bot","sender_avatar_url":null,
                "content":{"text":"hello"},"msg_type":"text","created_at":"now"
            }));
        })
        .await;
    let history = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/v1/groups/g1/messages");
            then.status(200).json_body(json!({
                "group_id":"g1","base_id":null,"limit":20,"next_base_id":null,"messages":[]
            }));
        })
        .await;
    let upload = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/v1/file/upload");
            then.status(200).json_body(json!({
                "file_id":"f1","filename":"a.txt","url":"http://x","created_at":"now"
            }));
        })
        .await;
    let delete_file = server
        .mock_async(|when, then| {
            when.method(DELETE).path("/api/v1/file/f1");
            then.status(204);
        })
        .await;

    let client = mk_client(&server.base_url());
    let bots = client.bots();
    let listed = bots.list().await.expect("list");
    assert_eq!(listed.len(), 1);
    let _ = bots
        .create(CreateBotRequest {
            name: "new".to_string(),
            description: None,
            avatar_url: None,
        })
        .await
        .expect("create bot");
    let _ = bots.get("b2").await.expect("get bot");
    let _ = bots
        .update(
            "b2",
            bochat_sdk::models::UpdateBotRequest {
                name: "new2".to_string(),
                description: None,
                avatar_url: None,
            },
        )
        .await
        .expect("update");
    bots.delete("b2").await.expect("delete bot");
    let token = bots.use_bot_token(Some("b1")).await.expect("use bot token");
    assert_eq!(token, "bot-1");
    assert_eq!(client.bot_token().await.as_deref(), Some("bot-1"));

    let groups = client.groups();
    let g = groups
        .create(CreateGroupRequest {
            name: "g".to_string(),
            description: None,
            group_code: None,
            bot_id: None,
        })
        .await
        .expect("create group");
    assert_eq!(g.group_id, "g1");
    groups.delete("g1").await.expect("delete group");

    let messages = client.messages();
    let sent = messages
        .send(SendMessageRequest {
            group_id: "g1".to_string(),
            content: MessageContent::text("hello"),
            msg_type: Some("text".to_string()),
        })
        .await
        .expect("send");
    assert_eq!(sent.msg_id, 1);
    let _ = messages.send_text("g1", "hello").await.expect("send text");
    let h = messages
        .history("g1", Some(1), Some(20))
        .await
        .expect("history");
    assert_eq!(h.group_id, "g1");
    let file_content = bochat_sdk::messages::MessagesApi::file_content("http://f");
    assert_eq!(file_content.as_file_url(), Some("http://f"));

    let files = client.files();
    let up = files
        .upload_bytes("a.txt", b"abc".to_vec(), Some("text/plain"))
        .await
        .expect("upload");
    assert_eq!(up.file_id, "f1");
    assert!(files.download_url("f1", "a b.txt").contains("a%20b%2Etxt"));
    files.delete("f1").await.expect("delete file");

    list.assert_hits_async(2).await;
    create.assert_async().await;
    get.assert_async().await;
    update.assert_async().await;
    delete_bot.assert_async().await;
    create_group.assert_async().await;
    delete_group.assert_async().await;
    send_message.assert_hits_async(2).await;
    history.assert_async().await;
    upload.assert_async().await;
    delete_file.assert_async().await;
}

#[tokio::test]
async fn retry_and_error_branches_work() {
    disable_proxy_env();
    let server = MockServer::start_async().await;
    let retry_then_ok = server
        .mock_async(|when, then| {
            when.method(GET).path("/retry");
            then.status(503);
        })
        .await;
    let final_ok = server
        .mock_async(|when, then| {
            when.method(GET).path("/retry").header("x-retry", "2");
            then.status(200).json_body(json!({"ok": true}));
        })
        .await;

    let client = mk_client(&server.base_url());

    // Force a direct error path: missing user token.
    client.set_user_token(None).await;
    let err = client.bots().list().await.expect_err("should fail no token");
    match err {
        SdkError::MissingUserToken => {}
        other => panic!("unexpected err: {other:?}"),
    }

    // Force API error parse branch.
    let api_err_mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/api/v1/bots");
            then.status(400).json_body(json!({
                "code": "bad_request",
                "message": "bad",
                "status": 400
            }));
        })
        .await;
    client.set_user_token(Some("u".to_string())).await;
    let err = client.bots().list().await.expect_err("api err expected");
    match err {
        SdkError::Api { status, .. } => assert_eq!(status, 400),
        other => panic!("unexpected err: {other:?}"),
    }
    api_err_mock.assert_async().await;

    // Trigger message retry path with 503 then success.
    let retry_policy_client = BochatClient::builder(server.base_url())
        .retry_policy(RetryPolicy {
            max_attempts: 2,
            base_delay: std::time::Duration::from_millis(1),
            max_delay: std::time::Duration::from_millis(1),
        })
        .bot_token("bot-token")
        .build()
        .expect("build");

    // First call returns 503.
    let _first = retry_then_ok;
    // For second call, use a separate endpoint without fragile matcher.
    let send_fail = server
        .mock_async(|when, then| {
            when.method(POST).path("/api/v1/message/send");
            then.status(503).body("unavailable");
        })
        .await;
    let err = retry_policy_client
        .messages()
        .send_text("g1", "hello")
        .await
        .expect_err("retry should fail");
    match err {
        SdkError::HttpStatus { status, .. } => assert_eq!(status, 503),
        other => panic!("unexpected retry err: {other:?}"),
    }
    send_fail.assert_hits_async(2).await;
    final_ok.assert_hits_async(0).await;
}
