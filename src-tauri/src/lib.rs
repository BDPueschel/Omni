pub mod autostart;
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
                        // Position at top-center of screen so results expand downward
                        if let Ok(Some(monitor)) = w.current_monitor() {
                            let screen = monitor.size();
                            let scale = monitor.scale_factor();
                            let win_width = 600.0 * scale;
                            let x = ((screen.width as f64 - win_width) / 2.0) as i32;
                            let y = (screen.height as f64 * 0.15) as i32; // 15% from top
                            let _ = w.set_position(tauri::Position::Physical(
                                tauri::PhysicalPosition::new(x, y),
                            ));
                        }
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

            // System tray
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let show_item = MenuItem::with_id(app, "show", "Show Omni", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_item, &settings_item, &quit_item])?;

            let win_tray = window.clone();
            TrayIconBuilder::new()
                .tooltip("Omni — Alt+Space")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        let _ = win_tray.show();
                        let _ = win_tray.set_focus();
                    }
                    "settings" => {
                        if let Some(settings_win) = app.get_webview_window("settings") {
                            let _ = settings_win.show();
                            let _ = settings_win.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Auto-start registration
            let config = app.state::<AppState>().config.lock().unwrap().clone();
            if config.start_with_windows && !autostart::is_autostart_enabled() {
                if let Ok(exe) = std::env::current_exe() {
                    let _ = autostart::enable_autostart(&exe.to_string_lossy());
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search::search,
            search::expand_category,
            search::refresh_apps,
            search::open_containing_folder,
            search::open_in_terminal,
            search::open_in_vscode,
            search::run_as_admin,
            search::get_config,
            search::save_config,
            search::execute_action,
            search::hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
