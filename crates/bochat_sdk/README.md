# bochat_sdk

`bochat_sdk` 是一个面向 BoChat 平台的 Rust 异步 SDK。

## 能力概览

- 账号密码注册/登录（支持可选昵称）
- 用户 token 与 Bot token 状态管理
- Bot 列表、创建、更新、删除
- 消息发送与历史拉取（支持 `idempotency_key`）
- 文件上传/下载 URL 生成
- 可选高级 WebSocket 会话（自动重连、心跳、事件流）

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

## WebSocket 会话

启用 `ws` feature：

```toml
bochat_sdk = { path = "crates/bochat_sdk", features = ["ws"] }
```

示例见：

- `examples/basic_flow.rs`
- `examples/ws_session.rs`
