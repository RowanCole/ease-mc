//! 配置加载：默认值 → 配置文件（`AGENTHUB_CONFIG_FILE`）→ 环境变量（`AGENTHUB_*`）。
//!
//! 环境变量的嵌套键使用双下划线分隔：`AGENTHUB_SERVER__PORT` 对应 `server.port`。

use common::{AgentHubError, Result};
use config_rs::{Config, Environment, File, Value};

use crate::settings::AgentHubSettings;

/// 按优先级加载配置：默认值 < 配置文件 < 环境变量。
///
/// 配置文件路径取环境变量 `AGENTHUB_CONFIG_FILE`（可选，支持 toml/yaml/json）。
pub fn load() -> Result<AgentHubSettings> {
    let path = std::env::var("AGENTHUB_CONFIG_FILE").ok();
    load_from(path.as_deref())
}

/// 从指定路径加载配置文件（测试/调试辅助；`None` 时仅环境变量 + 默认值）。
///
/// 文件缺失时降级为「默认值 + 环境变量」，不视为错误。
pub fn load_from(config_file: Option<&str>) -> Result<AgentHubSettings> {
    let mut builder = Config::builder();
    for (key, value) in defaults() {
        builder = builder
            .set_default(key, value)
            .map_err(|e| AgentHubError::Config(e.to_string()))?;
    }

    // 可选配置文件（路径不存在时跳过）
    if let Some(path) = config_file {
        builder = builder.add_source(File::with_name(path).required(false));
    }

    // 环境变量：AGENTHUB_<SECTION>__<FIELD>
    builder = builder.add_source(
        Environment::with_prefix("AGENTHUB")
            .separator("__")
            .try_parsing(true),
    );

    let settings: AgentHubSettings = builder
        .build()
        .and_then(|config| config.try_deserialize())
        .map_err(|e| AgentHubError::Config(e.to_string()))?;
    Ok(settings)
}

/// 内置默认值（与 .env.example 保持一致）。
fn defaults() -> Vec<(&'static str, Value)> {
    vec![
        ("env", "development".into()),
        ("server.host", "0.0.0.0".into()),
        ("server.port", 8080i64.into()),
        ("auth.jwt_issuer", "agenthub".into()),
        ("auth.jwt_audience", "agenthub-clients".into()),
        ("auth.access_token_ttl_secs", 3600i64.into()),
        ("auth.api_key_prefix", "ah_".into()),
        ("storage.backend", "memory".into()),
        ("storage.database_max_connections", 10i64.into()),
        ("registry.lease_ttl_secs", 60i64.into()),
        ("registry.cleanup_interval_secs", 15i64.into()),
        ("gateway.request_timeout_secs", 30i64.into()),
        ("gateway.connect_timeout_secs", 5i64.into()),
        ("gateway.max_body_bytes", 4_194_304i64.into()),
        ("gateway.cors_allow_origins", "*".into()),
        ("observability.log_format", "pretty".into()),
        ("observability.metrics_enabled", true.into()),
        ("observability.metrics_path", "/metrics".into()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_applied_without_file_and_env() {
        let settings = load_from(None).unwrap();
        assert_eq!(settings.env, "development");
        assert_eq!(settings.server.host, "0.0.0.0");
        assert_eq!(settings.server.port, 8080);
        assert_eq!(settings.gateway.request_timeout_secs, 30);
        assert_eq!(settings.registry.lease_ttl_secs, 60);
    }
}
