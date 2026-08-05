use std::path::Path;

use futures_util::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;

pub fn collect_jars(dir: &Path, base: &Path, jars: &mut Vec<String>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("read_dir failed: {}", e))? {
        let entry = entry.map_err(|e| format!("entry failed: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_jars(&path, base, jars)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("jar") {
            if let Ok(rel) = path.strip_prefix(base) {
                jars.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(())
}

/// 下载单个文件到目标路径，自动创建缺失的父目录。
pub async fn download_file(client: &Client, url: &str, dest: &Path) -> Result<(), String> {
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
        download_file(client, url, &dest).await?;
    }

    // 3. 资源索引
    if let Some(url) = manifest["assetIndex"]["url"].as_str() {
        let index_id = manifest["assetIndex"]["id"].as_str().unwrap_or("index");
        let dest = minecraft_dir
            .join("assets")
            .join("indexes")
            .join(format!("{}.json", index_id));
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
        download_file(client, url, &dest).await?;
    }

    Ok(())
}

/// 解压压缩包到目标目录，支持 .zip 和 .tar.gz。
fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let ext = archive_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    match ext {
        "zip" => {
            let file = std::fs::File::open(archive_path)
                .map_err(|e| format!("打开 {} 失败: {}", archive_path.display(), e))?;
            let mut zip = zip::ZipArchive::new(file)
                .map_err(|e| format!("解析压缩包失败: {}", e))?;
            zip.extract(dest_dir)
                .map_err(|e| format!("解压到 {} 失败: {}", dest_dir.display(), e))?;
        }
        "gz" => {
            let file = std::fs::File::open(archive_path)
                .map_err(|e| format!("打开 {} 失败: {}", archive_path.display(), e))?;
            let gz = flate2::read::GzDecoder::new(file);
            let mut tar = tar::Archive::new(gz);
            tar.unpack(dest_dir)
                .map_err(|e| format!("解压到 {} 失败: {}", dest_dir.display(), e))?;
        }
        other => return Err(format!("不支持的压缩包格式: {}", other)),
    }
    Ok(())
}

/// 根据当前平台从 config.json 读取 JRE 下载地址（macJrePath / winJrePath），
/// 下载、解压并重命名为 java 放到 game 目录下。
///
/// 目标结构：
/// - Windows: game/java/bin/java.exe
/// - macOS:   game/java/bin/java（自动去掉解压后的 jdk-*/Contents/Home 层级）
pub async fn download_jre(client: &Client, game_dir: &Path) -> Result<(), String> {
    let key = if cfg!(windows) { "winJrePath" } else { "macJrePath" };
    let url = crate::config::get_config(key)?;

    // 下载到临时文件（Windows 为 zip，macOS 为 tar.gz）
    let archive_name = if cfg!(windows) { ".jre-download.zip" } else { ".jre-download.tar.gz" };
    let archive_path = game_dir.join(archive_name);
    download_file(client, &url, &archive_path).await?;

    // 解压到临时目录，避免与 game 目录已有内容冲突
    let tmp_dir = game_dir.join(".jre_tmp");
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)
            .map_err(|e| format!("清理临时目录失败: {}", e))?;
    }
    extract_archive(&archive_path, &tmp_dir)?;

    // 找到解压后的根目录
    let root = std::fs::read_dir(&tmp_dir)
        .map_err(|e| format!("读取 {} 失败: {}", tmp_dir.display(), e))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.is_dir())
        .ok_or_else(|| "解压后未找到 JRE 目录".to_string())?;

    // macOS JRE 解压后为 jdk-*/Contents/Home，取其作为 java 根，保持跨平台目录一致
    let java_root = if root.join("Contents").join("Home").is_dir() {
        root.join("Contents").join("Home")
    } else {
        root
    };

    // 重命名为 java
    let java_dir = game_dir.join("java");
    if java_dir.exists() {
        std::fs::remove_dir_all(&java_dir)
            .map_err(|e| format!("删除旧 java 目录失败: {}", e))?;
    }
    std::fs::rename(&java_root, &java_dir)
        .map_err(|e| format!("重命名 {} -> {} 失败: {}", java_root.display(), java_dir.display(), e))?;

    // 清理临时文件
    let _ = std::fs::remove_dir_all(&tmp_dir);
    let _ = std::fs::remove_file(&archive_path);

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

    /// 下载 JRE 到 game/java 目录，并验证 java 可执行文件存在。
    /// 运行：cargo test -- --nocapture download_jre_to_game_dir
    #[tokio::test]
    async fn download_jre_to_game_dir() {
        let client = Client::new();
        let game_dir = Path::new("game");
        download_jre(&client, game_dir)
            .await
            .expect("下载 JRE 失败");

        let java_bin = if cfg!(windows) {
            game_dir.join("java").join("bin").join("java.exe")
        } else {
            game_dir.join("java").join("bin").join("java")
        };
        assert!(
            java_bin.exists(),
            "java 可执行文件不存在: {}",
            java_bin.display()
        );
    }
}
