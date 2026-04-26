use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
        State,
    },
    http::HeaderMap,
    response::Response,
};
use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::services::authz::bot_has_global_group_access;
use crate::ws::{WsEvent, WsManager};
use crate::{
    error::AppResult,
    middlewares::{authenticate_bot_headers, authenticate_bot_token},
    repositories::GroupRepository,
    AppState,
};

#[derive(serde::Deserialize, Default)]
pub struct WsAuthQuery {
    pub token: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsAuthQuery>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let auth = if let Some(token) = query.token.as_deref() {
        authenticate_bot_token(&state, token).await?
    } else {
        authenticate_bot_headers(&state, &headers).await?
    };

    let group_ids: Vec<String> =
        if bot_has_global_group_access(&state.pool, &auth.bot_id).await? {
            GroupRepository::list_all_group_ids(&state.pool).await?
        } else {
            GroupRepository::list_group_ids_by_member(&state.pool, &auth.bot_id).await?
        };

    let ws_manager = state.ws_manager.clone();
    let bot_id = auth.bot_id;
    let bot_name = auth.name;

    Ok(ws.on_upgrade(move |socket| async move {
        handle_socket(socket, ws_manager, bot_id, bot_name, group_ids).await;
    }))
}

async fn handle_socket(
    mut socket: WebSocket,
    ws_manager: WsManager,
    bot_id: String,
    bot_name: String,
    group_ids: Vec<String>,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<WsEvent>();
    ws_manager.add_connection(bot_id.clone(), tx.clone()).await;

    let connection_event = WsEvent {
        event_type: "connection".to_string(),
        payload: serde_json::json!({
            "bot_id": bot_id,
            "bot_name": bot_name,
            "group_ids": group_ids,
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
                            if socket.send(Message::Text(text.into())).await.is_err() {
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

    ws_manager.remove_connection(&bot_id).await;
}
