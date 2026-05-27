use chat_platform::{
    cache::RedisMessageCache,
    config::DatabaseConfig,
    db,
    repositories::{MessageIdempotencyQuery, NewMessage},
    services::message_record_manager::MessageRecordManager,
};

fn test_database_config() -> DatabaseConfig {
    DatabaseConfig {
        url: std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://chat_user:chat_password@127.0.0.1:50032/chat_db".to_string()),
        max_connections: 2,
        min_connections: 1,
    }
}

fn test_redis_url() -> String {
    std::env::var("TEST_REDIS_URL")
        .unwrap_or_else(|_| "redis://:redis_password@127.0.0.1:50079/0".to_string())
}

#[tokio::test]
async fn message_record_manager_core_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let redis_client = redis::Client::open(test_redis_url().as_str()).expect("redis url");
    let redis_conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("redis conn");
    let cache = RedisMessageCache::new(redis_conn);
    let mgr = MessageRecordManager::new(pool.clone(), cache)
        .await
        .expect("new manager");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let user_id = format!("u_mrm_{}", unique);
    let account = format!("mrm_{}", unique);
    let bot_id = format!("b_mrm_{}", unique);
    let group_id = format!("g_mrm_{}", unique);

    sqlx::query(
        "INSERT INTO users (user_id, name, account, password_hash, id_number, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&user_id)
    .bind("MRM User")
    .bind(&account)
    .bind("hash")
    .bind(&account)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert user");

    sqlx::query(
        "INSERT INTO bots (bot_id, owner_id, name, status, token, secret, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(&bot_id)
    .bind(&user_id)
    .bind("MRM Bot")
    .bind("active")
    .bind(format!("{}:1:sig", bot_id))
    .bind("secret")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert bot");

    sqlx::query(
        "INSERT INTO groups (group_id, group_code, creator_id, name, description, avatar_url, is_public, status, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(&group_id)
    .bind(format!("MRM{}", unique))
    .bind(&user_id)
    .bind("MRM Group")
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(true)
    .bind("active")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert group");

    let msg_id = mgr.next_id();
    let new_msg = NewMessage {
        msg_id,
        group_id: &group_id,
        sender_id: &bot_id,
        content: r#"{"text":"hello mrm"}"#,
        msg_type: "text",
        idempotency_key: "idem-mrm-1",
        created_at: &now,
        sender_name: "MRM Bot",
        sender_avatar_url: None,
    };

    let sent = mgr.send_message(&new_msg).await.expect("send message");
    assert_eq!(sent.msg_id, msg_id);

    // Wait briefly for async persistence.
    for _ in 0..20 {
        if mgr.get_by_id(msg_id).await.expect("get by id").is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    let found = mgr
        .find_idempotent_message(&MessageIdempotencyQuery {
            sender_bot_id: &bot_id,
            group_id: &group_id,
            idempotency_key: "idem-mrm-1",
        })
        .await
        .expect("find idempotent")
        .expect("idempotent exists");
    assert_eq!(found.msg_id, msg_id);

    let by_id = mgr
        .get_by_id(msg_id)
        .await
        .expect("get by id")
        .expect("message exists");
    assert_eq!(by_id.msg_id, msg_id);

    let page = mgr
        .get_group_messages(&group_id, i64::MAX, 20)
        .await
        .expect("get group messages");
    assert!(!page.is_empty());

    mgr.remove_group_messages(&group_id)
        .await
        .expect("remove group messages");
    let page_after_clear = mgr
        .get_group_messages(&group_id, i64::MAX, 20)
        .await
        .expect("get group messages after clear");
    assert!(!page_after_clear.is_empty());
}
