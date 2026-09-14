//! 路由组装：对外暴露的 HTTP API 与中间件编排。

use std::sync::Arc;
use std::time::Duration;

use axum::http::{HeaderValue, StatusCode};
use axum::routing::get;
use axum::{middleware, Router};
use registry::AgentRegistry;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::middleware::request_id;

/// 网关运行期依赖（由 server 装配注入）。
pub struct GatewayDeps {
    /// 注册中心句柄。
    pub registry: Arc<dyn AgentRegistry>,
    /// 请求超时（覆盖整个请求-响应周期）。
    pub request_timeout: Duration,
    /// CORS 允许来源，逗号分隔；`*` 表示不限。
    pub cors_allow_origins: String,
}

impl std::fmt::Debug for GatewayDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayDeps").finish_non_exhaustive()
    }
}

pub fn build_router(deps: GatewayDeps) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .layer(build_cors(&deps.cors_allow_origins))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            deps.request_timeout,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(request_id))
}

/// 健康检查。
async fn healthz() -> &'static str {
    "ok"
}

/// 解析 CORS 允许来源：`*` 放开全部，否则按逗号分隔白名单。
fn build_cors(allow_origins: &str) -> CorsLayer {
    let origins: Vec<&str> = allow_origins
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .collect();

    if origins.iter().any(|origin| *origin == "*") {
        CorsLayer::permissive()
    } else {
        let list: Vec<HeaderValue> = origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        CorsLayer::new().allow_origin(AllowOrigin::list(list))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn healthz_returns_ok() {
        let deps = GatewayDeps {
            registry: Arc::new(registry::InMemoryRegistry::default()),
            request_timeout: Duration::from_secs(5),
            cors_allow_origins: "*".to_string(),
        };
        let app = build_router(deps);

        let response = app
            .oneshot(axum::http::Request::builder()
                .uri("/healthz")
                .body(axum::body::Body::empty())
                .unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn request_id_header_is_set() {
        let deps = GatewayDeps {
            registry: Arc::new(registry::InMemoryRegistry::default()),
            request_timeout: Duration::from_secs(5),
            cors_allow_origins: "*".to_string(),
        };
        let app = build_router(deps);

        let response = app
            .oneshot(axum::http::Request::builder()
                .uri("/healthz")
                .body(axum::body::Body::empty())
                .unwrap())
            .await
            .unwrap();
        assert!(response.headers().contains_key("x-request-id"));
    }
}
