use actix_web::web;

pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod services;
pub mod utils;
pub mod ws;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/health", web::get().to(|| async { "OK" }))
        .route("/api/v1/auth/register", web::post().to(handlers::register))
        .route("/api/v1/auth/login", web::post().to(handlers::login))
        .route("/api/v1/users/delete", web::delete().to(handlers::delete_user))
        .route("/api/v1/bots", web::post().to(handlers::create_bot))
        .route("/api/v1/bots", web::get().to(handlers::list_bots))
        .route("/api/v1/bots/{bot_id}", web::get().to(handlers::get_bot))
        .route("/api/v1/bots/{bot_id}", web::put().to(handlers::update_bot))
        .route("/api/v1/bots/{bot_id}", web::delete().to(handlers::delete_bot))
        .route("/api/v1/groups", web::post().to(handlers::create_group))
        .route("/api/v1/groups", web::get().to(handlers::list_user_groups))
        .route("/api/v1/groups/{group_id}", web::get().to(handlers::get_group))
        .route(
            "/api/v1/groups/{group_id}",
            web::delete().to(handlers::delete_group),
        )
        .route(
            "/api/v1/groups/{group_id}/messages",
            web::get().to(handlers::get_group_messages),
        )
        .route("/api/v1/groups/join", web::post().to(handlers::join_group))
        .route(
            "/api/v1/groups/{group_id}/leave",
            web::delete().to(handlers::leave_group),
        )
        .route(
            "/api/v1/groups/{group_id}/members",
            web::get().to(handlers::list_group_members),
        )
        .route(
            "/api/v1/groups/{group_id}/members/{bot_id}",
            web::delete().to(handlers::remove_group_member),
        )
        .route("/api/v1/message/send", web::post().to(handlers::send_message))
        .route("/api/v1/file/upload", web::post().to(handlers::upload_file))
        .route(
            "/api/v1/file/download/{file_id}",
            web::get().to(handlers::download_file),
        )
        .route("/ws", web::get().to(handlers::ws_handler));
}
