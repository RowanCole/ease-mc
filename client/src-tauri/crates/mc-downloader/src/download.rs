// 下载编排层：从 COS 下载整包游戏压缩包，解压出 .minecraft 与 java，
// 对接后续的启动逻辑（mc-launcher 按 game/java + game/.minecraft 结构启动）。
use std::io::Read;
use std::path::Path;

use reqwest::Client;
use tauri::Emitter;
use tracing::{info, warn};

use mc_core::{config::{get_config, set_config}, paths::game_path};

use crate::jre::download_jre;
use crate::net;
use crate::progress::ProgressCtx;
use crate::source::{resolve_config_url, CosSource};

/// 游戏整包压缩包的配置项名称
const GAME_ZIP_KEY: &str = "gameZipPath";

/// 解压游戏整包 zip 到 dest_dir。
/// 若所有条目共享同一个一级目录（如打包的是 game 目录本身），自动去掉该层；
/// 否则按压缩包内原始结构解压。
fn extract_game_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("打开 {} 失败: {}", zip_path.display(), e))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|e| format!("解析压缩包失败: {}", e))?;

    // 判断所有条目是否共享同一个一级目录
    let mut common_root: Option<String> = None;
    for name in zip.file_names() {
        let first = name.split('/').next().unwrap_or("").to_string();
        if first.is_empty() {
            continue;
        }
        match &common_root {
            None => common_root = Some(first),
            Some(root) if root == &first => {}
            Some(_) => {
                common_root = None;
                break;
            }
        }
    }

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| format!("读取压缩包条目失败: {}", e))?;
        let rel = entry
            .enclosed_name()
            .ok_or_else(|| format!("压缩包内存在非法路径: {}", entry.name()))?;
        // 去掉公共一级目录
        let rel: std::path::PathBuf = match &common_root {
            Some(_) => rel.components().skip(1).collect(),
            None => rel.to_path_buf(),
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let out = dest_dir.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out)
                .map_err(|e| format!("创建目录 {} 失败: {}", out.display(), e))?;
        } else {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录 {} 失败: {}", parent.display(), e))?;
            }
            let mut f = std::fs::File::create(&out)
                .map_err(|e| format!("创建文件 {} 失败: {}", out.display(), e))?;
            std::io::copy(&mut entry, &mut f)
                .map_err(|e| format!("解压 {} 失败: {}", out.display(), e))?;
        }
    }
    Ok(())
}

/// 『一键安装』流程：从 COS 下载游戏整包 zip → 解压 → （缺 JRE 时）下载 JRE → 标记安装完成。
pub async fn download_game(app: tauri::AppHandle) -> Result<(), String> {
    info!("=== 开始游戏下载流程 ===");
    let client = Client::new();
    let cos = CosSource::from_config(Some(&app));
    let game_dir = game_path()?;

    // 1. 解析整包地址（gameZipPath 支持 cos://对象key 或普通 URL）
    let zip_value = get_config(Some(&app), GAME_ZIP_KEY)?;
    let zip_url = resolve_config_url(cos.as_ref(), &zip_value, GAME_ZIP_KEY)?;
    info!("游戏整包地址: {}", zip_url);

    // 2. 下载整包 zip（进度 5% → 80%）
    let _ = app.emit(
        "download-progress",
        serde_json::json!({ "percent": 5.0, "stage": "download" }),
    );
    let zip_path = game_dir.join(".game-download.zip");
    let progress = ProgressCtx::new(app.clone(), 5.0, 80.0, 0, "download");
    net::download_file(&client, &zip_url, &zip_path, Some(&progress)).await?;
    progress.finish();

    // 3. 解压到 game 目录（进度 80% → 90%）
    let _ = app.emit("extract-start", serde_json::json!({ "stage": "extract" }));
    info!("解压游戏整包到 {}", game_dir.display());
    extract_game_zip(&zip_path, &game_dir)?;
    let _ = app.emit(
        "download-progress",
        serde_json::json!({ "percent": 90.0, "stage": "extract" }),
    );
    let _ = std::fs::remove_file(&zip_path);

    // 4. 压缩包内已包含 java 目录时跳过 JRE 安装；否则下载 JRE（90% → 100%）
    let java_bin = game_dir
        .join("java")
        .join("bin")
        .join(if cfg!(windows) { "java.exe" } else { "java" });
    if java_bin.exists() {
        info!("整包已包含 JRE，跳过 JRE 下载");
        let _ = app.emit(
            "download-progress",
            serde_json::json!({ "percent": 100.0, "stage": "jre" }),
        );
    } else {
        if cos.is_none() {
            warn!("整包未包含 JRE 且未配置 COS，JRE 将按 winJrePath/macJrePath 配置下载");
        }
        let _ = app.emit("extract-start", serde_json::json!({ "stage": "jre" }));
        download_jre(&client, &game_dir, cos.as_ref(), Some(&app)).await?;
    }

    // 5. 标记安装完成，后续由启动逻辑直接使用 game 目录
    set_config(Some(&app), "gameIsInstalled", "true")?;
    info!("=== 游戏下载流程完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个带单层根目录 game/ 的 zip 验证解压与根目录剥离
    fn make_zip_with_root(path: &Path) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.add_directory("game/java/bin", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.start_file("game/java/bin/java.exe", zip::write::SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut zip, b"fake-java").unwrap();
        zip.start_file("game/.minecraft/versions/1.21.1/1.21.1.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut zip, b"{}").unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn extract_game_zip_strips_common_root() {
        let tmp = std::env::temp_dir().join("easemc_zip_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let zip_path = tmp.join("game.zip");
        make_zip_with_root(&zip_path);

        let dest = tmp.join("out");
        extract_game_zip(&zip_path, &dest).unwrap();

        assert!(dest.join("java/bin/java.exe").exists());
        assert!(dest.join(".minecraft/versions/1.21.1/1.21.1.json").exists());
        assert!(!dest.join("game").exists(), "不应残留 game 一级目录");
    }
}
