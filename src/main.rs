use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use chat_platform::{config::Config, db, handlers, ws};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("chat_platform=debug".parse().unwrap()),
        )
        .init();

    tracing::info!("════════════════════════════════════════════════════════════════");
    tracing::info!("🚀 群聊平台后端服务 启动中...");
    tracing::info!("════════════════════════════════════════════════════════════════");

    // 加载配置
    let config = Config::from_env();
    tracing::info!("📋 配置已加载");
    tracing::debug!("服务器配置: host={}, port={}, workers={}",
        config.server.host,
        config.server.port,
        config.server.workers
    );
    tracing::debug!("数据库配置: DATABASE_URL={}", config.database.url);

    // 初始化数据库
    tracing::info!("🔌 正在初始化数据库连接池...");
    let db_pool = db::init_pool(&config.database)
        .await
        .expect("❌ 创建数据库连接池失败");

    tracing::info!("✅ 数据库连接池创建成功");

    // 运行迁移
    tracing::info!("🔄 正在运行数据库迁移...");
    db::run_migrations(&db_pool)
        .await
        .expect("❌ 数据库迁移失败");

    tracing::info!("✅ 数据库迁移完成");

    // 创建 WebSocket 管理器
    tracing::info!("📡 正在初始化 WebSocket 管理器...");
    let ws_manager = ws::WsManager::new();
    tracing::info!("✅ WebSocket 管理器初始化完成");

    // 准备应用数据
    let config_data = web::Data::new(config.clone());
    let pool_data = web::Data::new(db_pool);
    let ws_manager_data = web::Data::new(ws_manager);

    tracing::info!(
        "🌐 服务器将启动在 http://{}:{}",
        config.server.host,
        config.server.port
    );
    tracing::info!("════════════════════════════════════════════════════════════════");

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(ws_manager_data.clone())
            // Health check endpoint
            .route("/health", web::get().to(|| async { "OK" }))
            // Auth endpoints
            .route("/api/v1/auth/register", web::post().to(handlers::register))
            .route("/api/v1/auth/login", web::post().to(handlers::login))
            // User endpoints
            .route(
                "/api/v1/users/delete",
                web::delete().to(handlers::delete_user),
            )
            // Bot endpoints
            .route("/api/v1/bots", web::post().to(handlers::create_bot))
            .route("/api/v1/bots", web::get().to(handlers::list_bots))
            .route("/api/v1/bots/{bot_id}", web::get().to(handlers::get_bot))
            .route("/api/v1/bots/{bot_id}", web::delete().to(handlers::delete_bot))
            // Group endpoints
            .route("/api/v1/groups", web::post().to(handlers::create_group))
            .route("/api/v1/groups", web::get().to(handlers::list_user_groups))
            .route(
                "/api/v1/groups/{group_id}",
                web::get().to(handlers::get_group),
            )
            .route(
                "/api/v1/groups/{group_id}",
                web::delete().to(handlers::delete_group),
            )
            .route(
                "/api/v1/groups/{group_id}/messages",
                web::get().to(handlers::get_group_messages),
            )
            .route(
                "/api/v1/groups/join",
                web::post().to(handlers::join_group),
            )
            .route(
                "/api/v1/groups/{group_id}/leave",
                web::delete().to(handlers::leave_group),
            )
            .route(
                "/api/v1/groups/{group_id}/members",
                web::get().to(handlers::list_group_members),
            )
            // Message endpoints
            .route(
                "/api/v1/message/send",
                web::post().to(handlers::send_message),
            )
            // File endpoints
            .route("/api/v1/file/upload", web::post().to(handlers::upload_file))
            .route(
                "/api/v1/file/download/{file_id}",
                web::get().to(handlers::download_file),
            )
            // WebSocket endpoint
            .route("/ws", web::get().to(handlers::ws_handler))
    })
    .workers(config.server.workers)
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await?;

    tracing::info!("════════════════════════════════════════════════════════════════");
    tracing::info!("👋 服务器已停止");
    tracing::info!("════════════════════════════════════════════════════════════════");

    Ok(())
}
