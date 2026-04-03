use bochat_sdk::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

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

    let group = client
        .groups()
        .create(CreateGroupRequest {
            name: format!("SDK测试群-{}", ts % 10000),
            description: Some("bochat_sdk 基础流程自动创建".to_string()),
            group_code: Some(format!("SDK{}", ts % 100000)),
            bot_id: Some(first_bot.bot_id.clone()),
        })
        .await?;

    println!("群聊创建成功: group_id={}", group.group_id);

    let message = client
        .messages()
        .send_text(
            &group.group_id,
            "你好，来自 bochat_sdk",
            format!("example-msg-{}", ts),
        )
        .await?;

    println!("发送成功: msg_id={}", message.msg_id);

    let uploaded = client
        .files()
        .upload_bytes("hello.txt", b"hello from sdk".to_vec(), Some("text/plain"))
        .await?;

    println!("文件上传成功: {}", uploaded.url);

    client.files().delete(&uploaded.file_id).await?;
    println!("文件资源清理完成: {}", uploaded.file_id);

    client.groups().delete(&group.group_id).await?;
    println!("群聊资源清理完成: {}", group.group_id);

    let bots = client.bots().list().await?;
    for bot in bots {
        client.bots().delete(&bot.bot_id).await?;
        println!("Bot资源清理完成: {}", bot.bot_id);
    }

    client.auth().delete_account().await?;
    println!("用户账号清理完成");

    Ok(())
}
