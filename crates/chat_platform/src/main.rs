use chat_platform::{app_router, config::Config, db, ws, AppState};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_tracing(config: &Config) -> std::io::Result<WorkerGuard> {
    std::fs::create_dir_all(&config.logging.dir)?;

    let file_appender =
        tracing_appender::rolling::daily(&config.logging.dir, &config.logging.file_prefix);
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::from_default_env()
        .add_directive("chat_platform=debug".parse().unwrap());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_ansi(true),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false),
        )
        .init();

    Ok(file_guard)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    let _log_guard = init_tracing(&config)?;

    tracing::info!("════════════════════════════════════════════════════════════════");
    tracing::info!("🚀 群聊平台后端服务 启动中...");
    tracing::info!("════════════════════════════════════════════════════════════════");

    tracing::info!("📋 配置已加载");
    tracing::debug!(
        "服务器配置: host={}, port={}, workers={}",
        config.server.host,
        config.server.port,
        config.server.workers
    );
    tracing::debug!("数据库配置: DATABASE_URL={}", config.database.url);
    tracing::info!(
        "📝 日志落盘已启用: dir={}, file_prefix={} (按天切分)",
        config.logging.dir,
        config.logging.file_prefix
    );

    tracing::info!("🔌 正在初始化数据库连接池...");
    let db_pool = db::init_pool(&config.database)
        .await
        .expect("❌ 创建数据库连接池失败");
    tracing::info!("✅ 数据库连接池创建成功");

    tracing::info!("🔧 正在初始化数据库表结构...");
    db::init_schema(&db_pool)
        .await
        .expect("❌ 初始化数据库表结构失败");
    tracing::info!("✅ 数据库表结构初始化完成");

    tracing::info!("🛡️ 正在初始化超级管理员账号与Bot...");
    let seed_result = chat_platform::services::bootstrap::ensure_super_admin_account(
        &db_pool,
        &config.security.jwt_secret,
    )
    .await
    .expect("❌ 超级管理员初始化失败");
    tracing::info!(
        "✅ 超级管理员初始化完成: account={}, user_created={}, bot_created={}",
        seed_result.account,
        seed_result.user_created,
        seed_result.bot_created
    );
    if !seed_result.password_source_env {
        tracing::warn!(
            "⚠️ 未设置 SUPER_ADMIN_PASSWORD，当前使用默认初始密码，请在生产环境尽快通过环境变量覆盖"
        );
    }

    tracing::info!("📡 正在初始化 WebSocket 管理器...");
    let ws_manager = ws::WsManager::new();
    tracing::info!("✅ WebSocket 管理器初始化完成");

    let state = AppState {
        config: config.clone(),
        pool: db_pool,
        ws_manager,
    };

    let app = app_router(state);
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);

    tracing::info!("🌐 服务器将启动在 http://{}", bind_addr);
    tracing::info!("════════════════════════════════════════════════════════════════");

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    tracing::info!("════════════════════════════════════════════════════════════════");
    tracing::info!("👋 服务器已停止");
    tracing::info!("════════════════════════════════════════════════════════════════");

    Ok(())
}
