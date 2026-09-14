//! AgentHub 可观测性：日志初始化、指标采集与 Prometheus 导出。
//!
//! 骨架阶段：仅类型与签名。

pub mod logging;
pub mod metrics;

pub use logging::init_logging;
pub use metrics::{init_metrics, ObservabilityError};
