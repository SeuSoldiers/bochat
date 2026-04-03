use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{Mutex, mpsc, watch};
use tokio::time::{interval, sleep};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::client::BochatClient;
use crate::error::{SdkError, SdkResult};
use crate::models::{WsConnectionPayload, WsEvent};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Builder for an advanced WebSocket session.
///
/// 高级 WebSocket 会话构建器。
///
/// It supports automatic reconnect, heartbeat pings, and buffered event
/// delivery.
///
/// 支持自动重连、心跳 ping，以及带缓冲的事件投递。
pub struct WsSessionBuilder {
    client: BochatClient,
    bot_token: Option<String>,
    auto_reconnect: bool,
    reconnect_max_attempts: usize,
    reconnect_base_delay: Duration,
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
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
            heartbeat_timeout: Duration::from_secs(60),
            event_buffer: 128,
        }
    }

    /// Override the bot token used for the WebSocket connection.
    ///
    /// 覆盖 WebSocket 连接使用的 Bot token。
    pub fn bot_token(mut self, token: impl Into<String>) -> Self {
        self.bot_token = Some(token.into());
        self
    }

    /// Enable or disable automatic reconnect.
    ///
    /// 启用或关闭自动重连。
    pub fn auto_reconnect(mut self, auto_reconnect: bool) -> Self {
        self.auto_reconnect = auto_reconnect;
        self
    }

    /// Set the maximum reconnect attempts.
    ///
    /// 设置最大重连次数。
    pub fn reconnect_max_attempts(mut self, reconnect_max_attempts: usize) -> Self {
        self.reconnect_max_attempts = reconnect_max_attempts;
        self
    }

    /// Set the ping heartbeat interval.
    ///
    /// 设置 ping 心跳发送间隔。
    pub fn heartbeat_interval(mut self, heartbeat_interval: Duration) -> Self {
        self.heartbeat_interval = heartbeat_interval;
        self
    }

    /// Set the timeout used to detect stale connections.
    ///
    /// 设置判定连接失活的超时时间。
    pub fn heartbeat_timeout(mut self, heartbeat_timeout: Duration) -> Self {
        self.heartbeat_timeout = heartbeat_timeout;
        self
    }

    /// Set the event channel buffer size.
    ///
    /// 设置事件通道缓冲区大小。
    pub fn event_buffer(mut self, event_buffer: usize) -> Self {
        self.event_buffer = event_buffer.max(1);
        self
    }

    /// Finalize the session configuration.
    ///
    /// 完成会话配置并构建会话对象。
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
            heartbeat_timeout: self.heartbeat_timeout,
            event_buffer: self.event_buffer,
        })
    }
}

/// Configured WebSocket session that can be spawned into a background task.
///
/// 已配置的 WebSocket 会话，可启动为后台任务。
pub struct WsSession {
    client: BochatClient,
    bot_token: String,
    auto_reconnect: bool,
    reconnect_max_attempts: usize,
    reconnect_base_delay: Duration,
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
    event_buffer: usize,
}

/// Handle returned after spawning a WebSocket session.
///
/// 启动 WebSocket 会话后返回的控制句柄。
pub struct WsSessionHandle {
    pub events: mpsc::Receiver<WsEvent>,
    shutdown_tx: watch::Sender<bool>,
    connection_rx: watch::Receiver<Option<WsConnectionPayload>>,
}

impl WsSessionHandle {
    /// Signal the background task to stop.
    ///
    /// 通知后台任务停止。
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Return the latest known group list from the connection payload.
    ///
    /// 返回最近一次连接事件中携带的群列表。
    pub fn available_groups(&self) -> Vec<String> {
        self.connection_rx
            .borrow()
            .as_ref()
            .map(|v| v.group_ids.clone())
            .unwrap_or_default()
    }

    /// Wait until the initial `connection` payload is received.
    ///
    /// 等待收到初始 `connection` 事件载荷。
    pub async fn wait_connection_payload(&mut self) -> SdkResult<WsConnectionPayload> {
        if let Some(v) = self.connection_rx.borrow().clone() {
            return Ok(v);
        }

        loop {
            self.connection_rx
                .changed()
                .await
                .map_err(|e| SdkError::WebSocket(e.to_string()))?;

            if let Some(v) = self.connection_rx.borrow().clone() {
                return Ok(v);
            }
        }
    }

