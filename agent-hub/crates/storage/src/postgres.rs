//! PostgreSQL 实现（骨架占位）。

use common::Result;
    
/// 基于 sqlx 连接池的 PostgreSQL 存储（骨架占位）。
#[derive(Debug)]
pub struct PgStore;

impl PgStore {
    /// 建立连接池并执行数据库迁移。
    pub async fn connect(_database_url: &str, _max_connections: u32) -> Result<Self> {
        todo!("sqlx::PgPool::connect + sqlx::migrate!")
    }
}
