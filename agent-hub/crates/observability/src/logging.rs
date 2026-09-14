//! 日志初始化（骨架占位）：tracing-subscriber + EnvFilter。

/// 初始化全局日志订阅者（`RUST_LOG` env-filter，pretty/json 可选）。
pub fn init_logging() {
    tracing_declarative::init().expect("tracing init failed");
}

/// 构建滚动文件日志（可选，生产环境）。
pub fn file_logger(_dir: &str, _file_prefix: &str) {
    todo!("tracing_appender::rolling")
}
