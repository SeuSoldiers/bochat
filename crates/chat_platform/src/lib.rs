use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::CorsLayer;

pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod http;
pub mod middlewares;
pub mod models;
pub mod services;
pub mod utils;
pub mod ws;

#[derive(Clone)]
pub struct AppState {
    pub config: config::Config,
    pub pool: db::DbPool,
    pub ws_manager: ws::WsManager,
}

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/v1/auth/register", post(handlers::register))
        .route("/api/v1/auth/login", post(handlers::login))
        .route(
            "/api/v1/users/me",
            get(handlers::get_current_user).put(handlers::update_current_user),
        )
        .route("/api/v1/users/delete", delete(handlers::delete_user))
        .route(
            "/api/v1/bots",
            post(handlers::create_bot).get(handlers::list_bots),
        )
        .route(
            "/api/v1/bots/{bot_id}",
            get(handlers::get_bot)
                .put(handlers::update_bot)
                .delete(handlers::delete_bot),
        )
        .route(
            "/api/v1/groups",
            post(handlers::create_group).get(handlers::list_user_groups),
        )
        .route(
            "/api/v1/groups/{group_id}",
            get(handlers::get_group).delete(handlers::delete_group),
        )
        .route(
            "/api/v1/groups/{group_id}/messages",
            get(handlers::get_group_messages),
        )
        .route("/api/v1/groups/join", post(handlers::join_group))
        .route(
            "/api/v1/groups/{group_id}/leave",
            delete(handlers::leave_group),
        )
        .route(
            "/api/v1/groups/{group_id}/members",
            get(handlers::list_group_members),
        )
        .route(
            "/api/v1/groups/{group_id}/members/{bot_id}",
            delete(handlers::remove_group_member),
        )
        .route("/api/v1/message/send", post(handlers::send_message))
        .route("/api/v1/file/upload", post(handlers::upload_file))
        .route(
            "/api/v1/file/download/{file_id}/{filename}",
            get(handlers::download_file),
        )
        .route("/api/v1/file/{file_id}", delete(handlers::delete_file))
        .route("/ws", get(handlers::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
