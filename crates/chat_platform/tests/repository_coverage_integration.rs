use chat_platform::{
    config::DatabaseConfig,
    db,
    repositories::{
        AuthzRepository, BotRepository, FileRepository, FileScanRepository, FileScanResultUpdate,
        GroupMemberLink, GroupMessagesQuery, GroupMessagesPage, GroupRepository,
        MessageIdempotencyQuery, MessageRepository, NewBot, NewFile, NewGroup, NewGroupMember,
        NewFileUploader, NewMessage, NewNotification, NewPendingFileScan, NewUser,
        NotificationListFilter, NotificationRepository, UpdateGroupProfile, UploaderRelation,
        UserFilesPage, UserRepository,
    },
    services::{
        bootstrap::ensure_super_admin_account,
        file::FileService,
        message::MessageService,
        notification::{NotificationRecord, create_best_effort as create_notification_best_effort},
    },
};
use serde_json::json;

fn test_database_config() -> DatabaseConfig {
    DatabaseConfig {
        url: std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://chat_user:chat_password@127.0.0.1:50032/chat_db".to_string()),
        max_connections: 2,
        min_connections: 1,
    }
}

#[tokio::test]
async fn notification_repository_crud_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let user_id = format!("u_repo_notify_{}", unique);
    let account = format!("notify_{}", unique);
    let notification_id = format!("n_repo_{}", unique);

    sqlx::query(
        "INSERT INTO users (user_id, name, account, password_hash, id_number, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&user_id)
    .bind("Repo User")
    .bind(&account)
    .bind("hash")
    .bind(&account)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert user");

    NotificationRepository::insert(
        &pool,
        &NewNotification {
            notification_id: &notification_id,
            recipient_user_id: &user_id,
            kind: "system_info",
            title: "hello",
            content: "world",
            requires_action: true,
            action_payload: Some(r#"{"request_id":"r1"}"#),
            related_request_id: Some("r1"),
            related_group_id: None,
            related_bot_id: None,
            now: &now,
        },
    )
    .await
    .expect("insert notification");

    let list = NotificationRepository::list(
        &pool,
        &user_id,
        &NotificationListFilter {
            status: Some("all"),
            kind: Some("system_info"),
            limit: 10,
            offset: 0,
        },
    )
    .await
    .expect("list notifications");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].notification_id, notification_id);

    let stats = NotificationRepository::stats(&pool, &user_id)
        .await
        .expect("stats");
    assert_eq!(stats.unread_count, 1);
    assert_eq!(stats.pending_count, 1);

    let found = NotificationRepository::find_by_id(&pool, &user_id, &notification_id)
        .await
        .expect("find by id")
        .expect("exists");
    assert!(found.requires_action);
    assert!(!found.is_read);

    let marked = NotificationRepository::mark_read(&pool, &user_id, &notification_id, &now)
        .await
        .expect("mark read");
    assert!(marked);

    let resolved = NotificationRepository::resolve(&pool, &user_id, &notification_id, &now)
        .await
        .expect("resolve");
    assert!(resolved);

    NotificationRepository::resolve_by_request_id(&pool, "r1", &now)
        .await
        .expect("resolve by request id");

    create_notification_best_effort(
        &pool,
        NotificationRecord {
            recipient_user_id: &user_id,
            kind: "system_info",
            title: "from service",
            content: "created by service",
            requires_action: false,
            action_payload: Some(json!({"from":"service"})),
            related_request_id: None,
            related_group_id: None,
            related_bot_id: None,
        },
    )
    .await;
    let list_after_service = NotificationRepository::list(
        &pool,
        &user_id,
        &NotificationListFilter {
            status: Some("all"),
            kind: Some("system_info"),
            limit: 50,
            offset: 0,
        },
    )
    .await
    .expect("list after service");
    assert!(list_after_service.len() >= 2);
}

