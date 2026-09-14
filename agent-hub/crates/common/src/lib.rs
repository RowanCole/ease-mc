//! AgentHub 核心域模型：统一错误、强类型 ID 与共享基础类型。
//!
//! 骨架阶段仅包含占位类型与签名；具体实现随各里程碑补齐。

pub mod error;
pub mod id;
pub mod types;

pub use error::{AgentHubError, Result};
pub use id::{AgentId, RequestId, TenantId};
pub use types::{AgentDescriptor, AgentStatus, InvocationRequest, InvocationResponse};
