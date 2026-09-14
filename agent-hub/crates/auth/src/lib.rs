//! AgentHub 身份验证：JWT 签发校验、API Key 管理与调用方身份模型。
//!
//! 骨架阶段：仅类型与签名，具体实现待补。

pub mod api_key;
pub mod jwt;
pub mod principal;

pub use jwt::{JwtClaims, JwtSigner, JwtVerifier};
pub use principal::{Principal, PrincipalKind};
