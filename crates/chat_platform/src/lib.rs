use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use services::message_record_manager::MessageRecordManager;
use tower_http::cors::CorsLayer;

pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod http;
pub mod middlewares;
pub mod models;
pub mod repositories;
pub mod services;
pub mod utils;
pub mod ws;

#[derive(Clone)]
pub struct AppState {
    pub config: config::Config,
    pub pool: db::DbPool,
    pub ws_manager: ws::WsManager,
    pub message_record_manager: MessageRecordManager,
}

pub fn app_router(state: AppState) -> Router {
    let user_auth_layer =
        middleware::from_fn_with_state(state.clone(), middlewares::require_user_auth);
    let bot_auth_layer =
        middleware::from_fn_with_state(state.clone(), middlewares::require_bot_auth);

    let public_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/v1/auth/register", post(handlers::register))
        .route("/api/v1/auth/login", post(handlers::login))
        .route("/api/v1/bots/{bot_id}", get(handlers::get_bot))
        .route("/api/v1/groups/search", get(handlers::search_group_by_code))
        .route("/api/v1/groups/{group_id}", get(handlers::get_group))
        .route(
            "/api/v1/file/download/{file_id}/{filename}",
            get(handlers::download_file),
        );

    let user_auth_routes = Router::new()
        .route(
            "/api/v1/users/me",
            get(handlers::get_current_user).put(handlers::update_current_user),
        )
        .route("/api/v1/users/delete", delete(handlers::delete_user))
        .route(
            "/api/v1/bots",
            post(handlers::create_bot).get(handlers::list_bots),
        )
        .route("/api/v1/bots/search", get(handlers::search_bot_by_id))
        .route(
            "/api/v1/bots/{bot_id}",
            axum::routing::put(handlers::update_bot).delete(handlers::delete_bot),
        )
        .route(
            "/api/v1/groups",
            post(handlers::create_group).get(handlers::list_user_groups),
        )
        .route(
            "/api/v1/groups/{group_id}",
            put(handlers::update_group).delete(handlers::delete_group),
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
        .route("/api/v1/audit/logs", get(handlers::list_audit_logs))
        .route(
            "/api/v1/audit/logs/export",
            get(handlers::export_audit_logs_csv),
        )
        .route("/api/v1/notifications", get(handlers::list_notifications))
        .route(
            "/api/v1/notifications/{notification_id}/read",
            post(handlers::mark_notification_read),
        )
        .route(
            "/api/v1/notifications/{notification_id}/approve",
            post(handlers::approve_notification_join_request),
        )
        .route(
            "/api/v1/notifications/{notification_id}/reject",
            post(handlers::reject_notification_join_request),
        )
        .layer(user_auth_layer);

    let bot_auth_routes = Router::new()
        .route("/api/v1/bot/profile", get(handlers::get_current_bot))
        .route("/api/v1/bot/groups", get(handlers::list_bot_groups))
        .route(
            "/api/v1/groups/{group_id}/messages",
            get(handlers::get_group_messages),
        )
        .route("/api/v1/message/send", post(handlers::send_message))
        .route("/api/v1/file/upload", post(handlers::upload_file))
        .route("/api/v1/file/{file_id}", delete(handlers::delete_file))
        .layer(bot_auth_layer);

    public_routes
        .merge(user_auth_routes)
        .merge(bot_auth_routes)
        .route("/ws", get(handlers::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
