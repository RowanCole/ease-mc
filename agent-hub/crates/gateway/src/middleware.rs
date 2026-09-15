//! 网关中间件：请求 ID、认证（P2 实现）。

use axum::extract::Request;
use axum::response::Response;
use common::RequestId;

/// 认证中间件：校验 JWT / API Key，校验通过后注入 `Principal`。
pub async fn auth(_req: Request, _next: axum::middleware::Next) -> Response {
    todo!("提取 Authorization / X-Api-Key，失败返回 401")
}

pub async fn request_id(mut req: Request, next: axum::middleware::Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(RequestId::from_raw)
        .unwrap_or_else(RequestId::generate);

    req.extensions_mut().insert(request_id.clone());

    let mut response = next.run(req).await;
    if let Ok(value) = request_id.to_string().parse() {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}
