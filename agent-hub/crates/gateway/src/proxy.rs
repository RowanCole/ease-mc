//! 转发服务（骨架占位）：按注册信息选择 Agent 并转发调用。

use common::{InvocationRequest, InvocationResponse, Result};

/// 转发服务：认证通过后将请求转发至目标 Agent。
#[derive(Debug, Default)]
pub struct ProxyService;

impl ProxyService {
    /// 创建转发服务。
    pub fn new() -> Self {
        Self
    }

    /// 同步转发：负载均衡选择 Agent → 调用 → 回传响应。
    pub async fn forward(&self, _request: InvocationRequest) -> Result<InvocationResponse> {
        todo!("选择 Agent 并经 AgentClient 转发")
    }
}
