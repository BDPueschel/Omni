use super::{ResultAction, SearchResult};
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

/// Path to es.exe — resolved once at startup.
static ES_EXE_PATH: OnceLock<Option<String>> = OnceLock::new();

#[derive(Debug, PartialEq)]
pub enum EverythingStatus {
    Ready,
    NotInstalled,
    NotRunning,
}

pub struct EverythingProvider;

impl EverythingProvider {
    fn find_es_exe() -> &'static Option<String> {
        ES_EXE_PATH.get_or_init(|| {
            // Check next to our own executable and parent dirs
            // (test binaries live in target/debug/deps/, but es.exe is in target/debug/)
            if let Ok(exe) = std::env::current_exe() {
                let mut dir = exe.parent();
                for _ in 0..3 {
                    if let Some(d) = dir {
                        let candidate = d.join("es.exe");
                        if candidate.exists() {
                            return Some(candidate.to_string_lossy().to_string());
                        }
                        dir = d.parent();
                    }
                }
            }
            // Check common install locations
            for path in &[
                "C:\\Program Files\\Everything\\es.exe",
                "C:\\Program Files\\Everything 1.5a\\es.exe",
                "C:\\Program Files (x86)\\Everything\\es.exe",
            ] {
                if Path::new(path).exists() {
                    return Some(path.to_string());
                }
            }
            None
        })
    }

    fn run_es(args: &[&str]) -> Result<Vec<String>, String> {
        let es_path = match Self::find_es_exe() {
            Some(path) => path,
            None => return Err("es.exe not found".to_string()),
        };

        let output = Command::new(es_path)
            .args(["-instance", "1.5a"])
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run es.exe: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("es.exe error: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect())
    }

    pub fn check_status_at_path(path: &str) -> EverythingStatus {
        if !Path::new(path).exists() {
            return EverythingStatus::NotInstalled;
        }
        EverythingStatus::NotRunning
    }

    pub fn check_status() -> EverythingStatus {
        if Self::find_es_exe().is_none() {
            return EverythingStatus::NotInstalled;
        }
        match Self::run_es(&["-n", "1", "OMNI_HEALTH_CHECK"]) {
            Ok(_) => EverythingStatus::Ready,
            Err(_) => EverythingStatus::NotRunning,
        }
    }

    /// Convert "car toml" into "*car*toml*" for fuzzy fragment matching.
    fn wildcardify(query: &str) -> String {
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }
        format!("*{}*", parts.join("*"))
    }

    /// Check if es.exe is available, return error result if not.
    fn unavailable_result() -> Vec<SearchResult> {
        vec![SearchResult {
            category: "Files".to_string(),
            title: "Everything search not available".to_string(),
            subtitle: "es.exe not found — reinstall Omni or Everything".to_string(),
            action: ResultAction::OpenUrl {
                url: "https://www.voidtools.com/downloads/".to_string(),
            },
            icon: "alert".to_string(),
        }]
    }

    /// Search for files only (no directories).
    pub fn search_files(query: &str, max_results: usize) -> Vec<SearchResult> {
        if Self::find_es_exe().is_none() {
            return Self::unavailable_result();
        }

        let wildcard_query = Self::wildcardify(query);
        let max_str = max_results.to_string();
        match Self::run_es(&["-n", &max_str, "-a-d", "-sort-date-modified-descending", &wildcard_query]) {
            Ok(paths) => Self::format_file_results(paths),
            Err(e) => {
                eprintln!("Everything file search error: {}", e);
                vec![]
            }
        }
    }

    /// Search for directories only.
    pub fn search_dirs(query: &str, max_results: usize) -> Vec<SearchResult> {
        if Self::find_es_exe().is_none() {
            return vec![];
        }

        let wildcard_query = Self::wildcardify(query);
        let max_str = max_results.to_string();
        match Self::run_es(&["-n", &max_str, "-ad", &wildcard_query]) {
            Ok(paths) => Self::format_dir_results(paths),
            Err(e) => {
                eprintln!("Everything dir search error: {}", e);
                vec![]
            }
        }
    }

    /// Legacy method for tests — searches both files and dirs.
    pub fn search(query: &str, max_results: usize) -> Vec<SearchResult> {
        if Self::find_es_exe().is_none() {
            return Self::unavailable_result();
        }
        let wildcard_query = Self::wildcardify(query);
        let max_str = max_results.to_string();
        match Self::run_es(&["-n", &max_str, &wildcard_query]) {
            Ok(paths) => Self::format_file_results(paths),
            Err(e) => {
                eprintln!("Everything search error: {}", e);
                vec![]
            }
        }
    }

    /// Search for applications (.lnk in Start Menu, .exe in Program Files).
    pub fn search_apps(query: &str, max_results: usize) -> Vec<SearchResult> {
        if Self::find_es_exe().is_none() {
            return vec![];
        }

        let max_str = max_results.to_string();
        let mut all_paths = Vec::new();

        // Search Start Menu shortcuts (.lnk) — best source for apps
        let wildcard = Self::wildcardify(query);
        let lnk_query = format!("{}.lnk", wildcard);
        for start_menu in &[
            "C:\\ProgramData\\Microsoft\\Windows\\Start Menu",
            "C:\\Users\\Brian\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu",
        ] {
            if let Ok(paths) = Self::run_es(&["-n", &max_str, &lnk_query, "-path", start_menu]) {
                all_paths.extend(paths);
            }
        }

        // Also search for .exe in Program Files if we have few results
        if all_paths.len() < max_results {
            let exe_query = format!("{}.exe", wildcard);
            for prog_dir in &[
                "C:\\Program Files",
                "C:\\Program Files (x86)",
            ] {
                if let Ok(paths) = Self::run_es(&["-n", "3", &exe_query, "-path", prog_dir]) {
                    all_paths.extend(paths);
                }
            }
        }

        // Deduplicate by filename stem (prefer .lnk over .exe)
        let mut seen = std::collections::HashSet::new();
        all_paths.retain(|p| {
            let stem = Path::new(p)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            seen.insert(stem)
        });

        all_paths.truncate(max_results);
        Self::format_app_results(all_paths)
    }

    fn format_file_results(paths: Vec<String>) -> Vec<SearchResult> {
        paths
            .into_iter()
            .map(|path| {
                let filename = Path::new(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                SearchResult {
                    category: "Files".to_string(),
                    title: filename,
                    subtitle: path.clone(),
                    action: ResultAction::OpenFile { path },
                    icon: "file".to_string(),
                }
            })
            .collect()
    }

    fn format_app_results(paths: Vec<String>) -> Vec<SearchResult> {
        paths
            .into_iter()
            .map(|path| {
                let name = Path::new(&path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                SearchResult {
                    category: "Apps".to_string(),
                    title: name,
                    subtitle: path.clone(),
                    action: ResultAction::LaunchApp { path },
                    icon: "app".to_string(),
                }
            })
            .collect()
    }

    fn format_dir_results(paths: Vec<String>) -> Vec<SearchResult> {
        paths
            .into_iter()
            .map(|path| {
                let dirname = Path::new(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                SearchResult {
                    category: "Directories".to_string(),
                    title: dirname,
                    subtitle: path.clone(),
                    action: ResultAction::OpenFile { path },
                    icon: "folder".to_string(),
                }
            })
            .collect()
    }

    pub fn format_results(paths: Vec<String>) -> Vec<SearchResult> {
        Self::format_file_results(paths)
    }
}
