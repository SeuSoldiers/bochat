# bochat_sdk

`bochat_sdk` 是一个面向 BoChat 平台的 Rust 异步 SDK。

`bochat_sdk` is an async Rust SDK for the BoChat platform.

## 能力概览

- 账号密码注册/登录（支持可选昵称）
- 用户 token 与 Bot token 状态管理
- Bot 列表、创建、更新、删除
- 消息发送与历史拉取（支持 `idempotency_key`）
- 文件上传/下载 URL 生成
- 可选高级 WebSocket 会话（自动重连、心跳、事件流）

## Feature Overview

- Register/login with account and password, with optional nickname
- Built-in user token and bot token storage
- Bot list, create, update, delete
- Send messages and fetch history with `idempotency_key`
- Upload files and build download URLs
- Optional advanced WebSocket session with reconnect and heartbeat

## 快速开始

```rust
use bochat_sdk::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> SdkResult<()> {
    let client = BochatClient::builder("http://127.0.0.1:8080").build()?;

    client
        .auth()
        .login()
        .account("demo_user_01")
        .password("Passw0rd!")
        .send()
        .await?;

    let bots = client.bots().list().await?;
    let first_bot = bots.first().expect("至少需要一个 Bot");
    client.set_bot_token(Some(first_bot.token.clone())).await;

    let sent = client
        .messages()
        .send_text("g_demo", "hello", "demo-idempotency-1")
        .await?;

    println!("msg_id={}", sent.msg_id);
    Ok(())
}
```

## Quick Start

1. Create a client with the backend base URL.
2. Register or login to obtain a user token.
3. Query your bots and select one bot token for chat/file APIs.
4. Use `messages()`, `groups()`, `files()` and optional `ws()` APIs.

## Token 说明 / Token Notes

- 用户级接口使用 user token，例如资料、Bot 管理、群管理。
- Bot 级接口使用 bot token，例如发消息、拉历史、上传文件、WebSocket。
- `register()` / `login()` 成功后，SDK 会自动保存 user token。
- 可以调用 `client.bots().use_bot_token(Some(bot_id)).await?` 自动选中 Bot token。

- User-level APIs require the user token, such as profile, bot management, and group management.
- Bot-level APIs require the bot token, such as messaging, history, file upload, and WebSocket.
- The SDK stores the user token automatically after `register()` or `login()`.
- You can call `client.bots().use_bot_token(Some(bot_id)).await?` to persist a chosen bot token in the client.

## 常见流程 / Common Flow

```rust
use bochat_sdk::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> SdkResult<()> {
    let client = BochatClient::builder("http://127.0.0.1:8080").build()?;

    client
        .auth()
        .login()
        .account("demo_user_01")
        .password("Passw0rd!")
        .send()
        .await?;

    let bots = client.bots().list().await?;
    let bot = bots.first().expect("at least one bot");
    client.set_bot_token(Some(bot.token.clone())).await;

    let group = client
        .groups()
        .create(CreateGroupRequest {
            name: "sdk-demo".to_string(),
            description: Some("created by SDK".to_string()),
            group_code: Some("SDKDEMO01".to_string()),
            bot_id: Some(bot.bot_id.clone()),
        })
        .await?;

    let sent = client
        .messages()
        .send_text(&group.group_id, "hello", "sdk-demo-msg-1")
        .await?;

    let history = client
        .messages()
        .history(&group.group_id, None, Some(20))
        .await?;

    println!("sent={}, history={}", sent.msg_id, history.messages.len());
    Ok(())
}
```

## 设计约束 / API Semantics

- `idempotency_key` is mandatory for message sending. Reuse the same key only for retries of the same logical message.
- Group history requires a bot token and only works when that bot can access the target group.
- `files().download_url(file_id, filename)` must include the original filename because the backend download route is `/api/v1/file/download/{file_id}/{filename}`.

- 发送消息时 `idempotency_key` 为必填项，只应在“同一条逻辑消息重试”时复用。
- 群消息历史查询要求使用 bot token，且该 Bot 必须对目标群有访问权限。
- `files().download_url(file_id, filename)` 必须带上原始文件名，因为后端下载路由为 `/api/v1/file/download/{file_id}/{filename}`。

## WebSocket 会话

启用 `ws` feature：

```toml
bochat_sdk = { path = "crates/bochat_sdk", features = ["ws"] }
```

示例见：

- `examples/basic_flow.rs`
- `examples/ws_session.rs`

## WebSocket Session

- Build the session with `client.ws().build().await?`
- Start it with `spawn()`
- Wait for the initial `connection` event through `wait_connection_payload()`
- Use `into_dispatcher()` if you want per-group subscriptions or handlers
