use external::cos::CosClient;

/// COS 下载源。未配置 cosBucket/cosRegion 时视为未启用。
pub struct CosSource {
    client: CosClient,
    signed: bool,
}

impl CosSource {
    /// 优先读 config.json，为空时回退到环境变量（.env）
    fn config_or_env(app: Option<&tauri::AppHandle>, key: &str, env: &str) -> String {
        let from_config = mc_core::config::get_config(app, key)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !from_config.is_empty() {
            return from_config;
        }
        std::env::var(env)
            .map(|v| v.trim().to_string())
            .unwrap_or_default()
    }

    /// 从 config.json / 环境变量构造；缺少 cosBucket/cosRegion 返回 None。
    pub fn from_config(app: Option<&tauri::AppHandle>) -> Option<Self> {
        let bucket = Self::config_or_env(app, "cosBucket", "COS_BUCKET");
        let region = Self::config_or_env(app, "cosRegion", "COS_REGION");
        if bucket.is_empty() || region.is_empty() {
            return None;
        }
        let secret_id = Self::config_or_env(app, "cosSecretId", "COS_SECRET_ID");
        let secret_key = Self::config_or_env(app, "cosSecretKey", "COS_SECRET_KEY");

        let signed = !secret_id.is_empty() && !secret_key.is_empty();

        Some(Self {
            client: CosClient::new(secret_id, secret_key, bucket, region),
            // 桶为私有读时需要凭据做预签名
            signed,
        })
    }

    /// COS 对象 key → 可下载 URL（私有桶自动预签名）。
    pub fn object_url(&self, key: &str) -> String {
        if self.signed {
            self.client.create_download_auth(key, None)
        } else {
            self.client.public_object_url(key)
        }
    }
}

/// 把配置值解析为可下载 URL：`cos://对象key` 走 COS 源，其余按普通 HTTP URL 原样返回。
pub fn resolve_config_url(
    cos: Option<&CosSource>,
    value: &str,
    config_key: &str,
) -> Result<String, String> {
    if let Some(obj_key) = value.strip_prefix("cos://") {
        let cos = cos.ok_or_else(|| {
            format!(
                "{} 使用 cos:// 源，但未配置 COS（cosBucket/cosRegion）",
                config_key
            )
        })?;
        Ok(cos.object_url(obj_key))
    } else {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> CosSource {
        CosSource {
            client: CosClient::new(
                String::new(),
                String::new(),
                "demo-1250000000".to_string(),
                "ap-beijing".to_string(),
            ),
            signed: false,
        }
    }

    #[test]
    fn object_url_maps_to_cos_host() {
        assert_eq!(
            source().object_url("minecraft/game.zip"),
            "https://demo-1250000000.cos.ap-beijing.myqcloud.com/minecraft/game.zip"
        );
    }

    #[test]
    fn resolve_config_url_passes_http_through() {
        let url = resolve_config_url(None, "https://cdn.example.com/jre.zip", "winJrePath").unwrap();
        assert_eq!(url, "https://cdn.example.com/jre.zip");
    }

    #[test]
    fn resolve_config_url_requires_cos_for_cos_scheme() {
        let err = resolve_config_url(None, "cos://minecraft/game.zip", "gameZipPath").unwrap_err();
        assert!(err.contains("gameZipPath"));
        let url =
            resolve_config_url(Some(&source()), "cos://minecraft/game.zip", "gameZipPath").unwrap();
        assert_eq!(
            url,
            "https://demo-1250000000.cos.ap-beijing.myqcloud.com/minecraft/game.zip"
        );
    }
}
