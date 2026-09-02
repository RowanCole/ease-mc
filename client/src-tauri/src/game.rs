use std::path::Path;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info};

use crate::manifest;
use crate::paths::game_path;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows flag: prevents the spawned process from creating a console window
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

static GAME: Mutex<Option<Child>> = Mutex::new(None);

/// 递归收集 libraries 目录下所有 jar 文件，转换为相对路径后写入 jars。
fn collect_jars(dir: &Path, base: &Path, jars: &mut Vec<String>) -> Result<(), String> {
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

#[tauri::command]
pub fn launch_game(app: AppHandle) -> Result<(), String> {
    let cwd = game_path()?;
    let minecraft_path = cwd.join(".minecraft");
    // Windows 下 JRE 可执行文件带 .exe 后缀，macOS/Linux 不带
    let java = cwd
        .join("java")
        .join("bin")
        .join(if cfg!(windows) { "java.exe" } else { "java" });
    debug!("cwd: {:?}", cwd);
    debug!("minecraft_path: {:?}", minecraft_path);
    debug!("java: {:?}", java.to_str().unwrap());

    if !minecraft_path.exists() {
        error!(".minecraft 目录不存在: {:?}", minecraft_path);
        return Err(format!(".minecraft directory not found at {:?}", minecraft_path));
    }

 
    let mut classpath: Vec<String> = Vec::new();
    let lib_dir = minecraft_path.join("libraries");
    if lib_dir.exists() {
        collect_jars(&lib_dir, &minecraft_path, &mut classpath)?;
    }

    // 版本/资源索引等参数统一来自 game.json（manifest），避免硬编码漂移
    let manifest_json = manifest::load_embedded()?;
    let version_id = manifest::version_id(&manifest_json)?;
    let asset_index_id = manifest::asset_index_id(&manifest_json)?;
    let client_jar = format!("versions/{version_id}/{version_id}.jar");

    classpath.push(client_jar);
    debug!("classpath 共 {} 个 jar", classpath.len());
  
    let cp = classpath.join(if cfg!(windows) { ";" } else { ":" });

    let mut game_cmd = Command::new(java.to_str().unwrap());

    #[cfg(target_os = "macos")]
    game_cmd.arg("-XstartOnFirstThread");

    // 启动参数：version/assetIndex 等来自 manifest，其余为固定 JVM/客户端参数
    let args: Vec<String> = vec![
        "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".into(),
        "-Djava.library.path=natives".into(),
        "-Djna.tmpdir=natives".into(),
        "-Dorg.lwjgl.system.SharedLibraryExtractPath=natives".into(),
        "-Dio.netty.native.workdir=natives".into(),
        "-Dminecraft.launcher.brand=manual".into(),
        format!("-Dminecraft.launcher.version={version_id}"),
        "-cp".into(),
        cp,
        "net.minecraft.client.main.Main".into(),
        "--username".into(),
        "Steve".into(),
        "--version".into(),
        version_id.clone(),
        "--gameDir".into(),
        ".".into(),
        "--assetsDir".into(),
        "assets".into(),
        "--assetIndex".into(),
        asset_index_id,
        "--uuid".into(),
        "00000000-0000-0000-0000-000000000000".into(),
        "--accessToken".into(),
        "0".into(),
        "--userType".into(),
        "mojang".into(),
        "--versionType".into(),
        "release".into(),
    ];
    game_cmd.args(args);
    game_cmd.current_dir(&minecraft_path);

    #[cfg(windows)]
    game_cmd.creation_flags(CREATE_NO_WINDOW);
    let game = game_cmd
        .spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    let pid = game.id();
    info!("游戏进程已启动 (PID: {}, version: {})", pid, version_id);

    *GAME.lock().map_err(|e| e.to_string())? = Some(game);

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let exited: bool = {
            let mut guard = match GAME.lock() {
                Ok(guard) => guard,
                Err(_) => continue,
            };
            match guard.as_mut() {
                Some(child) => match child.try_wait() {
                    Ok(Some(_)) => {
                        *guard = None;
                        true
                    }
                    Ok(None) => false,
                    Err(_) => {
                        *guard = None;
                        true
                    }
                },
                // The process was closed via close_game(); stop monitoring
                None => break,
            }
        };
        if exited {
            let _ = app.emit("game-exited", ());
            info!("Game process exited (PID: {})", pid);
            break;
        }
    });

    Ok(())
}

#[tauri::command]
pub fn close_game() -> Result<(), String> {
    let child = GAME.lock().map_err(|e| e.to_string())?.take();
    if let Some(child) = child {
        let pid = child.id();
        // Windows 用 taskkill 结束整个进程树；macOS/Linux 用 kill
        let output = if cfg!(windows) {
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .output()
        } else {
            Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output()
        }
        .map_err(|e| format!("Failed to kill process tree: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("kill error: {}", stderr);
        }
    }
    info!("Game process closed");
    Ok(())
}
