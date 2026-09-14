//! 内存实现（骨架占位）：开发与测试用。

use async_trait::async_trait;

use crate::repository::AgentRepository;

/// 进程内存储（骨架占位，方法走 trait 默认 `todo!()`）。
#[derive(Debug, Default)]
pub struct InMemoryStore;

#[async_trait]
impl AgentRepository for InMemoryStore {}
