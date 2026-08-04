use crate::config::{get_config, set_config};
use futures_util::StreamExt;
use reqwest::Client;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use tracing::info;

#[tauri::command]
pub async fn download_game(app: tauri::AppHandle) -> Result<(), String> {
    let client = Client::new();

    let mut offset = tokio::fs::metadata("game.zip").await.map(|m| m.len() as u64).unwrap_or(0) as u64;
    let server_url = get_config("serverUrl").unwrap();
    let response = match client
        .get(format!("{}/game.zip", server_url))
        .header("Range", format!("bytes={}-", offset))
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            let _ = app.emit("server-connection-failed", e.to_string());
            return Err(format!("服务器连接失败: {}", e));
        }
    };

    let mut file_len: u64 = 0;

    let mut file = match response.status().as_u16() {
        200 => {
            file_len = response.headers().get("content-length").unwrap().to_str().unwrap().parse().unwrap();
            tokio::fs::File::create("game.zip").await.unwrap()
        }
        206 => {
            file_len = response.headers().get(reqwest::header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.rsplit('/').next())
            .map(|v| v.parse::<u64>().unwrap())
            .unwrap_or(0);

            file_len += offset;
            file_len = file_len as u64;
            tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("game.zip")
                .await
                .unwrap()
        }
        _ => {
            tokio::fs::File::create("game.zip").await.unwrap()
        }
    };

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        file.write_all(&chunk).await.unwrap();
        offset += chunk.len() as u64;
        app.emit("download-progress", serde_json::json!({
        "downloaded": offset,
        "total": file_len,
        "percent": offset as f64 / file_len as f64 * 100.0,
    })).map_err(|e| format!("发送事件失败: {}", e)).unwrap();
    }
    file.flush().await.unwrap();

    info!("Download completed");
    app.emit("extract-start", ())
        .map_err(|e| format!("发送事件失败: {}", e))
        .unwrap();
    info!("Starting unzip game");

    drop(file);
    let std_file = std::fs::File::open("game.zip").unwrap();
    tokio::task::spawn_blocking(move || {
        let mut zip = zip::ZipArchive::new(std_file).unwrap();
        zip.extract("./").unwrap();
    })
    .await
    .unwrap();
    info!("Game unzipped");
    set_config("gameIsInstalled", "true").unwrap();
    Ok(())
}
