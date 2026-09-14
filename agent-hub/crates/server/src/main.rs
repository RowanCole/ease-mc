use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use config::load;
use gateway::{build_router, GatewayDeps};
use observability::init_logging;
use registry::{AgentRegistry, InMemoryRegistry};
use tokio::net::TcpListener;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().expect("failed to load .env");

    init_logging();

    info!("AgentHub is starting...");

    let settings = load()?;
    tracing::info!(env = %settings.env, "agenthub starting");

    // 3. 装配依赖（骨架：内存注册中心；后续按 storage.backend 替换实现）
    let registry: Arc<dyn AgentRegistry> = Arc::new(InMemoryRegistry::default());

    // 4. 组装网关路由并监听
    let deps = GatewayDeps {
        registry,
        request_timeout: Duration::from_secs(settings.gateway.request_timeout_secs),
        cors_allow_origins: settings.gateway.cors_allow_origins.clone(),
    };
    let app = build_router(deps);

    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = TcpListener::bind(&addr).await?;
    info!(%addr, "AgentHub listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("AgentHub shutdown complete");
    Ok(())
}

/// 优雅停机：等待 Ctrl-C。
async fn shutdown_signal() {
    if tokio::signal::ctrl_c().await.is_err() {
        error!("failed to install Ctrl-C handler");
    }
}
