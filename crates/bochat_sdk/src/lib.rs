//! BoChat Rust SDK.
//!
//! BoChat 平台 Rust SDK。
//!
//! This crate wraps the current BoChat HTTP and optional WebSocket APIs into an
//! async client suitable for bots, integration services, and CLI tools.
//!
//! 本 crate 将当前 BoChat 的 HTTP 接口和可选 WebSocket 能力封装为异步客户端，
//! 适合 Bot 服务、集成服务和命令行工具使用。
//!
//! Typical flow / 典型调用流程:
//!
//! 1. Build a [`client::BochatClient`].
//! 2. Register or login with [`auth::AuthApi`].
//! 3. Query bots with [`bots::BotsApi`] and set an active bot token.
//! 4. Create groups, send messages, upload files, or open a WebSocket session.
//!
//! 1. 构建 [`client::BochatClient`]。
//! 2. 通过 [`auth::AuthApi`] 注册或登录。
//! 3. 通过 [`bots::BotsApi`] 查询 Bot，并设置当前使用的 Bot token。
//! 4. 创建群、发送消息、上传文件，或建立 WebSocket 会话。

pub mod auth;
pub mod bots;
pub mod client;
pub mod error;
pub mod files;
pub mod groups;
pub mod messages;
pub mod models;
pub mod retry;
#[cfg(feature = "ws")]
pub mod ws;

/// Commonly used re-exports for quick integration.
///
/// 常用公开类型的便捷导出，适合 `use bochat_sdk::prelude::*;`。
pub mod prelude {
    pub use crate::client::{BochatClient, BochatClientBuilder};
    pub use crate::error::{SdkError, SdkResult};
    pub use crate::groups::GroupsApi;
    pub use crate::models::*;
    pub use crate::retry::RetryPolicy;
    #[cfg(feature = "ws")]
    pub use crate::ws::{WsDispatcher, WsSession, WsSessionBuilder};
}
