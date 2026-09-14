//! 配置模型：与 `.env.example` 中的 `AGENTHUB_*` 项一一对应。

use serde::Deserialize;

/// AgentHub 顶层配置。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AgentHubSettings {
    /// 运行环境：development | staging | production。
    pub env: String,
    /// HTTP 服务。
    pub server: ServerSettings,
    /// 认证。
    pub auth: AuthSettings,
    /// 存储。
    pub storage: StorageSettings,
    /// 注册中心。
    pub registry: RegistrySettings,
    /// 网关。
    pub gateway: GatewaySettings,
    /// 可观测性。
    pub observability: ObservabilitySettings,
}

/// 服务监听配置。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    /// 监听地址。
    pub host: String,
    /// 监听端口。
    pub port: u16,
}

/// 认证配置。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AuthSettings {
    /// JWT 签名密钥（敏感，生产环境勿入库/入日志）。
    pub jwt_secret: String,
    /// JWT 签发方。
    pub jwt_issuer: String,
    /// JWT 受众。
    pub jwt_audience: String,
    /// 访问令牌有效期（秒）。
    pub access_token_ttl_secs: u64,
    /// API Key 前缀。
    pub api_key_prefix: String,
}

/// 存储配置。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StorageSettings {
    /// 存储后端：memory | postgres。
    pub backend: String,
    /// PostgreSQL 连接串。
    pub database_url: String,
    /// 连接池上限。
    pub database_max_connections: u32,
    /// Redis 连接串。
    pub redis_url: String,
}

/// 注册中心配置。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RegistrySettings {
    /// 注册租约 TTL（秒）。
    pub lease_ttl_secs: u64,
    /// 过期清理扫描间隔（秒）。
    pub cleanup_interval_secs: u64,
}

/// 网关配置。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GatewaySettings {
    /// 请求超时（秒）。
    pub request_timeout_secs: u64,
    /// 连接超时（秒）。
    pub connect_timeout_secs: u64,
    /// 请求体大小上限（字节）。
    pub max_body_bytes: usize,
    /// CORS 允许来源，`*` 表示不限。
    pub cors_allow_origins: String,
}

/// 可观测性配置。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ObservabilitySettings {
    /// 日志格式：pretty | json。
    pub log_format: String,
    /// 是否启用指标。
    pub metrics_enabled: bool,
    /// 指标暴露路径。
    pub metrics_path: String,
    /// OTLP 导出端点（留空禁用）。
    pub otlp_endpoint: String,
}
