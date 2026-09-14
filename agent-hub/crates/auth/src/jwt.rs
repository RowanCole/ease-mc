//! JWT 签发与校验（骨架占位）。

use common::Result;
use serde::{Deserialize, Serialize};

/// JWT Claims（骨架占位）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// 主题（调用方/Agent ID）。
    pub sub: String,
    /// 签发方。
    pub iss: String,
    /// 受众。
    pub aud: String,
    /// 过期时间（Unix 秒）。
    pub exp: u64,
    /// 签发时间（Unix 秒）。
    pub iat: u64,
}

/// JWT 签发器（骨架占位）。
#[derive(Debug)]
pub struct JwtSigner;

impl JwtSigner {
    /// 使用配置密钥创建签发器。
    pub fn new(_secret: impl Into<String>, _issuer: impl Into<String>) -> Self {
        todo!("保存 HS256 密钥与 issuer")
    }

    /// 为指定主体签发访问令牌。
    pub fn sign(&self, _subject: &str, _ttl_secs: u64) -> Result<String> {
        todo!("jsonwebtoken 签发")
    }
}

/// JWT 校验器（骨架占位）。
#[derive(Debug)]
pub struct JwtVerifier;

impl JwtVerifier {
    /// 使用配置密钥创建校验器。
    pub fn new(
        _secret: impl Into<String>,
        _issuer: impl Into<String>,
        _audience: impl Into<String>,
    ) -> Self {
        todo!("保存校验参数")
    }

    /// 校验令牌并返回 Claims。
    pub fn verify(&self, _token: &str) -> Result<JwtClaims> {
        todo!("jsonwebtoken 校验：签名/过期/iss/aud")
    }
}
