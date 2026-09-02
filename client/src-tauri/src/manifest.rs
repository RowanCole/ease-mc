// Minecraft 版本清单解析：从编译期嵌入的 game.json 提取元数据与下载任务。
// 全项目唯一的版本数据源（下载侧与启动侧共用，避免硬编码 1.21.1 / assetIndex）。
use std::path::Path;

use serde_json::Value;
use tracing::info;

use crate::net::DownloadTask;

/// 编译期嵌入的 Minecraft 1.21.1 版本清单
pub const EMBEDDED_MANIFEST: &str = include_str!("game.json");

/// 解析编译期嵌入的清单
pub fn load_embedded() -> Result<Value, String> {
    serde_json::from_str(EMBEDDED_MANIFEST)
        .map_err(|e| format!("解析 game.json 失败: {}", e))
}

/// 版本 id（如 1.21.1）
pub fn version_id(manifest: &Value) -> Result<String, String> {
    manifest["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "game.json 缺少 id 字段".to_string())
}

/// 资源索引 id（如 17）
pub fn asset_index_id(manifest: &Value) -> Result<String, String> {
    manifest["assetIndex"]["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "game.json 缺少 assetIndex.id 字段".to_string())
}

/// 资源索引文件下载地址
pub fn asset_index_url(manifest: &Value) -> Option<String> {
    manifest["assetIndex"]["url"].as_str().map(|s| s.to_string())
}

/// 当前操作系统在 Minecraft 清单中的名称（osx / windows / linux）。
pub fn current_os_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "osx",
        _ => "linux",
    }
}

/// 判断库的 rules 是否允许在当前平台下载。无 rules 视为允许。
fn os_rules_allow(lib: &Value, os_name: &str) -> bool {
    let Some(rules) = lib["rules"].as_array() else {
        return true;
    };

    let mut allow = false;
    for rule in rules {
        let os = &rule["os"];
        let os_match = if os.is_null() {
            true
        } else {
            let name_match = os["name"]
                .as_str()
                .map(|n| n == os_name)
                .unwrap_or(true);
            let arch_match = os["arch"]
                .as_str()
                .map(|a| a == std::env::consts::ARCH)
                .unwrap_or(true);
            name_match && arch_match
        };
        if os_match {
            allow = rule["action"].as_str() == Some("allow");
        }
    }
    allow
}

/// 第三方库下载任务：依据平台 rules 过滤出本平台需要的主 artifact。
pub fn library_tasks(
    manifest: &Value,
    os_name: &str,
    minecraft_dir: &Path,
) -> Result<Vec<DownloadTask>, String> {
    let libraries = manifest["libraries"]
        .as_array()
        .ok_or_else(|| "game.json 缺少 libraries 字段".to_string())?;

    let mut tasks = Vec::new();
    for lib in libraries {
        if !os_rules_allow(lib, os_name) {
            continue;
        }
        let artifact = &lib["downloads"]["artifact"];
        let (path, url) = match (artifact["path"].as_str(), artifact["url"].as_str()) {
            (Some(path), Some(url)) => (path, url),
            // 仅含 natives 分类器、没有主 artifact 的库，跳过
            _ => continue,
        };
        let dest = minecraft_dir.join("libraries").join(path);
        let size = artifact["size"].as_u64().unwrap_or(0);
        tasks.push((url.to_string(), dest, size));
    }
    Ok(tasks)
}

/// 客户端主 jar 下载任务
pub fn client_jar_task(
    manifest: &Value,
    minecraft_dir: &Path,
) -> Result<Option<DownloadTask>, String> {
    let version = version_id(manifest)?;
    let Some(url) = manifest["downloads"]["client"]["url"].as_str() else {
        return Ok(None);
    };
    let dest = minecraft_dir
        .join("versions")
        .join(&version)
        .join(format!("{version}.jar"));
    let size = manifest["downloads"]["client"]["size"].as_u64().unwrap_or(0);
    Ok(Some((url.to_string(), dest, size)))
}

/// 从资源索引解析出全部资源对象下载任务（占大头，约 800MB+）。
pub fn asset_object_tasks(index: &Value, minecraft_dir: &Path) -> Vec<DownloadTask> {
    let Some(objects) = index["objects"].as_object() else {
        return Vec::new();
    };

    let mut tasks = Vec::new();
    for (_name, obj) in objects {
        let hash = obj["hash"].as_str().unwrap_or("");
        if hash.len() < 2 {
            continue;
        }
        let url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            &hash[..2],
            hash
        );
        let dest = minecraft_dir
            .join("assets")
            .join("objects")
            .join(&hash[..2])
            .join(hash);
        let size = obj["size"].as_u64().unwrap_or(0);
        tasks.push((url, dest, size));
    }
    info!("资源索引包含 {} 个资源对象", tasks.len());
    tasks
}

/// 日志配置文件下载任务（可能缺失）。
pub fn logging_task(manifest: &Value, minecraft_dir: &Path) -> Option<DownloadTask> {
    let file = &manifest["logging"]["client"]["file"];
    let url = file["url"].as_str()?;
    let file_id = file["id"].as_str().unwrap_or("client.xml");
    let dest = minecraft_dir.join("assets").join("log_configs").join(file_id);
    let size = file["size"].as_u64().unwrap_or(0);
    Some((url.to_string(), dest, size))
}
