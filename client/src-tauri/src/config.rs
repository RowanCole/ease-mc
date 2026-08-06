use tracing::{debug};

#[tauri::command]
pub fn get_config(key: &str) -> Result<String, String> {
    let content = std::fs::read_to_string("config.json")
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    let value = json[key]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("config.json 缺少 {} 字段", key))?;
    debug!("读取配置 {} = {}", key, value);
    Ok(value)
}

#[tauri::command]
pub fn set_config(key: &str, value: &str) -> Result<(), String> {
    debug!("写入配置 {} = {}", key, value);
    let mut json: serde_json::Value = if std::path::Path::new("config.json").exists() {
        let content = std::fs::read_to_string("config.json")
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?
    } else {
        serde_json::json!({})
    };
    json[key] = serde_json::Value::String(value.to_string());
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化配置文件失败: {}", e))?;
    std::fs::write("config.json", content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}
