use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, watch};
use tokio::time::{interval, sleep};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::client::BochatClient;
use crate::error::{SdkError, SdkResult};
use crate::models::WsEvent;

pub struct WsSessionBuilder {
    client: BochatClient,
    bot_token: Option<String>,
    auto_reconnect: bool,
    reconnect_max_attempts: usize,
    reconnect_base_delay: Duration,
    heartbeat_interval: Duration,
    event_buffer: usize,
}

impl WsSessionBuilder {
    pub(crate) fn new(client: BochatClient) -> Self {
        Self {
            client,
            bot_token: None,
            auto_reconnect: true,
            reconnect_max_attempts: 10,
            reconnect_base_delay: Duration::from_secs(1),
            heartbeat_interval: Duration::from_secs(20),
            event_buffer: 128,
        }
    }

    pub fn bot_token(mut self, token: impl Into<String>) -> Self {
        self.bot_token = Some(token.into());
        self
    }

    pub fn auto_reconnect(mut self, auto_reconnect: bool) -> Self {
        self.auto_reconnect = auto_reconnect;
        self
    }

    pub fn reconnect_max_attempts(mut self, reconnect_max_attempts: usize) -> Self {
        self.reconnect_max_attempts = reconnect_max_attempts;
        self
    }

    pub fn heartbeat_interval(mut self, heartbeat_interval: Duration) -> Self {
        self.heartbeat_interval = heartbeat_interval;
        self
    }

    pub fn event_buffer(mut self, event_buffer: usize) -> Self {
        self.event_buffer = event_buffer.max(1);
        self
    }

    pub async fn build(self) -> SdkResult<WsSession> {
        let token = if let Some(token) = self.bot_token {
            token
        } else {
            self.client
                .bot_token()
                .await
                .ok_or(SdkError::MissingBotToken)?
        };

        Ok(WsSession {
            client: self.client,
            bot_token: token,
            auto_reconnect: self.auto_reconnect,
            reconnect_max_attempts: self.reconnect_max_attempts,
            reconnect_base_delay: self.reconnect_base_delay,
            heartbeat_interval: self.heartbeat_interval,
            event_buffer: self.event_buffer,
        })
    }
}

pub struct WsSession {
    client: BochatClient,
    bot_token: String,
    auto_reconnect: bool,
    reconnect_max_attempts: usize,
    reconnect_base_delay: Duration,
    heartbeat_interval: Duration,
    event_buffer: usize,
}

pub struct WsSessionHandle {
    pub events: mpsc::Receiver<WsEvent>,
    shutdown_tx: watch::Sender<bool>,
}

impl WsSessionHandle {
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

impl WsSession {
    pub fn websocket_url(&self) -> SdkResult<String> {
        let base = self.client.base_url();
        let ws_base = if let Some(rest) = base.strip_prefix("https://") {
            format!("wss://{}", rest)
        } else if let Some(rest) = base.strip_prefix("http://") {
            format!("ws://{}", rest)
        } else {
            return Err(SdkError::InvalidUrl(base.to_string()));
        };

        Ok(format!("{}/ws?token={}", ws_base, self.bot_token))
    }

    pub async fn spawn(self) -> SdkResult<WsSessionHandle> {
        let url = self.websocket_url()?;
        let (events_tx, events_rx) = mpsc::channel(self.event_buffer);
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        tokio::spawn(async move {
            let mut reconnect_attempt = 0usize;

            loop {
                if *shutdown_rx.borrow() {
                    break;
                }

                let connect_res = tokio_tungstenite::connect_async(&url).await;
                let (mut ws_stream, _) = match connect_res {
                    Ok(v) => {
                        reconnect_attempt = 0;
                        v
                    }
                    Err(err) => {
                        tracing::warn!("WS 连接失败: {}", err);
                        if !self.auto_reconnect || reconnect_attempt >= self.reconnect_max_attempts
                        {
                            break;
                        }
                        reconnect_attempt += 1;
                        sleep(self.reconnect_base_delay * reconnect_attempt as u32).await;
                        continue;
                    }
                };

                let mut heartbeat = interval(self.heartbeat_interval);

                loop {
                    tokio::select! {
                        _ = heartbeat.tick() => {
                            if ws_stream.send(Message::Ping(Vec::new().into())).await.is_err() {
                                break;
                            }
                        }
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() {
                                let _ = ws_stream.close(None).await;
                                break;
                            }
                        }
                        msg = ws_stream.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                                        if events_tx.send(event).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                                Some(Ok(Message::Close(_))) => {
                                    break;
                                }
                                Some(Err(err)) => {
                                    tracing::warn!("WS 读取异常: {}", err);
                                    break;
                                }
                                None => break,
                                _ => {}
                            }
                        }
                    }
                }

                if !self.auto_reconnect || reconnect_attempt >= self.reconnect_max_attempts {
                    break;
                }

                reconnect_attempt += 1;
                sleep(self.reconnect_base_delay * reconnect_attempt as u32).await;
            }
        });

        Ok(WsSessionHandle {
            events: events_rx,
            shutdown_tx,
        })
    }
}
