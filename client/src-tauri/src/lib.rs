// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod chat;
mod config;
mod download;
mod game;
mod jre;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
