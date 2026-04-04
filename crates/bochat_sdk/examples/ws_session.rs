use bochat_sdk::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

#[tokio::main(flavor = "current_thread")]
async fn main() -> SdkResult<()> {
    let client = BochatClient::builder("http://127.0.0.1:8080").build()?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let account = format!("sdk_user_{}", ts);
    let password = "Passw0rd";
    let nickname = format!("SDK用户{}", ts % 10000);

    match client
        .auth()
        .register()
        .account(account.clone())
        .password(password)
        .nickname(nickname)
        .send()
        .await
    {
        Ok(resp) => {
            println!(
                "注册成功: token={}",
                &resp.token[..20.min(resp.token.len())]
            );
        }
        Err(SdkError::Api { code, .. }) if code == "account_conflict" => {
            println!("账号已存在，回退到登录流程");
            let _ = client
                .auth()
                .login()
                .account(account.clone())
                .password(password)
                .send()
                .await?;
        }
        Err(err) => return Err(err),
    }

    let bots = client.bots().list().await?;
    let first_bot = bots.first().expect("需要至少一个 Bot");
    client.set_bot_token(Some(first_bot.token.clone())).await;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut group_ids = Vec::new();
    for i in 0..3 {
        let group = client
            .groups()
            .create(CreateGroupRequest {
                name: format!("SDK-WS-Group-{}-{}", ts % 10000, i),
                description: Some("用于 ws 分发示例".to_string()),
                group_code: Some(format!("WS{}{}", ts % 10000, i)),
                bot_id: Some(first_bot.bot_id.clone()),
            })
            .await?;
        group_ids.push(group.group_id);
    }

    let session = client
        .ws()
        .heartbeat_interval(Duration::from_secs(15))
        .heartbeat_timeout(Duration::from_secs(45))
        .reconnect_max_attempts(20)
        .build()
        .await?;

    let handle = session.spawn().await?;
    let mut dispatcher = handle.into_dispatcher();

    let conn = dispatcher.wait_connection_payload().await?;
    println!(
        "连接成功: bot={} 可用群={}",
        conn.bot_name,
        conn.group_ids.join(",")
    );

    let handled = Arc::new(AtomicUsize::new(0));
    let handled_a = Arc::clone(&handled);
    let handled_b = Arc::clone(&handled);
    let handled_default = Arc::clone(&handled);

    dispatcher
        .default_message_handler(move |msg| {
            println!(
                "[Message-Handler] group={} sender={} content={:?}",
                msg.group_id, msg.sender_id, msg.content
            );
            handled_default.fetch_add(1, Ordering::Relaxed);
        })
        .await
        .group_message_handler(group_ids[0].clone(), move |msg| {
            println!(
                "[Handler-A] group={} sender={} content={:?}",
                msg.group_id, msg.sender_id, msg.content
            );
            handled_a.fetch_add(1, Ordering::Relaxed);
        })
        .await
        .group_message_handler(group_ids[1].clone(), move |msg| {
            println!(
                "[Handler-B] group={} sender={} content={:?}",
                msg.group_id, msg.sender_id, msg.content
            );
            handled_b.fetch_add(1, Ordering::Relaxed);
        })
        .await;

    for (idx, gid) in group_ids.iter().enumerate() {
        let _ = client
            .messages()
            .send_text(gid, format!("来自 ws 分发示例的消息 {}", idx + 1))
            .await?;
    }

    let target = group_ids.len();
    for _ in 0..20 {
        if handled.load(Ordering::Relaxed) >= target {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }

    dispatcher.shutdown();

    for group_id in &group_ids {
        client.groups().delete(group_id).await?;
        println!("群聊资源清理完成: {}", group_id);
    }

    let bots = client.bots().list().await?;
    for bot in bots {
        client.bots().delete(&bot.bot_id).await?;
        println!("Bot资源清理完成: {}", bot.bot_id);
    }

    client.auth().delete_account().await?;
    println!("用户账号清理完成");

    Ok(())
}
