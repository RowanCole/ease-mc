//! 认证后的调用方身份（骨架占位）。

use common::TenantId;

/// 调用方类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalKind {
    /// 外部调用方。
    Client,
    /// Agent 自身。
    Agent,
    /// 管理端。
    Admin,
}

/// 通过认证的调用方身份。
#[derive(Debug, Clone)]
pub struct Principal {
    /// 调用方标识。
    pub tenant_id: TenantId,
    /// 身份类型。
    pub kind: PrincipalKind,
    /// 授权能力/作用域（骨架占位）。
    pub scopes: Vec<String>,
}
