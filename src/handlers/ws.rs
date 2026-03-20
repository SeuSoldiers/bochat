use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::{
    error::{AppError, AppResult},
    AppState,
};
use crate::utils::verify_token;
use crate::ws::{WsEvent, WsManager};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: String,
}

#[tracing::instrument(skip(state, ws))]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> AppResult<Response> {
    let token = &query.token;
    let requester_bot_id = crate::http::token_bot_id(token)?;

    let requester_bot: crate::models::Bot = sqlx::query_as(
        "SELECT bot_id, owner_id, name, description, avatar_url, status, token, secret, created_at, updated_at FROM bots WHERE bot_id = ?"
    )
    .bind(requester_bot_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or(AppError::BotNotFound)?;

    let _token_payload = verify_token(
        token,
        &requester_bot.secret,
        state.config.security.token_expiry_secs,
    )?;

    let owned_bot_ids: Vec<String> =
        sqlx::query_scalar("SELECT bot_id FROM bots WHERE owner_id = ? ORDER BY created_at ASC")
            .bind(&requester_bot.owner_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let ws_manager = state.ws_manager.clone();
    let owner_id = requester_bot.owner_id.clone();

    Ok(ws.on_upgrade(move |socket| async move {
        handle_socket(socket, ws_manager, owned_bot_ids, owner_id).await;
    }))
}

async fn handle_socket(
    mut socket: WebSocket,
    ws_manager: WsManager,
    bot_ids: Vec<String>,
    owner_id: String,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<WsEvent>();
    for bot_id in &bot_ids {
        ws_manager.add_connection(bot_id.clone(), tx.clone()).await;
    }

    let connection_event = WsEvent {
        event_type: "connection".to_string(),
        payload: serde_json::json!({
            "owner_id": owner_id,
            "bot_ids": bot_ids,
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let _ = tx.send(connection_event);

    loop {
        tokio::select! {
            maybe_event = rx.recv() => {
                match maybe_event {
                    Some(event) => {
                        if let Ok(text) = serde_json::to_string(&event) {
                            if socket.send(Message::Text(text)).await.is_err() {
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }
            maybe_message = socket.next() => {
                match maybe_message {
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Text(_))) | Some(Ok(Message::Binary(_))) | Some(Ok(Message::Pong(_))) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }

    for bot_id in &bot_ids {
        ws_manager.remove_connection(bot_id).await;
    }
}
