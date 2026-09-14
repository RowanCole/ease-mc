//! AgentHub 注册中心：Agent 动态注册、心跳续约与租约管理。
//!
//! 骨架阶段：仅 trait 与占位实现。

pub mod model;
pub mod registry;

pub use model::{AgentRegistration, RegisteredAgent};
pub use registry::{AgentRegistry, InMemoryRegistry};
