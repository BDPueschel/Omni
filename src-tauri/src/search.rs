use crate::config::OmniConfig;
use crate::providers::apps::{AppEntry, AppProvider};
use crate::providers::color::ColorProvider;
use crate::providers::currency::CurrencyProvider;
use crate::providers::everything::EverythingProvider;
use crate::providers::math::MathProvider;
use crate::providers::process::ProcessProvider;
use crate::providers::system::SystemProvider;
use crate::providers::units::UnitProvider;
use crate::providers::url::UrlProvider;
use crate::providers::web_search::WebSearchProvider;
use crate::providers::{ResultAction, SearchResult};
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

    // Clipboard history (triggered by "clip" or "cb" prefix)
    let lower = query.to_lowercase();
    if lower == "clip" || lower == "cb" || lower.starts_with("clip ") || lower.starts_with("cb ") {
        let clip_query = if lower.starts_with("clip ") {
            &query[5..]
        } else if lower.starts_with("cb ") {
            &query[3..]
        } else {
            ""
        };
        let entries =
            crate::clipboard::get_clipboard_history_internal(clip_query, max as u32);
        for entry in entries {
            let pin_prefix = if entry.pinned { "\u{1f4cc} " } else { "" };
            all_results.push(SearchResult {
                category: "Clipboard".to_string(),
                title: entry.preview.clone(),
                subtitle: format!("{}{}", pin_prefix, entry.timestamp),
                action: ResultAction::Copy {
                    text: entry.content,
                },
                icon: "clipboard".to_string(),
                size: None,
                date_modified: None,
            });
        }
        return all_results;
    }

    // Color detection (#hex, rgb(), hsl())
    let color = ColorProvider::evaluate(query);
    let has_color = !color.is_empty();
    all_results.extend(color);

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

    // Everything file + directory search (single HTTP request, es.exe fallback)
    let (files, dirs) = EverythingProvider::search_all(query, max);
    all_results.extend(files);
    all_results.extend(dirs);

    // Web search fallback (suppress if we have a precise match)
    if !has_math && !has_url && !has_units && !has_currency && !has_color {
        all_results.extend(WebSearchProvider::evaluate(query, search_engine));
    }

    // Usage-based boosting — move frequently selected results to top of their category
    let usage = crate::usage::get_usage(query);
    if !usage.is_empty() {
        let usage_paths: std::collections::HashSet<String> =
            usage.iter().map(|(path, _, _, _)| path.clone()).collect();

        // Partition results: boosted first, then the rest, within each category
        all_results.sort_by(|a, b| {
            // Same category? Sort boosted items first
            if a.category == b.category {
                let a_boosted = usage_paths.contains(&a.subtitle);
                let b_boosted = usage_paths.contains(&b.subtitle);
                b_boosted.cmp(&a_boosted)
            } else {
                std::cmp::Ordering::Equal // preserve category order
            }
        });
    }

    all_results
}

/// Table panel search — files and directories only, with metadata, sortable.
pub fn search_table_query(
    query: &str,
    max: usize,
    sort_by: &str,
    ascending: bool,
) -> Vec<SearchResult> {
    let query = query.trim();
    if query.is_empty() {
        return vec![];
    }

    // Map frontend sort names to Everything HTTP API sort values.
    let sort = match sort_by {
        "name" => "name",
        "path" => "path",
        "size" => "size",
        "date_modified" => "date_modified",
        _ => "date_modified",
    };

    let http_query = EverythingProvider::build_http_query(query);
    match EverythingProvider::query_http_public(&http_query, max, sort, ascending) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("search_table HTTP error: {}", e);
            // Fallback: use regular search, filter to files/dirs
            let (files, dirs) = EverythingProvider::search_all(query, max / 2);
            let mut combined = files;
            combined.extend(dirs);
            combined
        }
    }
}

