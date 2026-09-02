// 通用 HTTP 下载层：BMCLAPI 镜像回退 + 并发调度。
// 不感知具体业务（Minecraft / JRE），仅负责"把 URL 下到本地文件并上报进度"。
use std::path::{Path, PathBuf};

use futures_util::stream::{self, StreamExt};
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

use crate::progress::ProgressCtx;

/// 单个下载任务：(目标 URL, 落盘路径, 预期大小字节)
pub type DownloadTask = (String, PathBuf, u64);

/// 将 Mojang 官方地址映射为 BMCLAPI 国内镜像；非官方域返回原地址。
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
            "https://bmclapi2.bangbang93.com/",
            1,
        )
    } else {
        url.to_string()
    }
}

/// 下载单个文件；官方源先走镜像，失败自动回退官方源。
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

/// 并发下载一批文件，聚合所有失败信息；成功时收敛进度到区间终点。
pub async fn download_files_concurrent(
    client: &Client,
    tasks: Vec<DownloadTask>,
    max_concurrent: usize,
    progress: Option<&ProgressCtx>,
) -> Result<(), String> {
    let total = tasks.len();
    let total_bytes: u64 = tasks.iter().map(|(_, _, size)| *size).sum();
    info!(
        "开始并发下载 {} 个文件（共约 {} MB，并发上限 {}）",
        total,
        total_bytes / 1024 / 1024,
        max_concurrent
    );

    let ctx = progress.cloned();

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
        if let Some(p) = progress {
            p.finish();
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
