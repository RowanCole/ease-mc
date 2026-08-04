// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod chat;
mod config;
mod download;
mod game;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            game::launch_game,
            game::close_game,
            config::get_config,
            download::download_game,
            chat::send_messages_to_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
