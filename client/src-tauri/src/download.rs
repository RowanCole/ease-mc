use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures_util::stream::{self, StreamExt};
use reqwest::Client;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

use crate::config::set_config;
use crate::jre::download_jre;

/// 并发下载上限。Mojang CDN 对单连接限速，8 并发可让多个库文件并行拉取。
const MAX_CONCURRENT_DOWNLOADS: usize = 8;

/// 将 Mojang 官方 URL 替换为国内镜像（BMCLAPI），显著提升国内下载速度。
/// 规则：域名替换为 bmclapi2.bangbang93.com，libraries 路径前缀为 /maven。
fn mirror_url(url: &str) -> String {
    if url.starts_with("https://libraries.minecraft.net/") {
        url.replacen(
            "https://libraries.minecraft.net/",
            "https://bmclapi2.bangbang93.com/maven/",
            1,
        )
    } else if url.starts_with("https://piston-data.mojang.com/") {
        url.replacen(
            "https://piston-data.mojang.com/",
            "https://bmclapi2.bangbang93.com/",
            1,
        )
    } else if url.starts_with("https://piston-meta.mojang.com/") {
        url.replacen(
            "https://piston-meta.mojang.com/",
            "https://bmclapi2.bangbang93.com/",
            1,
        )
    } else if url.starts_with("https://resources.download.minecraft.net/") {
        url.replacen(
            "https://resources.download.minecraft.net/",
            "https://bmclapi2.bangbang93.com/assets/",
            1,
        )
    } else {
        url.to_string()
    }
}

/// 字节级下载进度上下文：累计已下载字节数，按阈值（推进 >= 1%）节流上报。
///
/// `total_bytes` 优先使用版本清单预知的总大小（跨文件精确）；传 0 时
/// 退化为用单个响应的 Content-Length 计算（JRE 单文件下载场景）。
#[derive(Clone)]
pub(crate) struct ProgressCtx {
    app: tauri::AppHandle,
    done: Arc<AtomicU64>,
    last_percent: Arc<AtomicU64>,
    start: f64,
    span: f64,
    total_bytes: u64,
}

impl ProgressCtx {
    pub(crate) fn new(
        app: tauri::AppHandle,
        start: f64,
        end: f64,
        total_bytes: u64,
    ) -> Self {
        Self {
            app,
            done: Arc::new(AtomicU64::new(0)),
            last_percent: Arc::new(AtomicU64::new(0)),
            start,
            span: end - start,
            total_bytes,
        }
    }

    /// 累计 n 字节并节流上报进度（每次推进 >= 1% 才发一次事件）。
    fn add_bytes(&self, n: u64, response_len: u64) {
        let total = if self.total_bytes > 0 {
            self.total_bytes
        } else {
            response_len
        };
        if total == 0 {
            return;
        }
        let bytes = self.done.fetch_add(n, Ordering::SeqCst) + n;
        let percent = (self.start + self.span * (bytes as f64 / total as f64))
            .min(self.start + self.span);
        let key = (percent * 1000.0) as u64;
        if key >= self.last_percent.load(Ordering::SeqCst) + 10 {
            self.last_percent.store(key, Ordering::SeqCst);
            let _ = self.app.emit(
                "download-progress",
                serde_json::json!({ "percent": percent }),
            );
        }
    }
}

/// 下载单个文件到目标路径，自动创建缺失的父目录。
/// Mojang 官方地址会自动替换为镜像源以加速下载；
/// 若镜像源缺失（404），则自动回退官方地址，保证文件不遗漏。
pub async fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    progress: Option<&ProgressCtx>,
) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建目录 {} 失败: {}", parent.display(), e))?;
    }

    let mirrored = mirror_url(url);
    if mirrored != url {
        match download_from(client, &mirrored, dest, progress).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                debug!("镜像下载失败，回退官方源: {}", e);
            }
        }
    }

    download_from(client, url, dest, progress).await
}

/// 从指定 URL 下载单个文件到目标路径（不做镜像替换）。
async fn download_from(
    client: &Client,
    url: &str,
    dest: &Path,
    progress: Option<&ProgressCtx>,
) -> Result<(), String> {
    debug!("开始下载: {} -> {}", url, dest.display());

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求 {} 失败: {}", url, e))?;
    if !response.status().is_success() {
        return Err(format!("下载 {} 失败: HTTP {}", url, response.status()));
    }

    let content_len = response.content_length().unwrap_or(0);
    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("创建文件 {} 失败: {}", dest.display(), e))?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取下载流失败: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入 {} 失败: {}", dest.display(), e))?;
        if let Some(p) = progress {
            p.add_bytes(chunk.len() as u64, content_len);
        }
    }
    file.flush()
        .await
        .map_err(|e| format!("刷新 {} 失败: {}", dest.display(), e))?;
    debug!("下载完成: {}", dest.display());
    Ok(())
}

