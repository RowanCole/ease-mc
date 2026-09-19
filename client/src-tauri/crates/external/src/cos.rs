use std::collections::HashMap;

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

    fn build_sign_string(
        &self,
        key: &str,
        url_param_list: &str,
        http_parameters: &str,
        q_sign_time: &str,
    ) -> String {
        // TODO: 实现签名字符串构建
        todo!()
    }

    fn calculate_signature(&self, sign_str: &str, q_key_time: &str) -> String {
        // TODO: 实现签名计算
        todo!()
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














