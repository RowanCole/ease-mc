use crate::utils::collect_jars;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{error, info};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Windows flag: prevents the spawned process from creating a console window
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

static GAME: Mutex<Option<Child>> = Mutex::new(None);

#[tauri::command]
pub fn launch_game(app: AppHandle) -> Result<(), String> {
    let cwd = std::env::current_dir().unwrap().join("game");
    let minecraft_path = cwd.join(".minecraft");
    let java = cwd.join("java").join("bin").join("java.exe");
    println!("cwd: {:?}", cwd);
    println!("minecraft_path: {:?}", minecraft_path);
    println!("java: {:?}", java.to_str().unwrap());

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

    let mut game_cmd = Command::new(java.to_str().unwrap());
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
    println!("Game process started with PID: {}", pid);

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
