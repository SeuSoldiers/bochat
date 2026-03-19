use actix_web::{web, App, HttpServer};
use chat_platform::{config::Config, db, handlers, ws};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("chat_platform=info".parse().unwrap()),
        )
        .init();

    // Load configuration
    let config = Config::from_env();
    tracing::info!("Starting Chat Platform Server...");
    tracing::info!("Config: {:?}", config);

    // Initialize database
    let db_pool = db::init_pool(&config.database)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Database pool created successfully");

    // Run migrations
    db::run_migrations(&db_pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations completed");

    // Create WebSocket manager
    let ws_manager = ws::WsManager::new();

    // Prepare app data
    let config_data = web::Data::new(config.clone());
    let pool_data = web::Data::new(db_pool);
    let ws_manager_data = web::Data::new(ws_manager);

    tracing::info!(
        "Server starting on {}:{}",
        config.server.host,
        config.server.port
    );

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(ws_manager_data.clone())
            // Health check endpoint
            .route("/health", web::get().to(|| async { "OK" }))
            // Auth endpoints
            .route("/api/v1/auth/register", web::post().to(handlers::register))
            // User endpoints
            .route("/api/v1/users/delete", web::delete().to(handlers::delete_user))
            // Bot endpoints
            .route("/api/v1/bots", web::post().to(handlers::create_bot))
            .route("/api/v1/bots", web::get().to(handlers::list_bots))
            .route("/api/v1/bots/{bot_id}", web::get().to(handlers::get_bot))
            .route("/api/v1/bots/{bot_id}", web::delete().to(handlers::delete_bot))
            // Message endpoints
            .route("/api/v1/message/send", web::post().to(handlers::send_message))
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
    .await
}

