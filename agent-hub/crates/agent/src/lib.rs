//! AgentHub Agent 接入层：向已注册 Agent 转发调用的客户端封装。
//!
//! 骨架阶段：仅类型与签名。

pub mod client;

pub use client::AgentClient;
