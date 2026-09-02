// 下载编排层：把「清单解析（manifest）」「HTTP 下载（net）」「JRE 安装（jre）」
// 组装成一条『一键安装』命令。本身不再包含传输细节。
use std::path::Path;

use reqwest::Client;
use tauri::Emitter;
use tracing::info;

use crate::config::set_config;
use crate::jre::download_jre;
use crate::manifest;
use crate::net::{self, DownloadTask};
use crate::paths::game_path;
use crate::progress::ProgressCtx;

const MAX_CONCURRENT_DOWNLOADS: usize = 8;

/// 下载游戏文件到 minecraft_dir（进度区间 5% → 90%）。
pub async fn download_game_files(
    client: &Client,
    manifest_json: &serde_json::Value,
    minecraft_dir: &Path,
    app: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    let os_name = manifest::current_os_name();
    info!(
        "开始下载游戏文件到 {:?}（平台: {}）",
        minecraft_dir,
        os_name
    );

    // 先收集全部下载任务，再统一并发下载。元组第三个元素为文件预期大小（字节），用于字节级进度。
    let mut tasks: Vec<DownloadTask> = Vec::new();

    // 1. 第三方库
    tasks.extend(manifest::library_tasks(manifest_json, os_name, minecraft_dir)?);

    // 2. 客户端主 jar
    if let Some((url, dest, _size)) = manifest::client_jar_task(manifest_json, minecraft_dir)? {
        info!("下载客户端主 jar: {}", dest.display());
        tasks.push((url, dest, _size));
    }

    // 3. 资源索引：先单独下载索引 JSON，再解析出全部资源对象清单
    let index_id = manifest::asset_index_id(manifest_json)
        .unwrap_or_else(|_| "index".to_string());
    let index_dest = minecraft_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", index_id));
    if let Some(url) = manifest::asset_index_url(manifest_json) {
        info!("下载资源索引: {}", url);
        net::download_file(client, &url, &index_dest, None).await?;
    }

    // 4. 解析资源索引，为每个资源对象生成下载任务（占大头，约 800MB+）
    if index_dest.exists() {
        let content = std::fs::read_to_string(&index_dest)
            .map_err(|e| format!("读取资源索引失败: {}", e))?;
        let index: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("解析资源索引失败: {}", e))?;
        tasks.extend(manifest::asset_object_tasks(&index, minecraft_dir));
    }

    // 5. 日志配置文件
    if let Some(task) = manifest::logging_task(manifest_json, minecraft_dir) {
        tasks.push(task);
    }

    // 并发下载全部任务
    let total_bytes: u64 = tasks.iter().map(|(_, _, size)| *size).sum();
    let progress = app.map(|a| ProgressCtx::new(a.clone(), 5.0, 90.0, total_bytes));
    net::download_files_concurrent(client, tasks, MAX_CONCURRENT_DOWNLOADS, progress.as_ref())
        .await?;

    info!("游戏文件下载完成");
    Ok(())
}

/// 『一键安装』命令：下载游戏文件 → 下载并解压 JRE → 标记安装完成。
#[tauri::command]
pub async fn download_game(app: tauri::AppHandle) -> Result<(), String> {
    info!("=== 开始游戏下载流程 ===");
    let client = Client::new();

    // 1. 读取内置的 game.json 版本清单（唯一版本数据源）
    let manifest_json = manifest::load_embedded()?;
    let version_id = manifest::version_id(&manifest_json)?;
    info!("版本清单加载成功，版本: {}", version_id);

    // 2. 按清单下载游戏文件到 game/.minecraft
    let _ = app.emit("download-progress", serde_json::json!({ "percent": 5.0 }));
    let game_dir = game_path()?;
    let minecraft_dir = game_dir.join(".minecraft");
    download_game_files(&client, &manifest_json, &minecraft_dir, Some(&app)).await?;
    let _ = app.emit("download-progress", serde_json::json!({ "percent": 90.0 }));

    // 3. 下载并解压 JRE 到 game/java
    let _ = app.emit("extract-start", ());
    download_jre(&client, &game_dir, Some(&app)).await?;
    let _ = app.emit("download-progress", serde_json::json!({ "percent": 100.0 }));

    // 4. 标记安装完成
    set_config(app, "gameIsInstalled", "true")?;
    info!("=== 游戏下载流程完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn download_game_files_to_game_dir() {
        let client = Client::new();

        // game.json 位于 src/ 下，以 crate 根目录（client/src-tauri）定位
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("game.json");
        let content = std::fs::read_to_string(&manifest_path).expect("读取 game.json 失败");
        let manifest_json: serde_json::Value =
            serde_json::from_str(&content).expect("解析 game.json 失败");

        // 与 launch_game 使用的目录结构保持一致：./game/.minecraft
        let game_dir = Path::new("game").join(".minecraft");
        download_game_files(&client, &manifest_json, &game_dir, None)
            .await
            .expect("下载 game.json 中的文件失败");

        // 验证客户端主 jar
        let version_id = manifest_json["id"].as_str().expect("缺少 id 字段");
        let client_jar = game_dir
            .join("versions")
            .join(version_id)
            .join(format!("{}.jar", version_id));
        assert!(
            client_jar.exists(),
            "客户端 jar 未下载: {}",
            client_jar.display()
        );

        // 验证资源索引
        let index_id = manifest_json["assetIndex"]["id"]
            .as_str()
            .expect("缺少 assetIndex.id 字段");
        let asset_index = game_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", index_id));
        assert!(
            asset_index.exists(),
            "资源索引未下载: {}",
            asset_index.display()
        );

        // 验证日志配置
        let log_id = manifest_json["logging"]["client"]["file"]["id"]
            .as_str()
            .expect("缺少 logging.id 字段");
        let log_file = game_dir.join("assets").join("log_configs").join(log_id);
        assert!(
            log_file.exists(),
            "日志配置未下载: {}",
            log_file.display()
        );

        // 验证至少下载了第三方库
        let lib_dir = game_dir.join("libraries");
        let lib_count = std::fs::read_dir(&lib_dir)
            .map(|d| d.count())
            .unwrap_or(0);
        assert!(lib_count > 0, "libraries 目录为空: {}", lib_dir.display());
    }
}
