use std::path::PathBuf;
use tauri::Manager;
use tracing::{debug, info};

/// 解析 config.json 的实际路径：有 AppHandle 时使用用户配置目录
/// （app_config_dir，可写且按用户隔离），无 AppHandle 时回退当前目录
/// （供测试场景使用）。
fn config_path(app: Option<&tauri::AppHandle>) -> Result<PathBuf, String> {
    match app {
        Some(handle) => {
            let dir = handle
                .path()
                .app_config_dir()
                .map_err(|e| format!("获取配置目录失败: {}", e))?;
            Ok(dir.join("config.json"))
        }
        None => Ok(std::path::PathBuf::from("config.json")),
    }
}

/// 确保用户配置目录下存在 config.json：若不存在，则从随应用分发的
/// 资源目录（resource_dir，dev 模式下即 src-tauri 目录）复制初始配置，
/// 保证 macJrePath / winJrePath 等初始值可读。
fn ensure_config_file(app: &tauri::AppHandle) -> Result<(), String> {
    let cfg_path = config_path(Some(app))?;
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

/// 读取配置值（app 为 None 时回退当前目录，供测试/无窗口上下文场景使用）
pub fn get_config_inner(app: Option<&tauri::AppHandle>, key: &str) -> Result<String, String> {
    if let Some(handle) = app {
        ensure_config_file(handle)?;
    }
    info!("get_config 调用: key={}", key);
    let cfg_path = config_path(app)?;
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
    let cfg_path = config_path(app)?;
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
