use std::time::{SystemTime, UNIX_EPOCH};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use reqwest::StatusCode;
use serde_json::{Value, json};
use tokio::runtime::Runtime;

struct BenchContext {
    client: reqwest::Client,
    base_url: String,
    bot_token: String,
    group_id: String,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_millis()
}

async fn http_json(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Value {
    let mut req = client.request(method, url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    if let Some(v) = body {
        req = req.json(&v);
    }
    let resp = req.send().await.expect("http request");
    let status = resp.status();
    let payload: Value = resp.json().await.unwrap_or_else(|_| json!({}));
    if !status.is_success() {
        panic!("request failed: status={status} body={payload}");
    }
    payload
}

async fn setup_context(base_url: &str) -> BenchContext {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .expect("build reqwest client");
    let ts = now_millis();
    let account = format!("bench_user_{ts}");
    let password = "Passw0rd!";
    let group_code = format!("BENCH{}", ts % 1_000_000);

    let register_url = format!("{base_url}/api/v1/auth/register");
    let login_url = format!("{base_url}/api/v1/auth/login");
    let bots_url = format!("{base_url}/api/v1/bots");
    let groups_url = format!("{base_url}/api/v1/groups");

    let reg_resp = client
        .post(&register_url)
        .json(&json!({
            "name": format!("bench-{ts}"),
            "account": account,
            "password": password
        }))
        .send()
        .await
        .expect("register request");

    let reg_status = reg_resp.status();
    if reg_status != StatusCode::CREATED && reg_status != StatusCode::CONFLICT {
        let body = reg_resp.text().await.unwrap_or_default();
        panic!("register failed: {reg_status} {body}");
    }

    let login = http_json(
        &client,
        reqwest::Method::POST,
        &login_url,
        None,
        Some(json!({
            "account": format!("bench_user_{ts}"),
            "password": password
        })),
    )
    .await;
    let user_token = login["token"].as_str().expect("user token").to_string();

    let bots = http_json(
        &client,
        reqwest::Method::GET,
        &bots_url,
        Some(&user_token),
        None,
    )
    .await;
    let first_bot = bots["bots"][0].clone();
    let bot_token = first_bot["token"]
        .as_str()
        .expect("bot token")
        .to_string();
    let bot_id = first_bot["bot_id"].as_str().expect("bot id");

    let group = http_json(
        &client,
        reqwest::Method::POST,
        &groups_url,
        Some(&user_token),
        Some(json!({
            "name": format!("bench-group-{ts}"),
            "description": "criterion benchmark group",
            "group_code": group_code,
            "bot_id": bot_id
        })),
    )
    .await;
    let group_id = group["group_id"].as_str().expect("group id").to_string();

    BenchContext {
        client,
        base_url: base_url.to_string(),
        bot_token,
        group_id,
    }
}

async fn send_batch(ctx: &BenchContext, batch_size: usize) {
    let url = format!("{}/api/v1/message/send", ctx.base_url);
    let mut tasks = Vec::with_capacity(batch_size);
    for i in 0..batch_size {
        let req = ctx
            .client
            .post(&url)
            .bearer_auth(&ctx.bot_token)
            .json(&json!({
                "group_id": ctx.group_id,
                "content": { "text": format!("bench message {i}") },
                "msg_type": "text",
                "idempotency_key": format!("bench-{}-{i}", now_millis()),
            }));
        tasks.push(tokio::spawn(async move {
            let resp = req.send().await.expect("send message request");
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                panic!("message send failed: {status} {body}");
            }
        }));
    }
    for task in tasks {
        task.await.expect("join send task");
    }
}

fn bench_message_send_batch(c: &mut Criterion) {
    let base_url = std::env::var("BOCHAT_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let rt = Runtime::new().expect("tokio runtime");
    let ctx = rt.block_on(setup_context(&base_url));

    let mut group = c.benchmark_group("message_send_batch");
    for size in [10usize, 50, 100, 200] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &batch_size| {
            b.to_async(&rt).iter(|| send_batch(&ctx, batch_size));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_message_send_batch);
criterion_main!(benches);
