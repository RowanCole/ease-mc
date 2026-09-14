//! AgentHub 客户端 SDK：供外部调用方接入 Agent 网关。
//!
//! 骨架阶段：仅类型与签名。

pub mod client;
pub mod error;

pub use client::AgentHubClient;
pub use error::{Result, SdkError};
