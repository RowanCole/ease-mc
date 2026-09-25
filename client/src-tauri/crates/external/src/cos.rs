use std::collections::HashMap;

pub struct CosClient {
    secret_id: String,
    secret_key: String,
    bucket: String,
    region: String,
}

impl CosClient {
    pub fn new(secret_id: String, secret_key: String, bucket: String, region: String) -> Self {
        CosClient {
            secret_id,
            secret_key,
            bucket,
            region,
        }
    }


    pub fn create_download_auth(
        &self,
        key: &str,
        query_params: Option<&HashMap<String, String>>,
    ) -> String {
        let path = format!("/{}", key.trim_start_matches('/'));
        let host = format!("{}.cos.{}.myqcloud.com", self.bucket, self.region);
        let base = format!("https://{}{}", host, path);

        // 签名需覆盖额外参数，且 URL 上按字典序放置这些参数以与服务端校验一致
        let signature = self.calculate_signature("GET", &path, query_params, None);
        let mut url = base;
        let mut has_query = false;
        if let Some(params) = query_params {
            let mut sorted: Vec<_> = params.iter().collect();
            sorted.sort_by_key(|(k, _)| *k);
            for (k, v) in sorted {
                url.push_str(if has_query { "&" } else { "?" });
                url.push_str(&format!("{}={}", k, v));
                has_query = true;
            }
        }
        url.push_str(if has_query { "&" } else { "?" });
        url.push_str(&signature);
        url
    }

    fn generate_time_params(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let end = start + 3600;
        format!("{};{}", start, end)
    }

    fn process_list(&self, params_map: Option<&HashMap<String, String>>) -> (String, String) {
        if let Some(params) = params_map {
            let mut sorted_params: Vec<_> = params.iter().collect();
            sorted_params.sort_by_key(|(k, _)| *k);
            let mut url_param_list = vec![];
            let mut http_param_list = vec![];
            for (key, value) in sorted_params {
                url_param_list.push(key.to_string());
                http_param_list.push(format!("{}={}", key, value));
            }
            (url_param_list.join(";"), http_param_list.join("&"))
        } else {
            (String::new(), String::new())
        }
    }

