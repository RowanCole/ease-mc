//! 统一错误类型：所有 crate 复用 `AgentHubError` 与 `Result` 别名。

use thiserror::Error;

/// AgentHub 统一错误。
#[derive(Debug, Error)]
pub enum AgentHubError {
    /// 配置加载或校验失败。
    #[error("config error: {0}")]
    Config(String),

    /// 认证或授权失败。
    #[error("auth error: {0}")]
    Auth(String),

    /// 注册中心操作失败。
    #[error("registry error: {0}")]
    Registry(String),

    /// 存储层错误。
    #[error("storage error: {0}")]
    Storage(String),

    /// 网关路由/转发失败。
    #[error("gateway error: {0}")]
    Gateway(String),

    /// Agent 调用失败。
    #[error("agent error: {0}")]
    Agent(String),

    /// 未分类的内部错误。
    #[error("internal error: {0}")]
    Internal(String),
}

/// 统一 Result 别名。
pub type Result<T, E = AgentHubError> = std::result::Result<T, E>;