#[tokio::test]
async fn file_repository_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let user_id = format!("u_repo_file_{}", unique);
    let account = format!("file_{}", unique);
    let bot_id = format!("b_repo_file_{}", unique);
    let file_id = format!("f_repo_file_{}", unique);
    let content_hash = format!("hash_{}", unique);

    sqlx::query(
        "INSERT INTO users (user_id, name, account, password_hash, id_number, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&user_id)
    .bind("File User")
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
    .bind("Repo Bot")
    .bind("active")
    .bind(format!("{}:1:sig", bot_id))
    .bind("secret")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert bot");

    FileRepository::insert_file(
        &pool,
        &NewFile {
            file_id: &file_id,
            owner_id: &bot_id,
            content_hash: &content_hash,
            filename: "repo.txt",
            size: 5,
            mime_type: "text/plain",
            storage_path: "/tmp/repo.txt",
            created_at: &now,
        },
    )
    .await
    .expect("insert file");

    FileRepository::add_uploader_ignore(
        &pool,
        &NewFileUploader {
            file_id: &file_id,
            uploader_id: &bot_id,
            created_at: &now,
        },
    )
    .await
    .expect("add uploader");

    let found_by_id = FileRepository::find_by_id(&pool, &file_id)
        .await
        .expect("find by id")
        .expect("exists");
    assert_eq!(found_by_id.filename, "repo.txt");

    let found_by_hash = FileRepository::find_by_content_hash(&pool, &content_hash)
        .await
        .expect("find by hash")
        .expect("exists");
    assert_eq!(found_by_hash.file_id, file_id);

    let exists = FileRepository::exists_file_id(&pool, &file_id)
        .await
        .expect("exists")
        .expect("exists val");
    assert_eq!(exists, file_id);

    let listed = FileRepository::list_by_owner_via_uploaders(
        &pool,
        &UserFilesPage {
            user_id: &user_id,
            limit: 10,
            offset: 0,
        },
    )
    .await
    .expect("list files");
    assert_eq!(listed.len(), 1);

    let relation_exists = FileRepository::uploader_relation_exists(
        &pool,
        &UploaderRelation {
            file_id: &file_id,
            uploader_id: &bot_id,
        },
    )
    .await
    .expect("relation exists");
    assert!(relation_exists);

    let by_id = FileService::get_file_by_id(&pool, &file_id)
        .await
        .expect("service get by id");
    assert_eq!(by_id.file_id, file_id);

    let files_before_remove = FileService::get_user_files(&pool, &user_id, 10, 0)
        .await
        .expect("service get user files before remove");
    assert_eq!(files_before_remove.len(), 1);

    FileRepository::remove_uploader(
        &pool,
        &UploaderRelation {
            file_id: &file_id,
            uploader_id: &bot_id,
        },
    )
    .await
    .expect("remove uploader");

    let relation_exists_after = FileRepository::uploader_relation_exists(
        &pool,
        &UploaderRelation {
            file_id: &file_id,
            uploader_id: &bot_id,
        },
    )
    .await
    .expect("relation exists after");
    assert!(!relation_exists_after);
}

#[tokio::test]
async fn bootstrap_super_admin_seed_is_idempotent() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let unique = chrono::Utc::now().timestamp_millis();
    let account = format!("super_admin_{}", unique);
    let password = format!("Admin{}!", unique);

    unsafe {
        std::env::set_var("SUPER_ADMIN_ACCOUNT", &account);
        std::env::set_var("SUPER_ADMIN_PASSWORD", &password);
    }

    let first = ensure_super_admin_account(&pool, "seed-secret")
        .await
        .expect("first seed");
    assert!(first.user_created);
    assert!(first.bot_created);
    assert_eq!(first.account, account);
    assert!(first.password_source_env);

    let second = ensure_super_admin_account(&pool, "seed-secret")
        .await
        .expect("second seed");
    assert!(!second.user_created);
    assert!(!second.bot_created);
    assert_eq!(second.account, account);
}

