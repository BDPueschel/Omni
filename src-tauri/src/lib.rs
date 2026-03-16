pub mod autostart;
pub mod clipboard;
pub mod config;
pub mod icons;
pub mod preview;
pub mod providers;
pub mod recolor;
pub mod search;
pub mod usage;

use config::OmniConfig;
use providers::apps::AppProvider;
use search::AppState;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[tauri::command]
fn update_tray_icon(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let config = state.config.lock().unwrap().clone();

    let default_icon = app.default_window_icon().unwrap().clone();
    let icon = if config.use_system_accent {
        let accent = search::get_system_accent();
        recolor::recolored_tray_icon(&default_icon, accent)
    } else {
        default_icon
    };

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

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
            // Track whether the Press event was a "show" so Released doesn't also select
            let was_show = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                let w = win_clone.clone();
                if event.state == ShortcutState::Pressed {
                    if w.is_visible().unwrap_or(false) {
                        // Already visible — Released will handle select-all
                        was_show.store(false, std::sync::atomic::Ordering::SeqCst);
                    } else {
                        // Show window on press
                        was_show.store(true, std::sync::atomic::Ordering::SeqCst);
                        if let Ok(Some(monitor)) = w.current_monitor() {
                            let screen = monitor.size();
                            let scale = monitor.scale_factor();
                            let win_width = 600.0 * scale;
                            let x = ((screen.width as f64 - win_width) / 2.0) as i32;
                            let y = (screen.height as f64 * 0.15) as i32;
                            let _ = w.set_position(tauri::Position::Physical(
                                tauri::PhysicalPosition::new(x, y),
                            ));
                        }
                        let _ = w.show();
                        let _ = w.set_focus();
                        let _ = w.emit("window-shown", ());
                    }
                } else if event.state == ShortcutState::Released {
                    // Only select-all if this wasn't a fresh show
                    if !was_show.load(std::sync::atomic::Ordering::SeqCst)
                        && w.is_visible().unwrap_or(false)
                    {
                        let _ = w.set_focus();
                        let _ = w.emit("select-query", ());
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

            // Determine tray icon — recolor if system accent is enabled
            let default_icon = app.default_window_icon().unwrap().clone();
            let use_accent = app.state::<AppState>().config.lock().unwrap().use_system_accent;
            let tray_icon = if use_accent {
                let accent = search::get_system_accent();
                recolor::recolored_tray_icon(&default_icon, accent)
            } else {
                default_icon
            };

            let win_tray = window.clone();
            TrayIconBuilder::with_id("main")
                .tooltip("Omni — Alt+Space")
                .icon(tray_icon)
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        let _ = win_tray.show();
                        let _ = win_tray.set_focus();
                    }
                    "settings" => {
                        let _ = win_tray.show();
                        let _ = win_tray.set_focus();
                        let _ = win_tray.emit("open-settings", ());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Start clipboard monitor
            crate::clipboard::start_monitor();

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
            search::search_table,
            search::expand_category,
            search::refresh_apps,
            search::open_containing_folder,
            search::open_in_terminal,
            search::open_in_vscode,
            search::open_with,
            search::delete_file,
            search::copy_file_to,
            search::move_file_to,
            search::run_as_admin,
            search::get_config,
            search::save_config,
            search::execute_action,
            search::kill_process,
            search::hide_window,
            search::record_selection,
            search::get_frequent_items,
            search::clear_usage_data,
            search::complete_path,
            search::batch_open,
            search::batch_copy_to,
            search::batch_move_to,
            search::batch_delete,
            search::get_system_accent,
            icons::get_icon,
            preview::preview_file,
            clipboard::get_clipboard_history,
            clipboard::delete_clipboard_entry,
            clipboard::pin_clipboard_entry,
            clipboard::clear_clipboard_history,
            update_tray_icon,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
