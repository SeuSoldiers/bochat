use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: String,
}

pub struct WsManager {
    // Map of bot_id to list of WebSocket connections
    connections: Arc<RwLock<HashMap<String, Vec<tokio::sync::mpsc::UnboundedSender<WsEvent>>>>>,
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
        tx: tokio::sync::mpsc::UnboundedSender<WsEvent>,
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

    pub async fn broadcast_message(&self, bot_id: &str, message: WsEvent) {
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

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tokio::sync::mpsc;

    use super::{WsEvent, WsManager};

    #[tokio::test]
    async fn add_broadcast_remove_connection_work() {
        let manager = WsManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel::<WsEvent>();
        manager.add_connection("b1".to_string(), tx).await;
        assert_eq!(manager.get_online_count("b1").await, 1);

        manager
            .broadcast_message(
                "b1",
                WsEvent {
                    event_type: "message".to_string(),
                    payload: json!({"k":"v"}),
                    timestamp: "now".to_string(),
                },
            )
            .await;
        let got = rx.recv().await.expect("should receive event");
        assert_eq!(got.event_type, "message");
        assert_eq!(got.payload["k"], "v");

        drop(rx);
        manager.remove_connection("b1").await;
        assert_eq!(manager.get_online_count("b1").await, 0);
    }
}
