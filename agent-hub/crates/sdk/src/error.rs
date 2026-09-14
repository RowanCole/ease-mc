//! SDK 错误类型。

use thiserror::Error;

/// SDK 调用错误。
#[derive(Debug, Error)]
pub enum SdkError {
    /// HTTP 传输错误。
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// 网关返回的非 2xx 响应。
    #[error("api error: {status} {message}")]
    Api {
        /// HTTP 状态码。
        status: u16,
        /// 错误信息。
        message: String,
    },

    /// 响应解析失败。
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

/// SDK Result 别名。
pub type Result<T> = std::result::Result<T, SdkError>;
