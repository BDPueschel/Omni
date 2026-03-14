pub mod config;
pub mod providers;
pub mod search;

use config::OmniConfig;
use providers::apps::AppProvider;
use search::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = OmniConfig::load();
    let apps = AppProvider::scan_start_menu();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            apps: Mutex::new(apps),
            config: Mutex::new(config),
        })
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            #[cfg(target_os = "windows")]
            {
                use window_vibrancy::apply_acrylic;
                let _ = apply_acrylic(&window, Some((10, 10, 15, 200)));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search::search,
            search::refresh_apps,
            search::get_config,
            search::save_config,
            search::execute_action,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
