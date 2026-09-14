//! 强类型标识符：避免不同领域的 ID 混用。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 基于 Uuid 的强类型 ID newtype。
macro_rules! define_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// 生成新 ID（UUID v7，按时间有序）。
            pub fn generate() -> Self {
                Self(Uuid::now_v7().to_string())
            }

            /// 以原始字符串构造（调试/测试辅助）。
            pub fn from_raw(raw: impl Into<String>) -> Self {
                Self(raw.into())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

define_id! {
    /// Agent 唯一标识。
    AgentId
}

define_id! {
    /// 调用方（租户）标识。
    TenantId
}

define_id! {
    /// 网关请求唯一标识（用于链路追踪）。
    RequestId
}
