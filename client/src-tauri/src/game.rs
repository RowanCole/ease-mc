use std::path::Path;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info};

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
    let cwd = crate::config::app_dir().join("game");
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

    // Build classpath from libraries/
    let mut classpath: Vec<String> = Vec::new();
    let lib_dir = minecraft_path.join("libraries");
    if lib_dir.exists() {
        collect_jars(&lib_dir, &minecraft_path, &mut classpath)?;
    }
    classpath.push("versions/1.21.1/1.21.1.jar".to_string());
    debug!("classpath 共 {} 个 jar", classpath.len());
    // Windows 用分号分隔 classpath，macOS/Linux 用冒号
    let cp = classpath.join(if cfg!(windows) { ";" } else { ":" });

    let mut game_cmd = Command::new(java.to_str().unwrap());
    // macOS 上 GLFW 要求 JVM 主线程必须是进程的第一个线程，否则启动即崩溃。
    // 该参数仅 macOS JVM 支持，Windows 传入会报 "Unrecognized option"，故用 cfg 门控。
    #[cfg(target_os = "macos")]
    game_cmd.arg("-XstartOnFirstThread");
    game_cmd.args([
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
    ]);
    game_cmd.current_dir(&minecraft_path);
    // Hide the console window when launching the game on Windows
    #[cfg(windows)]
    game_cmd.creation_flags(CREATE_NO_WINDOW);
    let game = game_cmd
        .spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    let pid = game.id();
    info!("游戏进程已启动 (PID: {})", pid);

    *GAME.lock().map_err(|e| e.to_string())? = Some(game);

    // Monitor the game process; notify the frontend when it exits on its own
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
