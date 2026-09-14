//! AgentHub 网关客户端（骨架占位）。

use crate::error::Result;

/// AgentHub 网关客户端：外部调用方接入入口。
#[derive(Debug)]
pub struct AgentHubClient;

impl AgentHubClient {
    /// 创建客户端（base_url + API Key）。
    pub fn new(_base_url: impl Into<String>, _api_key: impl Into<String>) -> Self {
        todo!("构建 reqwest::Client 并附加认证头")
    }

    /// 调用 Agent（同步请求/响应）。
    pub async fn invoke(
        &self,
        _model: &str,
        _payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        todo!("POST /v1/invoke")
    }

    /// 列出当前可用 Agent。
    pub async fn list_agents(&self) -> Result<Vec<String>> {
        todo!("GET /v1/agents")
    }
}
