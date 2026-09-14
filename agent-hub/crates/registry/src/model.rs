//! 注册中心数据模型（骨架占位）。

use serde::{Deserialize, Serialize};

use common::AgentDescriptor;

/// Agent 注册请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    /// 展示名称。
    pub name: String,
    /// Agent 服务地址（base URL）。
    pub endpoint: String,
    /// 支持的能力标签。
    pub capabilities: Vec<String>,
}

/// 已注册 Agent（含租约信息）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredAgent {
    /// Agent 描述信息。
    pub descriptor: AgentDescriptor,
    /// 租约到期时间（Unix 秒）；到期未续约视为下线。
    pub lease_expires_at: u64,
}

impl RegisteredAgent {
    /// 判断租约是否已过期。
    pub fn is_lease_expired(&self, _now_unix_secs: u64) -> bool {
        todo!("比较 now 与 lease_expires_at")
    }
}
