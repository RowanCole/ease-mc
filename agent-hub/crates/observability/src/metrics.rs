//! 指标初始化（骨架占位）：metrics + Prometheus 导出。

use thiserror::Error;

/// 可观测性初始化错误。
#[derive(Debug, Error)]
pub enum ObservabilityError {
    /// Prometheus 导出器初始化失败。
    #[error("metrics exporter error: {0}")]
    Exporter(String),
}

/// 初始化 Prometheus 导出器并绑定监听地址。
pub fn init_metrics(_listen_addr: &str) -> Result<(), ObservabilityError> {
    todo!("metrics_exporter_prometheus::PrometheusBuilder")
}

/// 记录一次请求耗时（占位）。
pub fn record_request(_route: &str, _status: u16, _duration_secs: f64) {
    todo!("metrics::histogram! 记录")
}