#[tokio::test]
async fn authz_repository_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let account = format!("authz_{}", unique);
    let user_id = format!("u_authz_{}", unique);
    let bot_id = format!("b_authz_{}", unique);

    sqlx::query(
        "INSERT INTO users (user_id, name, account, password_hash, id_number, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&user_id)
    .bind("Authz User")
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
    .bind("Authz Bot")
    .bind("active")
    .bind(format!("{}:1:sig", bot_id))
    .bind("secret")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert bot");

    assert!(AuthzRepository::user_exists(&pool, &user_id)
        .await
        .expect("user exists"));
    assert!(AuthzRepository::user_matches_account(&pool, &user_id, &account)
        .await
        .expect("user account match"));
    assert!(AuthzRepository::bot_owned_by_account(&pool, &bot_id, &account)
        .await
        .expect("bot owned by account"));
    let bot_ids = AuthzRepository::list_active_bot_ids_by_account(&pool, &account)
        .await
        .expect("list active bots");
    assert_eq!(bot_ids, vec![bot_id.clone()]);
    let user_ids = AuthzRepository::list_user_ids_by_account(&pool, &account)
        .await
        .expect("list user ids");
    assert_eq!(user_ids, vec![user_id]);
}

#[tokio::test]
async fn message_repository_and_service_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let user_id = format!("u_msg_repo_{}", unique);
    let account = format!("msg_repo_{}", unique);
    let bot_id = format!("b_msg_repo_{}", unique);
    let group_id = format!("g_msg_repo_{}", unique);

    sqlx::query(
        "INSERT INTO users (user_id, name, account, password_hash, id_number, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&user_id)
    .bind("Msg User")
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
    .bind("Msg Bot")
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
    .bind(format!("GMSG{}", unique))
    .bind(&user_id)
    .bind("Msg Group")
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(true)
    .bind("active")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert group");

    sqlx::query("INSERT INTO group_members (group_id, member_id, member_type, joined_at) VALUES ($1,$2,$3,$4)")
        .bind(&group_id)
        .bind(&bot_id)
        .bind("bot")
        .bind(&now)
        .execute(&pool)
        .await
        .expect("insert member");

    let inserted = MessageRepository::insert_message_returning(
        &pool,
        &NewMessage {
            msg_id: unique,
            group_id: &group_id,
            sender_id: &bot_id,
            content: r#"{"text":"hello"}"#,
            msg_type: "text",
            idempotency_key: "idem-1",
            created_at: &now,
            sender_name: "Msg Bot",
            sender_avatar_url: None,
        },
    )
    .await
    .expect("insert message");
    assert_eq!(inserted.msg_id, unique);

    let by_id = MessageRepository::find_by_id(&pool, unique)
        .await
        .expect("find by id")
        .expect("exists");
    assert_eq!(by_id.group_id, group_id);

    let enriched = MessageRepository::find_by_id_enriched(&pool, unique)
        .await
        .expect("find by id enriched")
        .expect("exists");
    assert_eq!(enriched.sender_id, bot_id);

    let all = MessageRepository::list_all_enriched_by_group(&pool, &group_id)
        .await
        .expect("list enriched");
    assert_eq!(all.len(), 1);

    let idem = MessageRepository::find_idempotent_message(
        &pool,
        &MessageIdempotencyQuery {
            sender_bot_id: &bot_id,
            group_id: &group_id,
            idempotency_key: "idem-1",
        },
    )
    .await
    .expect("find idempotent")
    .expect("exists");
    assert_eq!(idem.msg_id, unique);

    let listed = MessageRepository::list_by_group_before(
        &pool,
        &GroupMessagesPage {
            group_id: &group_id,
            base_id: unique + 10,
            limit: 20,
        },
    )
    .await
    .expect("list by group before");
    assert_eq!(listed.len(), 1);

    let member_ids = MessageRepository::list_member_bot_ids(&pool, &group_id)
        .await
        .expect("list member bot ids");
    assert_eq!(member_ids, vec![bot_id.clone()]);

    let svc_by_id = MessageService::get_message_by_id(&pool, unique)
        .await
        .expect("service get by id");
    assert_eq!(svc_by_id.msg_id, unique);
    let svc_list = MessageService::get_messages_for_group_before(&pool, &group_id, unique + 10, 10)
        .await
        .expect("service list");
    assert_eq!(svc_list.len(), 1);

    MessageService::on_message_persisted(&pool, unique, "text", &json!({"text":"hello"}))
        .await
        .expect("on message persisted");
    assert_eq!(svc_by_id.content_as_json().expect("content json"), json!({"text":"hello"}));
}

