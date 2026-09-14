//! API Key 生成与校验（骨架占位）。

use common::Result;

/// API Key 及其哈希（骨架占位）。
#[derive(Debug, Clone)]
pub struct ApiKey {
    /// 明文 Key（仅签发时可见）。
    pub plaintext: String,
    /// 服务端存储的哈希。
    pub hash: String,
}

/// 生成新 API Key（`<prefix>_<随机串>`）。
pub fn generate(_prefix: &str) -> String {
    todo!("rand 生成随机 Key")
}

/// 使用 Argon2id 计算 Key 哈希。
pub fn hash(_plaintext: &str) -> Result<String> {
    todo!("argon2 哈希")
}

/// 校验明文 Key 与哈希是否匹配。
pub fn verify(_plaintext: &str, _hash: &str) -> Result<bool> {
    todo!("argon2 常量时间校验")
}
