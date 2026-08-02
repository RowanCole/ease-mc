// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod utils;

use utils::collect_jars;
use std::process::{Child, Command};
use std::sync::Mutex;
use crate::utils::get_config;
use tracing::{error, info};

static GAME: Mutex<Option<Child>> = Mutex::new(None);

#[tauri::command]
fn init() -> Result<(), String> {
    let config = get_config();
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

// fn download_game() -> Result<(), String> {
//     reqwest::get("https://www.minecraft.net/download")
// }


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![launch_game, close_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