#[tokio::test]
async fn file_scan_repository_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let user_id = format!("u_scan_repo_{}", unique);
    let account = format!("scan_repo_{}", unique);
    let bot_id = format!("b_scan_repo_{}", unique);
    let file_id = format!("f_scan_repo_{}", unique);

    sqlx::query(
        "INSERT INTO users (user_id, name, account, password_hash, id_number, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&user_id)
    .bind("Scan User")
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
    .bind("Scan Bot")
    .bind("active")
    .bind(format!("{}:1:sig", bot_id))
    .bind("secret")
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert bot");

    sqlx::query(
        "INSERT INTO files (file_id, owner_id, content_hash, filename, size, mime_type, storage_path, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(&file_id)
    .bind(&bot_id)
    .bind(format!("hash_{}", unique))
    .bind("scan.bin")
    .bind(16_i64)
    .bind("application/octet-stream")
    .bind("/tmp/scan.bin")
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert file");

    let inserted = FileScanRepository::insert_pending_ignore(
        &pool,
        &NewPendingFileScan {
            file_id: &file_id,
            now: &now,
        },
    )
    .await
    .expect("insert pending");
    assert!(inserted);

    let inserted_again = FileScanRepository::insert_pending_ignore(
        &pool,
        &NewPendingFileScan {
            file_id: &file_id,
            now: &now,
        },
    )
    .await
    .expect("insert pending again");
    assert!(!inserted_again);

    let found = FileScanRepository::find_by_file_id(&pool, &file_id)
        .await
        .expect("find scan")
        .expect("exists");
    assert_eq!(found.status, "pending");

    FileScanRepository::update_result(
        &pool,
        &FileScanResultUpdate {
            file_id: &file_id,
            status: "suspicious",
            risk_level: "high",
            scan_result: Some(r#"{"engine":"t"}"#),
            scanned_at: &now,
            now: &now,
        },
    )
    .await
    .expect("update result");

    let updated = FileScanRepository::find_by_file_id(&pool, &file_id)
        .await
        .expect("find updated")
        .expect("exists");
    assert_eq!(updated.status, "suspicious");
    assert_eq!(updated.risk_level, "high");
}

#[tokio::test]
async fn user_repository_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let user_id = format!("u_user_repo_{}", unique);
    let account = format!("user_repo_{}", unique);

    UserRepository::insert_user(
        &pool,
        &NewUser {
            user_id: &user_id,
            name: "User Repo",
            account: &account,
            password_hash: "hash1",
            id_number: &account,
            avatar_url: None,
            now: &now,
        },
    )
    .await
    .expect("insert user");

    let found = UserRepository::find_by_id(&pool, &user_id)
        .await
        .expect("find by id")
        .expect("exists");
    assert_eq!(found.name, "User Repo");

    let login_row = UserRepository::find_login_by_account(&pool, &account)
        .await
        .expect("find login")
        .expect("exists");
    assert_eq!(login_row.user_id, user_id);
    assert_eq!(login_row.name, "User Repo");
    assert_eq!(login_row.password_hash.as_deref(), Some("hash1"));

    UserRepository::update_profile(
        &pool,
        &user_id,
        "User Repo Updated",
        Some("http://avatar"),
        Some("hash2"),
        &now,
    )
    .await
    .expect("update profile");
    let updated_login = UserRepository::find_login_by_account(&pool, &account)
        .await
        .expect("find updated login")
        .expect("exists");
    assert_eq!(updated_login.password_hash.as_deref(), Some("hash2"));

    let bot_id = format!("b_user_repo_{}", unique);
    BotRepository::insert(
        &pool,
        &NewBot {
            bot_id: &bot_id,
            owner_id: &user_id,
            name: "User Repo Bot",
            description: None,
            avatar_url: None,
            status: "active",
            token: &format!("{}:1:sig", bot_id),
            secret: "secret",
            now: &now,
        },
    )
    .await
    .expect("insert bot");

    UserRepository::delete_bots_by_owner(&pool, &user_id)
        .await
        .expect("delete bots by owner");
    assert!(BotRepository::find_by_id(&pool, &bot_id)
        .await
        .expect("find bot")
        .is_none());

    UserRepository::delete_user_by_id(&pool, &user_id)
        .await
        .expect("delete user");
    assert!(UserRepository::find_by_id(&pool, &user_id)
        .await
        .expect("find deleted user")
        .is_none());
}

#[tokio::test]
async fn bot_repository_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let user_id = format!("u_bot_repo_{}", unique);
    let account = format!("bot_repo_{}", unique);
    let bot_id = format!("b_bot_repo_{}", unique);

    sqlx::query(
        "INSERT INTO users (user_id, name, account, password_hash, id_number, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&user_id)
    .bind("Bot Repo User")
    .bind(&account)
    .bind("hash")
    .bind(&account)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert user");

    BotRepository::insert(
        &pool,
        &NewBot {
            bot_id: &bot_id,
            owner_id: &user_id,
            name: "Bot Repo",
            description: Some("desc"),
            avatar_url: Some("http://avatar"),
            status: "active",
            token: &format!("{}:1:sig", bot_id),
            secret: "secret",
            now: &now,
        },
    )
    .await
    .expect("insert bot");

    let found = BotRepository::find_by_id(&pool, &bot_id)
        .await
        .expect("find by id")
        .expect("exists");
    assert_eq!(found.name, "Bot Repo");
    let required = BotRepository::find_required_by_id(&pool, &bot_id)
        .await
        .expect("find required");
    assert_eq!(required.bot_id, bot_id);
    let missing_required = BotRepository::find_required_by_id(&pool, "b_missing")
        .await
        .expect_err("missing should fail");
    assert!(matches!(missing_required, chat_platform::error::AppError::BotNotFound));

    let all = BotRepository::list_all(&pool).await.expect("list all");
    assert!(!all.is_empty());
    let by_owner = BotRepository::list_by_owner(&pool, &user_id)
        .await
        .expect("list by owner");
    assert_eq!(by_owner.len(), 1);

    let secret_status = BotRepository::find_secret_and_status(&pool, &bot_id)
        .await
        .expect("secret status")
        .expect("exists");
    assert_eq!(secret_status.1, "active");

    let default_active = BotRepository::find_default_active_bot_id(&pool, &user_id)
        .await
        .expect("default active")
        .expect("exists");
    assert_eq!(default_active, bot_id);

    BotRepository::update_profile(
        &pool,
        &bot_id,
        "Bot Repo Updated",
        Some("updated"),
        Some("http://updated"),
        &now,
    )
    .await
    .expect("update profile");
    let updated = BotRepository::find_by_id(&pool, &bot_id)
        .await
        .expect("find updated")
        .expect("exists");
    assert_eq!(updated.name, "Bot Repo Updated");

    BotRepository::delete_by_id(&pool, &bot_id)
        .await
        .expect("delete bot");
    assert!(BotRepository::find_by_id(&pool, &bot_id)
        .await
        .expect("find deleted")
        .is_none());
}

#[tokio::test]
async fn group_repository_paths_work() {
    let cfg = test_database_config();
    let pool = db::init_pool(&cfg).await.expect("init db pool");
    db::init_schema(&pool).await.expect("init schema");

    let now = chrono::Utc::now().to_rfc3339();
    let unique = chrono::Utc::now().timestamp_millis();
    let user_id = format!("u_group_repo_{}", unique);
    let account = format!("group_repo_{}", unique);
    let bot_id = format!("b_group_repo_{}", unique);
    let group_id = format!("g_group_repo_{}", unique);
    let group_code = format!("GRC{}", unique);

    sqlx::query(
        "INSERT INTO users (user_id, name, account, password_hash, id_number, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&user_id)
    .bind("Group Repo User")
    .bind(&account)
    .bind("hash")
    .bind(&account)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert user");

    BotRepository::insert(
        &pool,
        &NewBot {
            bot_id: &bot_id,
            owner_id: &user_id,
            name: "Group Repo Bot",
            description: None,
            avatar_url: None,
            status: "active",
            token: &format!("{}:1:sig", bot_id),
            secret: "secret",
            now: &now,
        },
    )
    .await
    .expect("insert bot");

    GroupRepository::insert_group(
        &pool,
        &NewGroup {
            group_id: &group_id,
            group_code: Some(&group_code),
            creator_id: &user_id,
            name: "Group Repo",
            description: Some("desc"),
            avatar_url: None,
            is_public: true,
            status: "active",
            now: &now,
        },
    )
    .await
    .expect("insert group");

    GroupRepository::add_member_ignore(
        &pool,
        &NewGroupMember {
            group_id: &group_id,
            member_id: &bot_id,
            member_type: "bot",
            joined_at: &now,
        },
    )
    .await
    .expect("add member");

    let all = GroupRepository::list_all(&pool).await.expect("list all");
    assert!(!all.is_empty());
    let visible = GroupRepository::list_visible_by_user(&pool, &user_id)
        .await
        .expect("list visible");
    assert!(!visible.is_empty());
    let by_bot = GroupRepository::list_by_bot_member(&pool, &bot_id)
        .await
        .expect("list by bot");
    assert_eq!(by_bot.len(), 1);

    let found = GroupRepository::find_by_id(&pool, &group_id)
        .await
        .expect("find by id")
        .expect("exists");
    assert_eq!(found.group_id, group_id);

    GroupRepository::update_profile(
        &pool,
        &UpdateGroupProfile {
            group_id: &group_id,
            name: "Group Repo Updated",
            group_code: Some(&group_code),
            description: Some("new"),
            avatar_url: Some("http://avatar"),
            is_public: false,
            updated_at: &now,
        },
    )
    .await
    .expect("update profile");

    let by_code = GroupRepository::find_group_id_by_code(&pool, &group_code)
        .await
        .expect("find by code")
        .expect("exists");
    assert_eq!(by_code, group_id);
    let prefix = &group_code[..3];
    let by_prefix = GroupRepository::find_by_code_prefix(&pool, prefix)
        .await
        .expect("find by prefix");
    assert!(!by_prefix.is_empty());
    let public_groups = GroupRepository::list_public(&pool).await.expect("list public");
    assert!(public_groups.iter().all(|g| g.is_public));

    assert!(GroupRepository::exists(&pool, &group_id).await.expect("exists"));
    assert!(GroupRepository::is_member(
        &pool,
        &GroupMemberLink {
            group_id: &group_id,
            member_id: &bot_id,
        }
    )
    .await
    .expect("is member"));

    // Insert one message to cover message-list related repository functions.
    let msg_id = unique + 1;
    sqlx::query(
        "INSERT INTO messages (msg_id, group_id, sender_id, content, msg_type, idempotency_key, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(msg_id)
    .bind(&group_id)
    .bind(&bot_id)
    .bind(r#"{"text":"hello"}"#)
    .bind("text")
    .bind("idem-gr-1")
    .bind(&now)
    .execute(&pool)
    .await
    .expect("insert message");

    let list_messages = GroupRepository::list_messages(
        &pool,
        &GroupMessagesQuery {
            group_id: &group_id,
            base_id: msg_id + 10,
            limit: 10,
        },
    )
    .await
    .expect("list messages");
    assert_eq!(list_messages.len(), 1);

    assert!(GroupRepository::can_user_view_members(&pool, &group_id, &user_id)
        .await
        .expect("can view members"));
    let members = GroupRepository::list_members(&pool, &group_id)
        .await
        .expect("list members");
    assert_eq!(members.len(), 1);

    let all_ids = GroupRepository::list_all_group_ids(&pool)
        .await
        .expect("list all ids");
    assert!(all_ids.contains(&group_id));
    let member_ids = GroupRepository::list_group_ids_by_member(&pool, &bot_id)
        .await
        .expect("list ids by member");
    assert_eq!(member_ids, vec![group_id.clone()]);

    GroupRepository::delete_messages_by_group(&pool, &group_id)
        .await
        .expect("delete messages by group");
    GroupRepository::remove_member(
        &pool,
        &GroupMemberLink {
            group_id: &group_id,
            member_id: &bot_id,
        },
    )
    .await
    .expect("remove member");
    GroupRepository::delete_members_by_group(&pool, &group_id)
        .await
        .expect("delete members by group");
    GroupRepository::delete_group(&pool, &group_id)
        .await
        .expect("delete group");
    assert!(!GroupRepository::exists(&pool, &group_id)
        .await
        .expect("exists after delete"));
}
