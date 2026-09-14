//! 仓储抽象（骨架占位）：存储层对外的接口定义。
//!
//! 注意：存储层只依赖 core，避免与上层 crate 形成循环依赖。

use async_trait::async_trait;

use common::{AgentDescriptor, AgentId, Result};

/// Agent 仓储：注册信息的持久化接口。
#[async_trait]
pub trait AgentRepository: Send + Sync {
    /// 写入/更新 Agent 描述信息。
    async fn put(&self, _agent: &AgentDescriptor) -> Result<()> {
        todo!("upsert")
    }

    /// 读取指定 Agent；不存在时返回 `None`。
    async fn get(&self, _agent_id: &AgentId) -> Result<Option<AgentDescriptor>> {
        todo!("查询")
    }

    /// 删除指定 Agent。
    async fn delete(&self, _agent_id: &AgentId) -> Result<()> {
        todo!("删除")
    }

    /// 列出全部 Agent。
    async fn list(&self) -> Result<Vec<AgentDescriptor>> {
        todo!("列表")
    }
}