    /// Receive the next message event for a specific group.
    ///
    /// 接收指定群的下一条消息事件。
    pub async fn recv_message_for_group(&mut self, group_id: &str) -> Option<WsEvent> {
        while let Some(event) = self.events.recv().await {
            if event.event_type == "message" && event.group_id() == Some(group_id) {
                return Some(event);
            }
        }
        None
    }

    /// Convert the raw handle into a dispatcher with subscription helpers.
    ///
    /// 将原始句柄转换为带订阅辅助能力的分发器。
    pub fn into_dispatcher(self) -> WsDispatcher {
        WsDispatcher::new(self)
    }
}

/// Message dispatcher built on top of [`WsSessionHandle`].
///
/// 构建在 [`WsSessionHandle`] 之上的消息分发器。
pub struct WsDispatcher {
    group_subscribers: Arc<Mutex<HashMap<String, Vec<mpsc::Sender<WsEvent>>>>>,
    fallback_subscribers: Arc<Mutex<Vec<mpsc::Sender<WsEvent>>>>,
    all_message_subscribers: Arc<Mutex<Vec<mpsc::Sender<WsEvent>>>>,
    group_handlers: Arc<Mutex<HashMap<String, Arc<dyn Fn(&WsEvent) + Send + Sync>>>>,
    default_handler: Arc<Mutex<Option<Arc<dyn Fn(&WsEvent) + Send + Sync>>>>,
    connection_rx: watch::Receiver<Option<WsConnectionPayload>>,
    shutdown_tx: watch::Sender<bool>,
}

impl WsDispatcher {
    fn new(mut handle: WsSessionHandle) -> Self {
        let group_subscribers = Arc::new(Mutex::new(
            HashMap::<String, Vec<mpsc::Sender<WsEvent>>>::new(),
        ));
        let fallback_subscribers = Arc::new(Mutex::new(Vec::<mpsc::Sender<WsEvent>>::new()));
        let all_message_subscribers = Arc::new(Mutex::new(Vec::<mpsc::Sender<WsEvent>>::new()));
        let group_handlers = Arc::new(Mutex::new(HashMap::<
            String,
            Arc<dyn Fn(&WsEvent) + Send + Sync>,
        >::new()));
        let default_handler = Arc::new(Mutex::new(None::<Arc<dyn Fn(&WsEvent) + Send + Sync>>));

        let groups_ref = Arc::clone(&group_subscribers);
        let fallback_ref = Arc::clone(&fallback_subscribers);
        let all_ref = Arc::clone(&all_message_subscribers);
        let group_handlers_ref = Arc::clone(&group_handlers);
        let default_handler_ref = Arc::clone(&default_handler);

        tokio::spawn(async move {
            while let Some(event) = handle.events.recv().await {
                if event.event_type != "message" {
                    continue;
                }

                {
                    let mut all_subs = all_ref.lock().await;
                    all_subs.retain(|tx| tx.try_send(event.clone()).is_ok());
                }

                let mut handled_by_group_handler = false;
                if let Some(group_id) = event.group_id().map(|s| s.to_string()) {
                    let delivered_to_channel = {
                        let mut grouped = groups_ref.lock().await;
                        if let Some(list) = grouped.get_mut(&group_id) {
                            list.retain(|tx| tx.try_send(event.clone()).is_ok());
                            !list.is_empty()
                        } else {
                            false
                        }
                    };

                    let handlers = group_handlers_ref.lock().await;
                    if let Some(handler) = handlers.get(&group_id) {
                        handler(&event);
                        handled_by_group_handler = true;
                    }

                    if delivered_to_channel || handled_by_group_handler {
                        continue;
                    }
                }

                let mut fallback = fallback_ref.lock().await;
                fallback.retain(|tx| tx.try_send(event.clone()).is_ok());

                let handler_opt = default_handler_ref.lock().await;
                if let Some(handler) = handler_opt.as_ref() {
                    handler(&event);
                }
            }
        });

        Self {
            group_subscribers,
            fallback_subscribers,
            all_message_subscribers,
            group_handlers,
            default_handler,
            connection_rx: handle.connection_rx,
            shutdown_tx: handle.shutdown_tx,
        }
    }

