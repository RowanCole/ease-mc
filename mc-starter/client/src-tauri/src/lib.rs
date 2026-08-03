// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod utils;

use reqwest::Client;
use utils::collect_jars;
use std::process::{Child, Command};
use std::sync::Mutex;
use tracing::{error, info};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;


static GAME: Mutex<Option<Child>> = Mutex::new(None);

#[tauri::command]
fn init() -> Result<(), String> {
    let config = get_config("gameIsInstalled");
    match config {
        Ok(_) => Ok(()),
        Err(e) => Ok(()),
    }
}

#[tauri::command]
fn launch_game() -> Result<(), String> {
    let cwd = std::env::current_dir().unwrap().join("game");
    let minecraft_path = cwd.join(".minecraft");
    let java = cwd.join("jdk-21.0.11").join("bin").join("java.exe");
    println!("cwd: {:?}", cwd);
    println!("minecraft_path: {:?}", minecraft_path);
    println!("java: {:?}",java.to_str().unwrap());

    if !minecraft_path.exists() {
        return Err(format!(".minecraft directory not found at {:?}", minecraft_path));
    }

    // Build classpath from libraries/
    let mut classpath: Vec<String> = Vec::new();
    let lib_dir = minecraft_path.join("libraries");
    if lib_dir.exists() {
        collect_jars(&lib_dir, &minecraft_path, &mut classpath)?;
    }
    classpath.push("versions/1.21.1/1.21.1.jar".to_string());
    let cp = classpath.join(";");
 

    let game = Command::new(java.to_str().unwrap())
        .args([
            "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump",
            "-Djava.library.path=natives",
            "-Djna.tmpdir=natives",
            "-Dorg.lwjgl.system.SharedLibraryExtractPath=natives",
            "-Dio.netty.native.workdir=natives",
            "-Dminecraft.launcher.brand=manual",
            "-Dminecraft.launcher.version=1.21.1",
            "-cp",
            &cp,
            "net.minecraft.client.main.Main",
            "--username",
            "Steve",
            "--version",
            "1.21.1",
            "--gameDir",
            ".",
            "--assetsDir",
            "assets",
            "--assetIndex",
            "17",
            "--uuid",
            "00000000-0000-0000-0000-000000000000",
            "--accessToken",
            "0",
            "--userType",
            "mojang",
            "--versionType",
            "release",
        ])
        .current_dir(&minecraft_path)
        .spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    println!("Game process started with PID: {}", game.id());

    *GAME.lock().map_err(|e| e.to_string())? = Some(game);
    Ok(())
}

#[tauri::command]
fn close_game() -> Result<(), String> {
    let child = GAME.lock().map_err(|e| e.to_string())?.take();
    if let Some(child) = child {
        let pid = child.id();
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output()
            .map_err(|e| format!("Failed to kill process tree: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("taskkill error: {}", stderr);
        }
    }
    info!("Game process closed");
    Ok(())
}


#[tauri::command]
fn get_config(key: &str) -> Result<String, String> {
    let content = std::fs::read_to_string("config.json")
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    json[key]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("config.json 缺少 {} 字段", key))
}

#[tauri::command]
fn set_config(key: &str, value: &str) -> Result<(), String> {
    let mut json: serde_json::Value = if std::path::Path::new("config.json").exists() {
        let content = std::fs::read_to_string("config.json")
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?
    } else {
        serde_json::json!({})
    };
    json[key] = serde_json::Value::String(value.to_string());
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化配置文件失败: {}", e))?;
    std::fs::write("config.json", content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn download_game() {
    let client = Client::new();

    let offset = tokio::fs::metadata("game.zip").await.map(|m| m.len() as u64).unwrap_or(0) as u64;
    let server_url = get_config("serverUrl").unwrap();
    let response = client
        .get(format!("{}/game.zip", server_url))
        .header("Range", format!("bytes={}-", offset))
        .send()
        .await
        .unwrap();
    
    let mut file = match response.status().as_u16() {
        200 => {
            tokio::fs::File::create("game.zip").await.unwrap()
        }
        206 => {
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
    }
    file.flush().await.unwrap();


    info!("Download completed");
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_game() {
        download_game().await;
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![launch_game, close_game,get_config,download_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
