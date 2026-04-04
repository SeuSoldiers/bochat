use bochat_sdk::prelude::*;
use std::env::args;
use tokio::signal;

#[tokio::main(flavor = "current_thread")]
async fn main() -> SdkResult<()> {
    if args().len() < 2 {
        eprintln!("请提供token参数");
        return Err(SdkError::MissingBotToken);
    }
    let token = args().nth(1).unwrap();

    let client = BochatClient::builder("http://127.0.0.1:8080").build()?;

    client.set_bot_token(Some(token)).await;

    let session = client.ws().build().await?;

    let mut dispatcher = session.spawn().await?.into_dispatcher();

    let conn = dispatcher.wait_connection_payload().await?;
    println!(
        "连接成功: bot={} 可用群={}",
        conn.bot_name,
        conn.group_ids.join(",")
    );

    dispatcher
        .default_message_handler(move |msg| match &msg.content {
            MessageContent::Text { text } => {
                println!(
                    "[Message-Handler] group={} sender={} text={}",
                    msg.group_id, msg.sender_id, text
                );
            }
            _ => {
                println!(
                    "[Message-Handler] group={} sender={} msg_type={} content={:?}",
                    msg.group_id, msg.sender_id, msg.msg_type, msg.content
                );
            }
        })
        .await;

    signal::ctrl_c()
        .await
        .map_err(|e| SdkError::Transport(format!("等待退出信号失败: {}", e)))?;
    dispatcher.shutdown();
    Ok(())
}
