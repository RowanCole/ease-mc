use tauri::Manager;
use tracing::{debug, info};

use crate::paths;

/// 确保运行目录存在配置文件；缺失时从打包资源目录复制初始配置。
fn ensure_config_file(app: &tauri::AppHandle) -> Result<(), String> {
    let cfg_path = paths::config_path()?;
    if cfg_path.exists() {
        return Ok(());
    }
    let resource_cfg = app
        .path()
        .resource_dir()
        .map_err(|e| format!("获取资源目录失败: {}", e))?
        .join("config.json");
    if resource_cfg.exists() {
        if let Some(parent) = cfg_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
        std::fs::copy(&resource_cfg, &cfg_path)
            .map_err(|e| format!("复制初始配置失败: {}", e))?;
        debug!("已从资源目录初始化配置: {:?}", cfg_path);
    }
    Ok(())
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle, key: &str) -> Result<String, String> {
    get_config_inner(Some(&app), key)
}

pub fn get_config_inner(app: Option<&tauri::AppHandle>, key: &str) -> Result<String, String> {
    if let Some(handle) = app {
        ensure_config_file(handle)?;
    }
    info!("get_config 调用: key={}", key);
    let cfg_path = paths::config_path()?;
    let content = std::fs::read_to_string(&cfg_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
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
pub fn set_config(app: tauri::AppHandle, key: &str, value: &str) -> Result<(), String> {
    set_config_inner(Some(&app), key, value)
}

/// 写入配置值（app 为 None 时回退当前目录，供测试/无窗口上下文场景使用）
pub fn set_config_inner(
    app: Option<&tauri::AppHandle>,
    key: &str,
    value: &str,
) -> Result<(), String> {
    if let Some(handle) = app {
        ensure_config_file(handle)?;
    }
    debug!("写入配置 {} = {}", key, value);
    let cfg_path = paths::config_path()?;
    let mut json: serde_json::Value = if cfg_path.exists() {
        let content = std::fs::read_to_string(&cfg_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?
    } else {
        serde_json::json!({})
    };
    json[key] = serde_json::Value::String(value.to_string());
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化配置文件失败: {}", e))?;
    std::fs::write(&cfg_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}
