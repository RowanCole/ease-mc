//! AgentHub 存储层：仓储 trait、内存实现与 PostgreSQL 适配。
//!
//! 骨架阶段：仅 trait 与占位实现。

pub mod memory;
pub mod postgres;
pub mod repository;

pub use memory::InMemoryStore;
pub use postgres::PgStore;
pub use repository::AgentRepository;
