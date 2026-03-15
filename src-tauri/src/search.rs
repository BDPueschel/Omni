use crate::config::OmniConfig;
use crate::providers::apps::{AppEntry, AppProvider};
use crate::providers::currency::CurrencyProvider;
use crate::providers::everything::EverythingProvider;
use crate::providers::math::MathProvider;
use crate::providers::process::ProcessProvider;
use crate::providers::system::SystemProvider;
use crate::providers::units::UnitProvider;
use crate::providers::url::UrlProvider;
use crate::providers::web_search::WebSearchProvider;
use crate::providers::SearchResult;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub apps: Mutex<Vec<AppEntry>>,
    pub config: Mutex<OmniConfig>,
}

/// Core search logic, extracted for testability.
pub fn search_query(
    query: &str,
    apps: &[AppEntry],
    max: usize,
    search_engine: &str,
) -> Vec<SearchResult> {
    let query = query.trim();
    if query.is_empty() {
        return vec![];
    }

    let mut all_results: Vec<SearchResult> = Vec::new();

    // Unit conversion (most specific — "5km in miles")
    let units = UnitProvider::evaluate(query);
    let has_units = !units.is_empty();
    all_results.extend(units);

    // Currency conversion ("100 usd to eur")
    let currency = CurrencyProvider::evaluate(query);
    let has_currency = !currency.is_empty();
    all_results.extend(currency);

    // Math (skip if we already matched a unit/currency conversion)
    let math = if has_units || has_currency {
        vec![]
    } else {
        MathProvider::evaluate(query)
    };
    let has_math = !math.is_empty();
    all_results.extend(math);

    // URL detection
    let urls = UrlProvider::evaluate(query);
    let has_url = !urls.is_empty();
    all_results.extend(urls);

    // Apps — try Everything first (finds more apps), fall back to local scan
    let app_results = EverythingProvider::search_apps(query, max);
    if app_results.is_empty() {
        all_results.extend(AppProvider::search(apps, query, max));
    } else {
        all_results.extend(app_results);
    }

    // System commands
    let system_results = SystemProvider::evaluate(query);
    all_results.extend(system_results.into_iter().take(max));

    // Process search (only when query starts with "kill ")
    if query.to_lowercase().starts_with("kill ") {
        let process_results = ProcessProvider::evaluate(query);
        all_results.extend(process_results.into_iter().take(max));
    }

    // Everything file search (files only, no directories)
    all_results.extend(EverythingProvider::search_files(query, max));

    // Everything directory search
    all_results.extend(EverythingProvider::search_dirs(query, max));

    // Web search fallback (suppress if we have a precise match)
    if !has_math && !has_url && !has_units && !has_currency {
        all_results.extend(WebSearchProvider::evaluate(query, search_engine));
    }

    all_results
}

#[tauri::command]
pub fn search(query: &str, state: State<AppState>) -> Vec<SearchResult> {
    let config = state.config.lock().unwrap().clone();
    let apps = state.apps.lock().unwrap().clone();
    search_query(query, &apps, config.max_results_per_category, &config.search_engine)
}

#[tauri::command]
pub fn refresh_apps(state: State<AppState>) {
    let new_apps = AppProvider::scan_start_menu();
    *state.apps.lock().unwrap() = new_apps;
}

#[tauri::command]
pub fn expand_category(query: &str, category: &str, state: State<AppState>) -> Vec<SearchResult> {
    let query = query.trim();
    if query.is_empty() {
        return vec![];
    }
    let config = state.config.lock().unwrap().clone();
    let apps = state.apps.lock().unwrap().clone();
    let max = 50; // expanded limit

    match category {
        "Apps" => {
            let results = EverythingProvider::search_apps(query, max);
            if results.is_empty() {
                AppProvider::search(&apps, query, max)
            } else {
                results
            }
        }
        "Files" => EverythingProvider::search_files(query, max),
        "Directories" => EverythingProvider::search_dirs(query, max),
        "System" => SystemProvider::evaluate(query),
        "Processes" => ProcessProvider::evaluate(query),
        "Web" => WebSearchProvider::evaluate(query, &config.search_engine),
        _ => vec![],
    }
}

