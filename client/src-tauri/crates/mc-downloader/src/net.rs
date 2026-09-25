// 通用 HTTP 下载层：只负责"把 URL 下到本地文件并上报进度"，不感知具体来源与业务。
use std::path::Path;

use futures_util::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::debug;

use crate::progress::ProgressCtx;

/// 下载单个文件到指定路径。
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