    /// Register a default callback for messages that are not consumed by a
    /// group-specific handler or subscriber.
    ///
    /// 注册默认回调，用于处理未被群级处理器或订阅器消费的消息。
    pub async fn default_handler<F>(&self, handler: F) -> &Self
    where
        F: Fn(&WsEvent) + Send + Sync + 'static,
    {
        let mut h = self.default_handler.lock().await;
        *h = Some(Arc::new(handler));
        self
    }

    /// Register a callback for a specific group.
    ///
    /// 为指定群注册回调函数。
    pub async fn group_handler<F>(&self, group_id: impl Into<String>, handler: F) -> &Self
    where
        F: Fn(&WsEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.group_handlers.lock().await;
        handlers.insert(group_id.into(), Arc::new(handler));
        self
    }

    /// Signal the dispatcher and its underlying session to stop.
    ///
    /// 通知分发器及其底层会话停止。
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Return the latest known group list.
    ///
    /// 返回最近一次连接事件中的群列表。
    pub fn available_groups(&self) -> Vec<String> {
        self.connection_rx
            .borrow()
            .as_ref()
            .map(|v| v.group_ids.clone())
            .unwrap_or_default()
    }

    /// Wait for the initial connection payload.
    ///
    /// 等待初始连接事件载荷。
    pub async fn wait_connection_payload(&mut self) -> SdkResult<WsConnectionPayload> {
        if let Some(v) = self.connection_rx.borrow().clone() {
            return Ok(v);
        }

        loop {
            self.connection_rx
                .changed()
                .await
                .map_err(|e| SdkError::WebSocket(e.to_string()))?;

            if let Some(v) = self.connection_rx.borrow().clone() {
                return Ok(v);
            }
        }
    }

    /// Subscribe to messages for one specific group.
    ///
    /// 订阅某个指定群的消息。
    pub async fn subscribe_group(
        &self,
        group_id: impl Into<String>,
        buffer: usize,
    ) -> mpsc::Receiver<WsEvent> {
        let (tx, rx) = mpsc::channel(buffer.max(1));
        let mut grouped = self.group_subscribers.lock().await;
        grouped.entry(group_id.into()).or_default().push(tx);
        rx
    }

    /// Subscribe to unmatched message events.
    ///
    /// 订阅未命中任何群专属通道的消息事件。
    pub async fn subscribe_fallback(&self, buffer: usize) -> mpsc::Receiver<WsEvent> {
        let (tx, rx) = mpsc::channel(buffer.max(1));
        let mut fallback = self.fallback_subscribers.lock().await;
        fallback.push(tx);
        rx
    }

    /// Subscribe to every message event before group routing.
    ///
    /// 在群路由前订阅全部消息事件。
    pub async fn subscribe_all_messages(&self, buffer: usize) -> mpsc::Receiver<WsEvent> {
        let (tx, rx) = mpsc::channel(buffer.max(1));
        let mut list = self.all_message_subscribers.lock().await;
        list.push(tx);
        rx
    }
}

impl WsSession {
    /// Build the concrete WebSocket URL from the configured base URL and bot
    /// token.
    ///
    /// 基于当前基础 URL 和 Bot token 构造具体的 WebSocket 连接地址。
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

    /// Spawn the WebSocket session into a background task and return a handle.
    ///
    /// 将 WebSocket 会话启动为后台任务并返回控制句柄。
    pub async fn spawn(self) -> SdkResult<WsSessionHandle> {
        let url = self.websocket_url()?;
        let (events_tx, events_rx) = mpsc::channel(self.event_buffer);
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let (connection_tx, connection_rx) = watch::channel::<Option<WsConnectionPayload>>(None);

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
                let mut last_seen = Instant::now();

                loop {
                    tokio::select! {
                        _ = heartbeat.tick() => {
                            if last_seen.elapsed() > self.heartbeat_timeout {
                                tracing::warn!("WS 心跳超时，触发重连");
                                break;
                            }
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
                                    last_seen = Instant::now();
                                    if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                                        if let Some(payload) = event.as_connection_payload() {
                                            let _ = connection_tx.send(Some(payload));
                                        }
                                        if events_tx.send(event).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                                Some(Ok(Message::Ping(payload))) => {
                                    last_seen = Instant::now();
                                    if ws_stream.send(Message::Pong(payload)).await.is_err() {
                                        break;
                                    }
                                }
                                Some(Ok(Message::Pong(_))) => {
                                    last_seen = Instant::now();
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
            connection_rx,
        })
    }
}
