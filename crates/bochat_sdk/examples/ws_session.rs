use bochat_sdk::prelude::*;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> SdkResult<()> {
    let client = BochatClient::builder("http://127.0.0.1:8080").build()?;

    let _login = client
        .auth()
        .login()
        .account("demo_user_01")
        .password("Passw0rd")
        .send()
        .await?;

    let bots = client.bots().list().await?;
    let first_bot = bots.first().expect("需要至少一个 Bot");
    client.set_bot_token(Some(first_bot.token.clone())).await;

    let session = client
        .ws()
        .heartbeat_interval(Duration::from_secs(15))
        .reconnect_max_attempts(20)
        .build()
        .await?;

    let mut handle = session.spawn().await?;

    while let Some(event) = handle.events.recv().await {
        println!("[{}] {}", event.event_type, event.timestamp);
    }

    Ok(())
}
