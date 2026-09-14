//! 跨 crate 共享的基础域类型（骨架占位）。

use serde::{Deserialize, Serialize};

use crate::id::AgentId;

/// Agent 注册/发现时的描述信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDescriptor {
    /// Agent 唯一标识。
    pub id: AgentId,
    /// 展示名称。
    pub name: String,
    /// Agent 服务地址（base URL）。
    pub endpoint: String,
    /// 支持的能力标签（如 chat、embedding）。
    pub capabilities: Vec<String>,
}

/// Agent 运行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// 在线。
    Online,
    /// 离线。
    Offline,
    /// 降级（部分能力不可用）。
    Degraded,
}

/// 网关 → Agent 的调用请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationRequest {
    /// 模型/能力标识。
    pub model: String,
    /// 请求负载（透传给 Agent）。
    pub payload: serde_json::Value,
    /// 是否流式。
    pub stream: bool,
}

/// Agent → 网关 的调用响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationResponse {
    /// 响应负载（由 Agent 返回）。
    pub payload: serde_json::Value,
}
