//! AgentHub 网关：路由、认证中间件与 Agent 转发。
//!
//! P0 已就绪：基础路由（/healthz）、trace/timeout/cors、请求 ID；认证与转发待补。

pub mod middleware;
pub mod proxy;
pub mod router;

pub use router::{build_router, GatewayDeps};
