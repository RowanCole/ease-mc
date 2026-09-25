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

use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder =tauri::Builder::default();
        builder = builder.setup(|app| {
            let show = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("EaseMC")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;
            Ok(())
        });
        builder = builder.invoke_handler(tauri::generate_handler![
            launch_game,
            close_game,
            get_config,
            download_game,
            send_messages_to_mode
        ]);
        builder = builder.on_window_event(|window,evevt|{
            if let WindowEvent::CloseRequested { api, .. } = evevt {
                api.prevent_close();
                let _ = window.hide();
            }
        });
        builder.run(tauri::generate_context!()).expect("error while running tauri application");
}