/// 并发下载多个文件，限制最大并发数，并按已下载字节数上报下载进度。
///
/// 进度范围由 `progress_range` 指定（如 (5.0, 60.0)），`tasks` 携带每个文件的
/// 预期大小（来自版本清单），据此在 5%→60% 区间平滑推进。
/// 所有任务都完成后统一返回；只要有一个失败即返回错误（已完成的文件保留）。
/// `reqwest::Client` 内部共享连接池，`Clone` 后可在多任务中安全使用。
async fn download_files_concurrent(
    client: &Client,
    tasks: Vec<(String, PathBuf, u64)>,
    max_concurrent: usize,
    app: Option<&tauri::AppHandle>,
    progress_range: (f64, f64),
) -> Result<(), String> {
    let total = tasks.len();
    let (start, end) = progress_range;
    let total_bytes: u64 = tasks.iter().map(|(_, _, size)| *size).sum();
    info!(
        "开始并发下载 {} 个文件（共约 {} MB，并发上限 {}）",
        total,
        total_bytes / 1024 / 1024,
        max_concurrent
    );

    let ctx = app.map(|a| ProgressCtx::new(a.clone(), start, end, total_bytes));

    let results: Vec<Result<(), String>> = stream::iter(tasks)
        .map(|(url, dest, _size)| {
            let client = client.clone();
            let ctx = ctx.clone();
            async move { download_file(&client, &url, &dest, ctx.as_ref()).await }
        })
        .buffer_unordered(max_concurrent)
        .collect()
        .await;

    // 聚合失败信息
    let errors: Vec<String> = results.into_iter().filter_map(|r| r.err()).collect();
    if errors.is_empty() {
        // 收敛到区间终点（防止清单 size 缺失导致进度滞后）
        if let Some(app) = app {
            let _ = app.emit("download-progress", serde_json::json!({ "percent": end }));
        }
        info!("全部 {} 个文件下载完成", total);
        Ok(())
    } else {
        Err(format!(
            "{} 个文件下载失败，第一个错误: {}",
            errors.len(),
            errors[0]
        ))
    }
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
    app: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    let os_name = current_os_name();
    info!(
        "开始下载游戏文件到 {:?}（平台: {}）",
        minecraft_dir,
        os_name
    );

    // 先收集全部下载任务，再统一并发下载。第三元组为文件预期大小（字节），用于字节级进度。
    let mut tasks: Vec<(String, PathBuf, u64)> = Vec::new();

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
        let size = artifact["size"].as_u64().unwrap_or(0);
        tasks.push((url.to_string(), dest, size));
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
        let size = manifest["downloads"]["client"]["size"].as_u64().unwrap_or(0);
        tasks.push((url.to_string(), dest, size));
    }

    // 3. 资源索引
    if let Some(url) = manifest["assetIndex"]["url"].as_str() {
        let index_id = manifest["assetIndex"]["id"].as_str().unwrap_or("index");
        let dest = minecraft_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", index_id));
        info!("下载资源索引: {}", dest.display());
        let size = manifest["assetIndex"]["size"].as_u64().unwrap_or(0);
        tasks.push((url.to_string(), dest, size));
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
        let size = manifest["logging"]["client"]["file"]["size"]
            .as_u64()
            .unwrap_or(0);
        tasks.push((url.to_string(), dest, size));
    }

    download_files_concurrent(client, tasks, MAX_CONCURRENT_DOWNLOADS, app, (5.0, 50.0))
        .await?;

    // 5. 资源文件（必须等资源索引下载完成后才能读取其 objects 清单）
    //    资源按 hash 存储于 assets/objects/{hash[0..2]}/{hash}，游戏按此路径加载，
    //    若缺失会导致背景图、纹理、语言等资源加载失败（如全景图马赛克）。
    let index_id = manifest["assetIndex"]["id"].as_str().unwrap_or("index");
    let index_path = minecraft_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", index_id));
    if index_path.exists() {
        let content = std::fs::read_to_string(&index_path)
            .map_err(|e| format!("读取资源索引失败: {}", e))?;
        let index: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("解析资源索引失败: {}", e))?;
        let mut asset_tasks: Vec<(String, PathBuf, u64)> = Vec::new();
        if let Some(objects) = index["objects"].as_object() {
            for obj in objects.values() {
                let hash = obj["hash"].as_str().unwrap_or("");
                if hash.len() < 2 {
                    continue;
                }
                let size = obj["size"].as_u64().unwrap_or(0);
                let dest = minecraft_dir
                    .join("assets")
                    .join("objects")
                    .join(&hash[0..2])
                    .join(hash);
                if dest.exists() {
                    continue; // 已存在则跳过，支持断点续传
                }
                let url = format!(
                    "https://resources.download.minecraft.net/{}/{}",
                    &hash[0..2],
                    hash
                );
                asset_tasks.push((url, dest, size));
            }
        }
        info!(
            "资源文件 {} 个待下载（索引: {}）",
            asset_tasks.len(),
            index_path.display()
        );
        download_files_concurrent(client, asset_tasks, MAX_CONCURRENT_DOWNLOADS, app, (50.0, 85.0))
            .await?;
    } else {
        info!("资源索引不存在，跳过资源文件下载: {}", index_path.display());
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

    // 2. 按清单下载游戏文件到 game/.minecraft（进度 5% → 85%，含资源文件）
    let game_dir = crate::config::app_dir().join("game");
    let minecraft_dir = game_dir.join(".minecraft");
    let _ = app.emit("download-progress", serde_json::json!({ "percent": 5.0 }));
    download_game_files(&client, &manifest, &minecraft_dir, Some(&app)).await?;
    let _ = app.emit("download-progress", serde_json::json!({ "percent": 85.0 }));

    // 3. 下载并解压 JRE 到 game/java（进度 85% → 97%）
    let _ = app.emit("extract-start", ());
    download_jre(&client, &game_dir, Some(&app), (85.0, 97.0)).await?;
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
        download_game_files(&client, &manifest, &game_dir, None)
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
