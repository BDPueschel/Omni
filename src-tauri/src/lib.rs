pub mod config;
pub mod providers;
pub mod search;

use config::OmniConfig;
use providers::apps::AppProvider;
use search::AppState;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

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

            // Global hotkey: Alt+Space to toggle window
            use tauri_plugin_global_shortcut::{
                Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
            };

            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            let win_clone = window.clone();
            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let w = win_clone.clone();
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                        let _ = w.emit("clear-query", ());
                    } else {
                        let _ = w.center();
                        let _ = w.show();
                        let _ = w.set_focus();
                        let _ = w.emit("window-shown", ());
                    }
                }
            })?;

            // Hide on blur (click outside)
            let win_blur = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = win_blur.hide();
                    let _ = win_blur.emit("clear-query", ());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search::search,
            search::refresh_apps,
            search::get_config,
            search::save_config,
            search::execute_action,
            search::hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
