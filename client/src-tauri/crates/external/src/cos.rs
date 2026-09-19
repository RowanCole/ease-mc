use std::collections::HashMap;

use tauri::path;

const COS_REGION: &str = "ap-beijing";
const COS_BUCKET: &str = "ease-mc-1257344929";
const COS_SECRET_ID: &str = "AKIDa1X5X000000000000000000000000000";
const COS_SECRET_KEY: &str = "00000000000000000000000000000000";

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
        // TODO: 实现下载授权生成
        todo!()
    }

    fn generate_time_params(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let end = start + 70000;
        format!("{};{}", start, end)
    }

    fn process_list(&self, params: Option<&HashMap<String, String>>) -> (String, String) {
        if let Some(params) = params {
            let mut sorted_params: Vec<_> = params.iter().collect();
            sorted_params.sort_by_key(|(k, _)| *k);
            let mut url_param_list = vec![];
            let mut http_param_list = vec![];
            for (key,value) in sorted_params {
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

    fn build_HttpString(&self, method: &str, url_pathname: &str, http_parameters: &str, http_headers: &str) -> String {
        format!("{}\n{}\n{}\n{}\n", method.to_lowercase(), url_pathname, http_parameters, http_headers)
    }

    fn build_StringToSign(&self, key_time: &str, http_string: &str) -> String {
        use sha1::{Digest,Sha1};
        let mut hasher = Sha1::digest(http_string);
        format!("sha1\n{}\n{}\n", key_time, hex::encode(hasher))
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
    

    fn calculate_signature(&self, 
        method: &str,
        path: &str,
        params: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> String {
        let key_time = self.generate_time_params();
        let (url_param_list,http_params) = self.process_list(params);
        let (header_list,http_headers) = self.process_list(headers);
        let sign_key = self.build_SignKey(&key_time);
        let http_string = self.build_HttpString(&method, path, &http_params, &http_headers);
        let sign_str = self.build_StringToSign(&key_time, &http_string);
        format!("q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}&q-header-list={}&q-url-param-list={}&q-signature={}",
            self.secret_id, key_time, key_time, header_list, url_param_list, self.build_Signature(&sign_key, &sign_str)
        )
    }

    fn build_authorization(
        &self,
        q_key_time: &str,
        q_sign_time: &str,
        url_param_list: &str,
        signature: &str,
    ) -> String {
        // TODO: 实现 Authorization 字符串构建
        todo!()
    }

    // 最简单的下载接口（无查询参数）
    pub fn get_auth_for_download(&self, key: &str) -> String {
        // TODO: 实现简单下载授权
        todo!()
    }
}

pub async fn get_object(key: String) {
    // TODO: 实现文件下载
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_client() -> CosClient {
        CosClient::new(
            "test-secret-id".into(),
            "test-secret-key".into(),
            "test-bucket-1250000000".into(),
            "ap-beijing".into(),
        )
    }

    // =====================================================================
    // new
    // =====================================================================
    #[test]
    fn new_stores_all_fields() {
        let c = CosClient::new(
            "id".into(),
            "key".into(),
            "bucket".into(),
            "region".into(),
        );
        assert_eq!(c.secret_id, "id");
        assert_eq!(c.secret_key, "key");
        assert_eq!(c.bucket, "bucket");
        assert_eq!(c.region, "region");
    }

    // =====================================================================
    // generate_time_params
    // =====================================================================
    #[test]
    fn generate_time_params_format_is_start_semicolon_end() {
        let c = make_client();
        let s = c.generate_time_params();
        let parts: Vec<&str> = s.split(';').collect();
        assert_eq!(parts.len(), 2, "应为 `start;end` 格式");
    }

    #[test]
    fn generate_time_params_window_is_70000ms() {
        let c = make_client();
        let s = c.generate_time_params();
        let (start, end) = s.split_once(';').unwrap();
        let start: u128 = start.parse().expect("start 应为数字");
        let end: u128 = end.parse().expect("end 应为数字");
        assert_eq!(end - start, 70_000, "有效期应为 70000ms");
    }

    // =====================================================================
    // process_list
    // =====================================================================
    #[test]
    fn process_list_none_returns_empty_pair() {
        let c = make_client();
        let (url, http) = c.process_list(None);
        assert_eq!(url, "");
        assert_eq!(http, "");
    }

    #[test]
    fn process_list_empty_map_returns_empty_pair() {
        let c = make_client();
        let map: HashMap<String, String> = HashMap::new();
        let (url, http) = c.process_list(Some(&map));
        assert_eq!(url, "");
        assert_eq!(http, "");
    }

    #[test]
    fn process_list_is_sorted_by_key() {
        let c = make_client();
        let mut map = HashMap::new();
        map.insert("c".into(), "3".into());
        map.insert("a".into(), "1".into());
        map.insert("b".into(), "2".into());
        let (url, http) = c.process_list(Some(&map));
        assert_eq!(url, "a;b;c");
        assert_eq!(http, "a=1&b=2&c=3");
    }

    #[test]
    fn process_list_single_entry() {
        let c = make_client();
        let mut map = HashMap::new();
        map.insert("key".into(), "value".into());
        let (url, http) = c.process_list(Some(&map));
        assert_eq!(url, "key");
        assert_eq!(http, "key=value");
    }

    // =====================================================================
    // build_HttpString
    // =====================================================================
    #[test]
    fn build_http_string_lowercases_method() {
        let c = make_client();
        let s = c.build_HttpString("GET", "/a.txt", "x=1", "host=example.com");
        assert_eq!(s, "get\n/a.txt\nx=1\nhost=example.com\n");
    }

    #[test]
    fn build_http_string_preserves_path_case() {
        let c = make_client();
        let s = c.build_HttpString("PUT", "/MyFile.txt", "", "");
        assert_eq!(s, "put\n/MyFile.txt\n\n\n");
    }

    #[test]
    fn build_http_string_ends_with_newline_and_has_four() {
        let c = make_client();
        let s = c.build_HttpString("GET", "/", "", "");
        assert!(s.ends_with('\n'));
        assert_eq!(s.matches('\n').count(), 4, "应有 4 个换行分隔 4 段");
    }

    // =====================================================================
    // build_SignKey
    // =====================================================================
    // RFC 2202 HMAC-SHA-1 测试向量 2：
    //   HMAC-SHA1(key="Jefe", data="what do ya want for nothing?")
    //   = effcdf6ae5eb2fa2d27416d5f184df9c259a7c79
    #[test]
    fn build_sign_key_matches_rfc2202_vector() {
        let c = CosClient::new(
            String::new(),
            "Jefe".into(),
            String::new(),
            String::new(),
        );
        let out = c.build_SignKey("what do ya want for nothing?");
        assert_eq!(out, "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79");
    }

    #[test]
    fn build_sign_key_output_is_40_hex_chars() {
        let c = make_client();
        let out = c.build_SignKey("1234567890;1234567899");
        assert_eq!(out.len(), 40);
        assert!(out.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn build_sign_key_differs_by_key_time() {
        let c = make_client();
        let a = c.build_SignKey("t1;t2");
        let b = c.build_SignKey("t3;t4");
        assert_ne!(a, b);
    }

    // =====================================================================
    // build_StringToSign
    // =====================================================================
    // FIPS 180-1: SHA1("abc") = a9993e364706816aba3e25717850c26c9cd0d89d
    #[test]
    fn build_string_to_sign_format_and_sha1() {
        let c = make_client();
        let key_time = "1700000000000;1700000070000";
        let s = c.build_StringToSign(key_time, "abc");
        assert_eq!(
            s,
            format!(
                "sha1\n{}\na9993e364706816aba3e25717850c26c9cd0d89d\n",
                key_time
            )
        );
    }

    #[test]
    fn build_string_to_sign_starts_with_sha1_alg() {
        let c = make_client();
        let s = c.build_StringToSign("t", "");
        assert!(s.starts_with("sha1\n"));
        assert!(s.ends_with('\n'));
    }

    // =====================================================================
    // build_Signature
    // =====================================================================
    #[test]
    fn build_signature_matches_rfc2202_vector() {
        let c = make_client();
        let out = c.build_Signature("Jefe", "what do ya want for nothing?");
        assert_eq!(out, "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79");
    }

    #[test]
    fn build_signature_differs_by_message() {
        let c = make_client();
        let a = c.build_Signature("k", "msg-a");
        let b = c.build_Signature("k", "msg-b");
        assert_ne!(a, b);
    }

    #[test]
    fn build_signature_differs_by_sign_key() {
        let c = make_client();
        let a = c.build_Signature("key-a", "msg");
        let b = c.build_Signature("key-b", "msg");
        assert_ne!(a, b);
    }

    // =====================================================================
    // calculate_signature （集成）
    // =====================================================================
    #[test]
    fn calculate_signature_contains_all_required_fields() {
        let c = make_client();
        let sig = c.calculate_signature("get", "/test.txt", None, None);
        assert!(sig.contains("q-sign-algorithm=sha1"));
        assert!(sig.contains(&format!("q-ak={}", c.secret_id)));
        assert!(sig.contains("q-sign-time="));
        assert!(sig.contains("q-key-time="));
        assert!(sig.contains("q-header-list="));
        assert!(sig.contains("q-url-param-list="));
        assert!(sig.contains("q-signature="));
    }

    #[test]
    fn calculate_signature_url_param_list_reflects_params() {
        let c = make_client();
        let mut params = HashMap::new();
        params.insert("response-content-type".into(), "text/plain".into());
        let sig = c.calculate_signature("get", "/test.txt", Some(&params), None);
        assert!(sig.contains("q-url-param-list=response-content-type"));
    }

    #[test]
    fn calculate_signature_header_list_reflects_headers() {
        let c = make_client();
        let mut headers = HashMap::new();
        headers.insert("host".into(), "example.com".into());
        let sig = c.calculate_signature("get", "/test.txt", None, Some(&headers));
        assert!(sig.contains("q-header-list=host"));
    }

    #[test]
    fn calculate_signature_q_signature_is_40_hex_chars() {
        let c = make_client();
        let sig = c.calculate_signature("get", "/", None, None);
        let value = sig
            .split("q-signature=")
            .nth(1)
            .expect("应包含 q-signature");
        assert_eq!(value.len(), 40);
        assert!(value.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn calculate_signature_method_case_does_not_matter() {
        // 当前实现最终会 to_lowercase，所以 GET 与 get 应产生相同结果
        // （注意：若之后修复为按 COS 要求仅小写，此测试仍需成立）
        let c = make_client();
        let a = c.calculate_signature("get", "/", None, None);
        let b = c.calculate_signature("GET", "/", None, None);
        // 仅比较 q-signature 部分
        let extract = |s: &str| {
            s.split("q-signature=").nth(1).unwrap().to_string()
        };
        // 由于时间戳不同，只能比较结构，不能比较值；这里验证都能解析出 40 位 hex
        assert_eq!(extract(&a).len(), 40);
        assert_eq!(extract(&b).len(), 40);
    }

    // =====================================================================
    // 未实现函数：实现后移除 #[ignore]
    // =====================================================================
    #[test]
    #[ignore = "尚未实现：create_download_auth 会 panic"]
    fn create_download_auth_placeholder() {
        let c = make_client();
        let _ = c.create_download_auth("test.txt", None);
    }

    #[test]
    #[ignore = "尚未实现：build_authorization 会 panic"]
    fn build_authorization_placeholder() {
        let c = make_client();
        let _ = c.build_authorization("t", "t", "", "");
    }

    #[test]
    #[ignore = "尚未实现：get_auth_for_download 会 panic"]
    fn get_auth_for_download_placeholder() {
        let c = make_client();
        let _ = c.get_auth_for_download("test.txt");
    }

    // get_object 是 async 自由函数，这里无法直接用 #[test]。
    // 实现后建议加：
    //
    // #[tokio::test]
    // async fn get_object_should_download() {
    //     get_object("test.txt".into()).await;
    // }
}