/// Context menu actions for power users.
#[tauri::command]
pub fn open_containing_folder(path: &str) -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .args(["/select,", path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_in_terminal(path: &str) -> Result<(), String> {
    let dir = if std::path::Path::new(path).is_dir() {
        path.to_string()
    } else {
        std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("C:\\"))
            .to_string_lossy()
            .to_string()
    };
    std::process::Command::new("wt.exe")
        .args(["-d", &dir])
        .spawn()
        .or_else(|_| {
            // Fallback to powershell if Windows Terminal isn't installed
            std::process::Command::new("powershell.exe")
                .args(["-NoExit", "-Command", &format!("Set-Location '{}'", dir)])
                .spawn()
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_in_vscode(path: &str) -> Result<(), String> {
    let p = std::path::Path::new(path);
    if p.is_dir() {
        // Open folder in VS Code
        std::process::Command::new("cmd")
            .args(["/C", "code", path])
            .spawn()
            .map_err(|e| e.to_string())?;
    } else {
        // Open file in VS Code (also opens its parent folder as workspace)
        let dir = p.parent().unwrap_or(std::path::Path::new("C:\\"));
        std::process::Command::new("cmd")
            .args(["/C", "code", &dir.to_string_lossy(), path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn run_as_admin(path: &str) -> Result<(), String> {
    std::process::Command::new("powershell.exe")
        .args(["-Command", &format!("Start-Process '{}' -Verb RunAs", path)])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> OmniConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_config(config: OmniConfig, state: State<AppState>) -> Result<(), String> {
    config.save()?;
    *state.config.lock().unwrap() = config;
    Ok(())
}

#[tauri::command]
pub fn execute_action(action: crate::providers::ResultAction) -> Result<(), String> {
    match action {
        crate::providers::ResultAction::Copy { .. } => Ok(()),
        crate::providers::ResultAction::OpenFile { path } => {
            open::that(&path).map_err(|e| e.to_string())
        }
        crate::providers::ResultAction::OpenUrl { url } => {
            open::that(&url).map_err(|e| e.to_string())
        }
        crate::providers::ResultAction::LaunchApp { path } => {
            open::that(&path).map_err(|e| e.to_string())
        }
        crate::providers::ResultAction::WebSearch { url } => {
            open::that(&url).map_err(|e| e.to_string())
        }
        crate::providers::ResultAction::SystemCommand { command } => {
            execute_system_command(&command)
        }
        crate::providers::ResultAction::KillProcess { pid, name: _ } => {
            std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output()
                .map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}

#[tauri::command]
pub fn kill_process(pid: u32, _name: String) -> Result<(), String> {
    std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn hide_window(window: tauri::WebviewWindow) {
    use tauri::Emitter;
    let _ = window.hide();
    let _ = window.emit("clear-query", ());
}

fn execute_system_command(command: &str) -> Result<(), String> {
    match command {
        "lock" => {
            std::process::Command::new("rundll32.exe")
                .args(["user32.dll,LockWorkStation"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        "sleep" => {
            std::process::Command::new("rundll32.exe")
                .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        "shutdown" => {
            std::process::Command::new("shutdown")
                .args(["/s", "/t", "0"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        "restart" => {
            std::process::Command::new("shutdown")
                .args(["/r", "/t", "0"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        "sign_out" => {
            std::process::Command::new("shutdown")
                .args(["/l"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        "empty_recycle_bin" => {
            std::process::Command::new("powershell")
                .args(["-Command", "Clear-RecycleBin -Force -ErrorAction SilentlyContinue"])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("Unknown system command: {}", command)),
    }
    Ok(())
}

/// Dry-run version for testing — validates the command name without executing.
pub fn execute_system_command_dry(command: &str) -> Result<(), String> {
    match command {
        "lock" | "sleep" | "shutdown" | "restart" | "sign_out" | "empty_recycle_bin" => Ok(()),
        _ => Err(format!("Unknown system command: {}", command)),
    }
}
