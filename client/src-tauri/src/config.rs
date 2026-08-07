use std::path::PathBuf;
use tracing::{debug, info};

/// 应用数据根目录。
///
/// - dev 构建（cargo run / tauri dev）：cwd 为 src-tauri，config.json 与 game 目录在其下；
/// - release 构建（打包安装）：cwd 不固定，统一以 exe 所在目录为准，
///   这样用户把 config.json 放在 exe 同级即可生效，game 目录也在 exe 同级。
pub fn app_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        std::env::current_dir().unwrap_or_default()
    }
    #[cfg(not(debug_assertions))]
    {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    }
}

#[tauri::command]
pub fn get_config(key: &str) -> Result<String, String> {
    let path = app_dir().join("config.json");
    info!("get_config 调用: key={}, 路径={}", key, path.display());
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取配置文件失败 ({}): {}", path.display(), e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    let value = json[key]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("config.json 缺少 {} 字段", key))?;
    info!("get_config 返回: {} = {}", key, value);
    debug!("读取配置 {} = {}", key, value);
    Ok(value)
}

#[tauri::command]
pub fn set_config(key: &str, value: &str) -> Result<(), String> {
    let path = app_dir().join("config.json");
    debug!("写入配置 {} = {} -> {}", key, value, path.display());
    let mut json: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?
    } else {
        serde_json::json!({})
    };
    json[key] = serde_json::Value::String(value.to_string());
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化配置文件失败: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("写入配置文件失败 ({}): {}", path.display(), e))?;
    Ok(())
}
