use std::path::Path;

use reqwest::Client;
use tracing::{debug, info};

use crate::download::download_file;

/// 解压压缩包到目标目录，支持 .zip 和 .tar.gz。
fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let ext = archive_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    debug!("解压 {} -> {}", archive_path.display(), dest_dir.display());
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
    info!("开始下载 JRE（配置项: {}）", key);

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

    info!("JRE 安装完成: {}", java_dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