    fn build_SignKey(&self, key_time: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        type HmcSha1 = Hmac<Sha1>;
        let mut mac = HmcSha1::new_from_slice(self.secret_key.as_bytes()).unwrap();
        mac.update(key_time.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    fn build_HttpString(
        &self,
        method: &str,
        url_pathname: &str,
        http_parameters: &str,
        http_headers: &str,
    ) -> String {
        format!(
            "{}\n{}\n{}\n{}\n",
            method.to_lowercase(),
            url_pathname,
            http_parameters,
            http_headers
        )
    }

    fn build_StringToSign(&self, key_time: &str, http_string: &str) -> String {
        use sha1::{Digest, Sha1};
        let mut result = Sha1::digest(http_string);
        format!("sha1\n{}\n{}\n", key_time, hex::encode(result))
    }

    fn build_Signature(&self, sign_key: &str, string_to_sign: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        type HmcSha1 = Hmac<Sha1>;
        let mut mac = HmcSha1::new_from_slice(sign_key.as_bytes()).unwrap();
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    fn calculate_signature(
        &self,
        method: &str,
        path: &str,
        params: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> String {
        let key_time = self.generate_time_params();
        let (url_param_list, http_params) = self.process_list(params);
        let (header_list, http_headers) = self.process_list(headers);
        let sign_key = self.build_SignKey(&key_time);
        let http_string = self.build_HttpString(&method, path, &http_params, &http_headers);
        let sign_str = self.build_StringToSign(&key_time, &http_string);
        format!("q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list={}&q-signature={}",
            self.secret_id, key_time, key_time, header_list, url_param_list, self.build_Signature(&sign_key, &sign_str)
        )
    }

    /// 获取存储桶列表（GET Service）
    /// 请求地址为 https://service.cos.myqcloud.com/，签名中的 UriPathname 为 /
    /// 返回接口响应体（XML 格式的存储桶列表）
    pub async fn get_service(&self) -> Result<String, String> {
        let signature = self.calculate_signature("GET", "/", None, None);
        let resp = reqwest::Client::new()
            .get("https://service.cos.myqcloud.com/")
            .header("Authorization", signature)
            .send()
            .await
            .map_err(|e| format!("请求 COS GET Service 失败: {}", e))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("读取 COS 响应失败: {}", e))?;

        if !status.is_success() {
            return Err(format!("COS GET Service 返回 {}: {}", status, body));
        }

        Ok(body)
    }

    /// 对象的公开访问 URL（桶为公共读时可直接下载，无需签名）
    pub fn public_object_url(&self, key: &str) -> String {
        let path = format!("/{}", key.trim_start_matches('/'));
        let host = format!("{}.cos.{}.myqcloud.com", self.bucket, self.region);
        format!("https://{}{}", host, path)
    }

    /// 下载对象并返回其字节流（逐块读取）。
    pub async fn download_stream(
        &self,
        key: &str,
    ) -> Result<impl futures_util::Stream<Item = Result<Vec<u8>, String>>, String> {
        use futures_util::StreamExt;

        let path = format!("/{}", key.trim_start_matches('/'));
        let host = format!("{}.cos.{}.myqcloud.com", self.bucket, self.region);
        let mut header = HashMap::new();
        header.insert("host".to_string(), host.clone());
        let auth = self.calculate_signature("GET", &path, None, Some(&header));
        let resp = reqwest::Client::new()
            .get(format!("https://{}{}", host, path))
            .header("Authorization", auth)
            .send()
            .await
            .map_err(|e| format!("请求 COS GET Object 失败: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("COS GET Object 返回 {}: {}", status, body));
        }

        Ok(resp.bytes_stream().map(|chunk| {
            chunk
                .map(|bytes| bytes.to_vec())
                .map_err(|e| format!("读取 COS 对象流失败: {}", e))
        }))
    }

    /// 下载对象到本地文件，返回写入的字节数。
    pub async fn download_to_file(
        &self,
        key: &str,
        dest: &std::path::Path,
    ) -> Result<u64, String> {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("创建目录 {} 失败: {}", parent.display(), e))?;
        }

        let mut stream = self.download_stream(key).await?;
        let mut file = tokio::fs::File::create(dest)
            .await
            .map_err(|e| format!("创建文件 {} 失败: {}", dest.display(), e))?;

        let mut written: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            file.write_all(&bytes)
                .await
                .map_err(|e| format!("写入 {} 失败: {}", dest.display(), e))?;
            written += bytes.len() as u64;
        }
        file.flush()
            .await
            .map_err(|e| format!("刷新 {} 失败: {}", dest.display(), e))?;
        Ok(written)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 真实凭证通过环境变量注入，避免提交到仓库：
    // COS_SECRET_ID / COS_SECRET_KEY / COS_BUCKET / COS_REGION / COS_TEST_KEY
    fn env_or_skip(name: &str) -> String {
        match std::env::var(name) {
            Ok(v) if !v.is_empty() => v,
            _ => panic!("缺少环境变量 {}，跳过集成测试", name),
        }
    }

    /// 该测试会真实请求腾讯云接口，需要有效的凭证
    #[tokio::test]
    async fn test_get_service() {
        let client = CosClient::new(
            env_or_skip("COS_SECRET_ID"),
            env_or_skip("COS_SECRET_KEY"),
            env_or_skip("COS_BUCKET"),
            env_or_skip("COS_REGION"),
        );

        match client.get_service().await {
            Ok(body) => {
                println!("GET Service 响应体: {}", body);
                assert!(
                    body.contains("<ListAllMyBucketsResult>"),
                    "响应体不是存储桶列表 XML: {}",
                    body
                );
            }
            Err(e) => panic!("调用 GET Service 失败: {}", e),
        }
    }

    /// 该测试会真实请求腾讯云接口，需要有效的凭证与对象 key
    #[tokio::test]
    async fn test_download_stream() {
        let client = CosClient::new(
            env_or_skip("COS_SECRET_ID"),
            env_or_skip("COS_SECRET_KEY"),
            env_or_skip("COS_BUCKET"),
            env_or_skip("COS_REGION"),
        );
        let token = env_or_skip("COS_TEST_KEY");

        match client.download_stream(&token).await {
            Ok(mut stream) => {
                use futures_util::StreamExt;

                let mut total = 0usize;
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            total += bytes.len();
                            println!("收到分片: {} 字节", bytes.len());
                        }
                        Err(e) => panic!("读取对象流失败: {}", e),
                    }
                }
                println!("下载完成，共 {} 字节", total);
            }
            Err(e) => panic!("调用 download_stream 失败: {}", e),
        }
    }

    /// 该测试会真实请求腾讯云接口，需要有效的凭证与对象 key
    #[tokio::test]
    async fn test_download_to_file() {
        let client = CosClient::new(
            env_or_skip("COS_SECRET_ID"),
            env_or_skip("COS_SECRET_KEY"),
            env_or_skip("COS_BUCKET"),
            env_or_skip("COS_REGION"),
        );
        let token = env_or_skip("COS_TEST_KEY");

        let dest = std::env::temp_dir().join("cos_download_test.bin");
        let written = client
            .download_to_file(&token, &dest)
            .await
            .unwrap_or_else(|e| panic!("调用 download_to_file 失败: {}", e));
        println!("已写入文件 {}，共 {} 字节", dest.display(), written);
        assert!(written > 0, "下载内容为空");
        assert!(dest.exists(), "目标文件不存在: {}", dest.display());
        let _ = std::fs::remove_file(&dest);
    }
}
