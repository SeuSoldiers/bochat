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

pub mod prelude {
    pub use crate::client::{BochatClient, BochatClientBuilder};
    pub use crate::error::{SdkError, SdkResult};
    pub use crate::groups::GroupsApi;
    pub use crate::models::*;
    pub use crate::retry::RetryPolicy;
    #[cfg(feature = "ws")]
    pub use crate::ws::{WsSession, WsSessionBuilder};
}
