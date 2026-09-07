// Tauri 应用壳：仅负责窗口入口与命令转发，业务逻辑实现在各领域 crate 中。
// 命令名与前端 invoke() 的约定保持不变。

#[tauri::command]
fn launch_game(app: tauri::AppHandle) -> Result<(), String> {
    mc_launcher::game::launch_game(app)
}

#[tauri::command]
fn close_game() -> Result<(), String> {
    mc_launcher::game::close_game()
}

#[tauri::command]
fn get_config(app: tauri::AppHandle, key: &str) -> Result<String, String> {
    mc_core::config::get_config(Some(&app), key)
}

#[tauri::command]
async fn download_game(app: tauri::AppHandle) -> Result<(), String> {
    mc_downloader::download::download_game(app).await
}

#[tauri::command]
async fn send_messages_to_mode(app: tauri::AppHandle, message: String) -> Result<String, String> {
    mc_assistant::chat::send_messages_to_mode(app, message).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            launch_game,
            close_game,
            get_config,
            download_game,
            send_messages_to_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
