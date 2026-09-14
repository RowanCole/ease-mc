//! Agent 注册中心抽象与默认内存实现（骨架占位）。

use async_trait::async_trait;

use common::{AgentId, Result};

use crate::model::{AgentRegistration, RegisteredAgent};

/// Agent 动态注册中心：注册、心跳续约、注销与发现。
#[async_trait]
pub trait AgentRegistry: Send + Sync {
    /// 动态注册：生成 AgentId 并建立租约。
    async fn register(&self, _registration: AgentRegistration) -> Result<RegisteredAgent> {
        todo!("生成 AgentId，写入存储并返回租约")
    }

    /// 心跳续约：延长指定 Agent 的租约。
    async fn renew(&self, _agent_id: &AgentId) -> Result<()> {
        todo!("续约失败视为已下线")
    }

    /// 注销指定 Agent。
    async fn deregister(&self, _agent_id: &AgentId) -> Result<()> {
        todo!("移除注册记录")
    }

    /// 列出当前在线（租约未过期）的 Agent。
    async fn list_online(&self) -> Result<Vec<RegisteredAgent>> {
        todo!("过滤过期租约")
    }

    /// 查询指定 Agent；不存在时返回 Registry 错误。
    async fn get(&self, _agent_id: &AgentId) -> Result<RegisteredAgent> {
        todo!("查询")
    }
}

/// 基于内存的注册中心实现（骨架占位，方法走 trait 默认 `todo!()`）。
#[derive(Debug, Default)]
pub struct InMemoryRegistry;

#[async_trait]
impl AgentRegistry for InMemoryRegistry {}
