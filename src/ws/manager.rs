use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsMessage {
    pub msg_type: String,
    pub msg_id: Option<i64>,
    pub sender_id: Option<String>,
    pub to_id: Option<String>,
    pub content: Option<serde_json::Value>,
    pub created_at: Option<String>,
}

pub struct WsManager {
    // Map of bot_id to list of WebSocket connections
    connections: Arc<RwLock<HashMap<String, Vec<tokio::sync::mpsc::UnboundedSender<WsMessage>>>>>,
}

impl WsManager {
    pub fn new() -> Self {
        WsManager {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(
        &self,
        bot_id: String,
        tx: tokio::sync::mpsc::UnboundedSender<WsMessage>,
    ) {
        let mut connections = self.connections.write().await;
        connections.entry(bot_id).or_insert_with(Vec::new).push(tx);
    }

    pub async fn remove_connection(&self, bot_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conns) = connections.get_mut(bot_id) {
            conns.retain(|tx| !tx.is_closed());
            if conns.is_empty() {
                connections.remove(bot_id);
            }
        }
    }

    pub async fn broadcast_message(&self, bot_id: &str, message: WsMessage) {
        let connections = self.connections.read().await;
        if let Some(conns) = connections.get(bot_id) {
            for tx in conns {
                // Ignore send errors (closed connections will be cleaned up on next message)
                let _ = tx.send(message.clone());
            }
        }
    }

    pub async fn get_online_count(&self, bot_id: &str) -> usize {
        let connections = self.connections.read().await;
        connections.get(bot_id).map(|c| c.len()).unwrap_or(0)
    }
}

impl Clone for WsManager {
    fn clone(&self) -> Self {
        WsManager {
            connections: Arc::clone(&self.connections),
        }
    }
}

impl Default for WsManager {
    fn default() -> Self {
        Self::new()
    }
}
