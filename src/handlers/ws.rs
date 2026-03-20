use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::utils::verify_token;
use crate::ws::{WsEvent, WsManager};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub struct WsSession {
    bot_ids: Vec<String>,
    rx: mpsc::UnboundedReceiver<WsEvent>,
    ws_manager: web::Data<WsManager>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(std::time::Duration::from_millis(200), |act, ctx| {
            while let Ok(event) = act.rx.try_recv() {
                if let Ok(text) = serde_json::to_string(&event) {
                    ctx.text(text);
                }
            }
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let bot_ids = self.bot_ids.clone();
        let ws_manager = self.ws_manager.clone();
        actix_rt::spawn(async move {
            for bot_id in bot_ids {
                ws_manager.remove_connection(&bot_id).await;
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(_)) | Ok(ws::Message::Binary(_)) | Ok(ws::Message::Pong(_)) => {}
            Err(_) => ctx.stop(),
            _ => {}
        }
    }
}

#[tracing::instrument(skip(config, http_req, pool, ws_manager, stream))]
pub async fn ws_handler(
    config: web::Data<crate::config::Config>,
    pool: web::Data<DbPool>,
    ws_manager: web::Data<WsManager>,
    http_req: HttpRequest,
    query: web::Query<WsQuery>,
    stream: web::Payload,
) -> AppResult<HttpResponse> {
    let token = &query.token;
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 3 {
        return Err(AppError::InvalidToken);
    }
    let requester_bot_id = parts[0];

    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    let _token_payload = verify_token(
        token,
        &requester_bot.secret,
        config.security.token_expiry_secs,
    )?;

    let owned_bot_ids: Vec<String> =
        sqlx::query_scalar("SELECT bot_id FROM bots WHERE owner_id = ? ORDER BY created_at ASC")
            .bind(&requester_bot.owner_id)
            .fetch_all(pool.get_ref())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let (tx, rx) = mpsc::unbounded_channel();
    for bot_id in &owned_bot_ids {
        ws_manager.add_connection(bot_id.clone(), tx.clone()).await;
    }

    let connection_event = WsEvent {
        event_type: "connection".to_string(),
        payload: serde_json::json!({
            "owner_id": requester_bot.owner_id,
            "bot_ids": owned_bot_ids,
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let _ = tx.send(connection_event);

    let session = WsSession {
        bot_ids: owned_bot_ids,
        rx,
        ws_manager,
    };

    ws::start(session, &http_req, stream).map_err(|e| AppError::InternalError(e.to_string()))
}
