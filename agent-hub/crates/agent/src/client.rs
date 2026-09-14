//! Agent 调用客户端（骨架占位）：网关 → Agent 的转发封装。

use common::{AgentId, AgentStatus, InvocationRequest, InvocationResponse, Result};

/// Agent 调用客户端：按注册信息向目标 Agent 发起请求。
#[derive(Debug)]
pub struct AgentClient;

impl AgentClient {
    /// 创建客户端。
    pub fn new() -> Self {
        Self
    }

    /// 调用指定 Agent（同步请求/响应）。
    pub async fn invoke(
        &self,
        _agent_id: &AgentId,
        _request: InvocationRequest,
    ) -> Result<InvocationResponse> {
        todo!("查注册中心取 endpoint，reqwest 转发")
    }

    /// 建立流式调用（SSE/chunked 透传）。
    pub async fn invoke_stream(&self, _agent_id: &AgentId, _request: InvocationRequest) -> Result<()> {
        todo!("流式响应透传给网关调用方")
    }

    /// Agent 健康检查。
    pub async fn health(&self, _agent_id: &AgentId) -> Result<AgentStatus> {
        todo!("探测 Agent 健康端点")
    }
}

impl Default for AgentClient {
    fn default() -> Self {
        Self::new()
    }
}
