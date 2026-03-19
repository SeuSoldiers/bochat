use actix_web::{web, HttpRequest, HttpResponse};

use crate::error::AppResult;

#[tracing::instrument(skip(_config, _http_req))]
pub async fn ws_handler(
    _config: web::Data<crate::config::Config>,
    _http_req: HttpRequest,
) -> AppResult<HttpResponse> {
    // TODO: Implement WebSocket handler with actix-web-actors
    // For now, return a placeholder response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "WebSocket handler not yet implemented"
    })))
}