#[tauri::command]
pub fn search_table(
    query: &str,
    sort_by: &str,
    ascending: bool,
) -> Vec<SearchResult> {
    search_table_query(query, 100, sort_by, ascending)
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
        "Clipboard" => {
            let clip_query = if query.to_lowercase().starts_with("clip ") {
                &query[5..]
            } else if query.to_lowercase().starts_with("cb ") {
                &query[3..]
            } else {
                ""
            };
            crate::clipboard::get_clipboard_history_internal(clip_query, 50)
                .into_iter()
                .map(|entry| {
                    let pin_prefix = if entry.pinned { "\u{1f4cc} " } else { "" };
                    SearchResult {
                        category: "Clipboard".to_string(),
                        title: entry.preview.clone(),
                        subtitle: format!("{}{}", pin_prefix, entry.timestamp),
                        action: ResultAction::Copy { text: entry.content },
                        icon: "clipboard".to_string(),
                        size: None,
                        date_modified: None,
                    }
                })
                .collect()
        }
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
        // Use code --open-url with vscode:// URI to respect workbench.editorAssociations
        // (passing a bare file path to `code` forces text editor mode and pollutes the editor override cache)
        let uri = format!("vscode://file/{}", path.replace('\\', "/"));
        std::process::Command::new("cmd")
            .args(["/C", "code", "--open-url", &uri])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_with(path: &str) -> Result<(), String> {
    std::process::Command::new("rundll32.exe")
        .args(["shell32.dll,OpenAs_RunDLL", path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_file(path: &str) -> Result<(), String> {
    let ps_cmd = format!(
        "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile('{}', 'OnlyErrorDialogs', 'SendToRecycleBin')",
        path.replace("'", "''")
    );
    std::process::Command::new("powershell")
        .args(["-Command", &ps_cmd])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn copy_file_to(path: &str) -> Result<(), String> {
    let ps_cmd = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = 'Copy to...'; if ($f.ShowDialog() -eq 'OK') {{ Copy-Item '{}' -Destination $f.SelectedPath -Force }}"#,
        path.replace("'", "''")
    );
    std::process::Command::new("powershell")
        .args(["-Command", &ps_cmd])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn move_file_to(path: &str) -> Result<(), String> {
    let ps_cmd = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = 'Move to...'; if ($f.ShowDialog() -eq 'OK') {{ Move-Item '{}' -Destination $f.SelectedPath -Force }}"#,
        path.replace("'", "''")
    );
    std::process::Command::new("powershell")
        .args(["-Command", &ps_cmd])
        .spawn()
        .map_err(|e| e.to_string())?;
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

#[tauri::command]
pub fn record_selection(query: String, result_path: String, category: String, title: String) {
    crate::usage::record_usage(&query, &result_path, &category, &title);
}

#[tauri::command]
pub fn get_frequent_items() -> Vec<SearchResult> {
    let frequent = crate::usage::get_frequent(5);
    frequent
        .into_iter()
        .map(|(path, category, title, count)| {
            let action = if category == "Apps" {
                ResultAction::LaunchApp { path: path.clone() }
            } else {
                ResultAction::OpenFile { path: path.clone() }
            };
            SearchResult {
                category: "Frequent".to_string(),
                title,
                subtitle: format!("{} — used {} times", path, count),
                action,
                icon: if category == "Apps" {
                    "app".to_string()
                } else {
                    "file".to_string()
                },
                size: None,
                date_modified: None,
            }
        })
        .collect()
}

#[tauri::command]
pub fn clear_usage_data() {
    crate::usage::clear_usage();
}

#[tauri::command]
pub fn complete_path(partial: String) -> Vec<String> {
    use crate::providers::everything::EverythingProvider;

    // Normalize forward slashes to backslashes
    let partial = partial.replace("/", "\\");

    // Search for matching directories
    EverythingProvider::complete_path(&partial, 5)
}

#[tauri::command]
pub fn batch_open(paths: Vec<String>) -> Result<(), String> {
    for path in &paths {
        open::that(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn batch_copy_to(paths: Vec<String>) -> Result<(), String> {
    let paths_arg = paths.iter().map(|p| format!("'{}'", p.replace("'", "''"))).collect::<Vec<_>>().join(",");
    let ps_cmd = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = 'Copy {} items to...'; if ($f.ShowDialog() -eq 'OK') {{ @({}) | ForEach-Object {{ Copy-Item $_ -Destination $f.SelectedPath -Force }} }}"#,
        paths.len(), paths_arg
    );
    std::process::Command::new("powershell").args(["-Command", &ps_cmd]).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn batch_move_to(paths: Vec<String>) -> Result<(), String> {
    let paths_arg = paths.iter().map(|p| format!("'{}'", p.replace("'", "''"))).collect::<Vec<_>>().join(",");
    let ps_cmd = format!(
        r#"Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = 'Move {} items to...'; if ($f.ShowDialog() -eq 'OK') {{ @({}) | ForEach-Object {{ Move-Item $_ -Destination $f.SelectedPath -Force }} }}"#,
        paths.len(), paths_arg
    );
    std::process::Command::new("powershell").args(["-Command", &ps_cmd]).spawn().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn batch_delete(paths: Vec<String>) -> Result<(), String> {
    for path in &paths {
        let ps_cmd = format!(
            "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile('{}', 'OnlyErrorDialogs', 'SendToRecycleBin')",
            path.replace("'", "''")
        );
        std::process::Command::new("powershell").args(["-Command", &ps_cmd]).spawn().map_err(|e| e.to_string())?;
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
