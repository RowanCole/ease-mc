use std::path::Path;

use futures_util::StreamExt;
use reqwest::Client;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

use crate::config::set_config;
use crate::jre::download_jre;

/// 下载单个文件到目标路径，自动创建缺失的父目录。
pub async fn download_file(client: &Client, url: &str, dest: &Path) -> Result<(), String> {
    debug!("开始下载: {} -> {}", url, dest.display());
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建目录 {} 失败: {}", parent.display(), e))?;
    }

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求 {} 失败: {}", url, e))?;
    if !response.status().is_success() {
        return Err(format!("下载 {} 失败: HTTP {}", url, response.status()));
    }

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("创建文件 {} 失败: {}", dest.display(), e))?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取下载流失败: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入 {} 失败: {}", dest.display(), e))?;
    }
    file.flush()
        .await
        .map_err(|e| format!("刷新 {} 失败: {}", dest.display(), e))?;
    debug!("下载完成: {}", dest.display());
    Ok(())
}

/// 当前操作系统在 Minecraft 清单中的名称（osx / windows / linux）。
fn current_os_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "osx",
        _ => "linux",
    }
}

/// 判断库的 rules 是否允许在当前平台下载。无 rules 视为允许。
fn os_rules_allow(lib: &serde_json::Value, os_name: &str) -> bool {
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

/// 根据游戏版本清单（game.json）下载所有需要的文件到对应目录。
///
/// 目录结构（相对 minecraft_dir）：
/// - libraries/{path}          第三方库（按当前系统过滤 rules）
/// - versions/{id}/{id}.jar    客户端主 jar
/// - assets/indexes/{id}.json  资源索引
/// - assets/log_configs/{id}   日志配置文件
pub async fn download_game_files(
    client: &Client,
    manifest: &serde_json::Value,
    minecraft_dir: &Path,
) -> Result<(), String> {
    let os_name = current_os_name();
    info!(
        "开始下载游戏文件到 {:?}（平台: {}）",
        minecraft_dir,
        os_name
    );

    // 1. 第三方库
    let libraries = manifest["libraries"]
        .as_array()
        .ok_or_else(|| "game.json 缺少 libraries 字段".to_string())?;
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
        download_file(client, url, &dest).await?;
    }

    // 2. 客户端主 jar
    let version_id = manifest["id"]
        .as_str()
        .ok_or_else(|| "game.json 缺少 id 字段".to_string())?;
    if let Some(url) = manifest["downloads"]["client"]["url"].as_str() {
        let dest = minecraft_dir
            .join("versions")
            .join(version_id)
            .join(format!("{}.jar", version_id));
        info!("下载客户端主 jar: {}", dest.display());
        download_file(client, url, &dest).await?;
    }

    // 3. 资源索引
    if let Some(url) = manifest["assetIndex"]["url"].as_str() {
        let index_id = manifest["assetIndex"]["id"].as_str().unwrap_or("index");
        let dest = minecraft_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", index_id));
        info!("下载资源索引: {}", dest.display());
        download_file(client, url, &dest).await?;
    }

    // 4. 日志配置文件
    if let Some(url) = manifest["logging"]["client"]["file"]["url"].as_str() {
        let file_id = manifest["logging"]["client"]["file"]["id"]
            .as_str()
            .unwrap_or("client.xml");
        let dest = minecraft_dir
            .join("assets")
            .join("log_configs")
            .join(file_id);
        info!("下载日志配置: {}", dest.display());
        download_file(client, url, &dest).await?;
    }

    info!("游戏文件下载完成");
    Ok(())
}

/// 前端触发下载时调用（不再依赖本地静态服务器）：
/// 1. 解析项目内置的 game.json 版本清单（编译期嵌入，随应用发布）
/// 2. 按清单下载游戏文件到 game/.minecraft
/// 3. 按当前平台从 config.json 下载并解压 JRE 到 game/java
#[tauri::command]
pub async fn download_game(app: tauri::AppHandle) -> Result<(), String> {
    info!("=== 开始游戏下载流程 ===");
    let client = Client::new();

    // 1. 读取内置的 game.json 版本清单
    let manifest: serde_json::Value = serde_json::from_str(include_str!("game.json"))
        .map_err(|e| format!("解析 game.json 失败: {}", e))?;
    let version_id = manifest["id"].as_str().unwrap_or("unknown");
    info!("版本清单加载成功，版本: {}", version_id);

    // 2. 按清单下载游戏文件到 game/.minecraft
    let _ = app.emit("download-progress", serde_json::json!({ "percent": 5.0 }));
    let minecraft_dir = Path::new("game").join(".minecraft");
    download_game_files(&client, &manifest, &minecraft_dir).await?;
    let _ = app.emit("download-progress", serde_json::json!({ "percent": 60.0 }));

    // 3. 下载并解压 JRE 到 game/java
    let _ = app.emit("extract-start", ());
    download_jre(&client, Path::new("game")).await?;
    let _ = app.emit("download-progress", serde_json::json!({ "percent": 100.0 }));

    // 4. 标记安装完成
    set_config("gameIsInstalled", "true")?;
    info!("=== 游戏下载流程完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 将 game.json 中的游戏文件下载到当前目录下的 game/.minecraft 目录，仅验证下载功能。
    /// 运行：cargo test -- --nocapture download_game_files_to_game_dir
    #[tokio::test]
    async fn download_game_files_to_game_dir() {
        let client = Client::new();

        // game.json 位于 src/ 下，以 crate 根目录（client/src-tauri）定位
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("game.json");
        let content = std::fs::read_to_string(&manifest_path).expect("读取 game.json 失败");
        let manifest: serde_json::Value =
            serde_json::from_str(&content).expect("解析 game.json 失败");

        // 与 launch_game 使用的目录结构保持一致：./game/.minecraft
        let game_dir = Path::new("game").join(".minecraft");
        download_game_files(&client, &manifest, &game_dir)
            .await
            .expect("下载 game.json 中的文件失败");

        // 验证客户端主 jar
        let version_id = manifest["id"].as_str().expect("缺少 id 字段");
        let client_jar = game_dir
            .join("versions")
            .join(version_id)
            .join(format!("{}.jar", version_id));
        assert!(client_jar.exists(), "客户端 jar 未下载: {}", client_jar.display());

        // 验证资源索引
        let index_id = manifest["assetIndex"]["id"]
            .as_str()
            .expect("缺少 assetIndex.id 字段");
        let asset_index = game_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", index_id));
        assert!(asset_index.exists(), "资源索引未下载: {}", asset_index.display());

        // 验证日志配置
        let log_id = manifest["logging"]["client"]["file"]["id"]
            .as_str()
            .expect("缺少 logging.id 字段");
        let log_file = game_dir.join("assets").join("log_configs").join(log_id);
        assert!(log_file.exists(), "日志配置未下载: {}", log_file.display());

        // 验证至少下载了第三方库
        let lib_dir = game_dir.join("libraries");
        let lib_count = std::fs::read_dir(&lib_dir)
            .map(|d| d.count())
            .unwrap_or(0);
        assert!(lib_count > 0, "libraries 目录为空: {}", lib_dir.display());
    }
}
