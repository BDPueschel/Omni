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

// --- HTTP API response types ---

#[derive(Debug, serde::Deserialize)]
struct EverythingHttpResult {
    #[serde(rename = "type")]
    result_type: String,
    name: String,
    #[serde(default)]
    path: String,
}

#[derive(Debug, serde::Deserialize)]
struct EverythingResponse {
    #[serde(rename = "totalResults")]
    #[allow(dead_code)]
    total_results: u64,
    results: Vec<EverythingHttpResult>,
}

pub struct EverythingProvider;

impl EverythingProvider {
    // ---------------------------------------------------------------
    // es.exe discovery & execution (kept for fallback + tab completion)
    // ---------------------------------------------------------------

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

    // ---------------------------------------------------------------
    // HTTP API
    // ---------------------------------------------------------------

    fn query_http(
        query: &str,
        max_results: usize,
        sort: &str,
        ascending: bool,
    ) -> Result<EverythingResponse, String> {
        use std::io::{Read, Write};
        use std::net::TcpStream;

        let encoded_query = {
            use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
            utf8_percent_encode(query, NON_ALPHANUMERIC).to_string()
        };

        let url = format!(
            "/?s={}&c={}&j=1&path_column=1&sort={}&ascending={}",
            encoded_query,
            max_results,
            sort,
            if ascending { 1 } else { 0 }
        );

        let mut stream = TcpStream::connect("127.0.0.1:8080")
            .map_err(|e| format!("Everything HTTP: {}", e))?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .ok();

        let request = format!(
            "GET {} HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            url
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|e| e.to_string())?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| e.to_string())?;

        // Split headers from body
        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .ok_or("Invalid HTTP response")?;

        // Parse JSON
        let parsed: EverythingResponse =
            serde_json::from_str(body).map_err(|e| format!("JSON parse error: {}", e))?;

        Ok(parsed)
    }

    /// Check if the HTTP API is reachable (used for status checks).
    fn http_is_available() -> bool {
        use std::net::TcpStream;
        TcpStream::connect("127.0.0.1:8080").is_ok()
    }

    // ---------------------------------------------------------------
    // Query helpers (unchanged)
    // ---------------------------------------------------------------

    pub fn check_status_at_path(path: &str) -> EverythingStatus {
        if !Path::new(path).exists() {
            return EverythingStatus::NotInstalled;
        }
        EverythingStatus::NotRunning
    }

    pub fn check_status() -> EverythingStatus {
        // Try HTTP first — it's faster and doesn't need es.exe
        if Self::http_is_available() {
            return EverythingStatus::Ready;
        }
        // Fall back to es.exe check
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

    /// Detect if query contains a path prefix (e.g. "C:\", "F:/", "\\server").
    /// Returns (path, remaining_query) if found, or None.
    fn parse_path_context(query: &str) -> Option<(String, String)> {
        let trimmed = query.trim();

        // Match drive letter paths: C:\..., C:/...
        if trimmed.len() >= 3
            && trimmed.as_bytes()[0].is_ascii_alphabetic()
            && trimmed.as_bytes()[1] == b':'
            && (trimmed.as_bytes()[2] == b'\\' || trimmed.as_bytes()[2] == b'/')
        {
            // Find the last separator — everything before is path, after is query
            let normalized = trimmed.replace('/', "\\");
            if let Some(last_sep) = normalized.rfind('\\') {
                let path = &normalized[..=last_sep];
                let remainder = normalized[last_sep + 1..].trim().to_string();
                return Some((path.to_string(), remainder));
            }
        }

        // Match UNC paths: \\server\share\...
        if trimmed.starts_with("\\\\") {
            let normalized = trimmed.replace('/', "\\");
            if let Some(last_sep) = normalized.rfind('\\') {
                if last_sep > 1 {
                    let path = &normalized[..=last_sep];
                    let remainder = normalized[last_sep + 1..].trim().to_string();
                    return Some((path.to_string(), remainder));
                }
            }
        }

        None
    }

    /// Check if query uses Everything-style operators (pass raw, don't wildcardify).
    fn is_raw_query(query: &str) -> bool {
        let q = query.trim();
        q.contains('*') || q.contains('?') || q.contains('|')
            || q.contains("ext:") || q.contains("size:")
            || q.contains("dm:") || q.contains("dc:")
            || q.contains("regex:") || q.contains("!")
            || q.contains("parent:") || q.contains("startwith:")
            || q.contains("endwith:")
            || q.starts_with("r:")
    }

    /// Check if query is a regex search (regex: or r: prefix).
    /// Returns Some(pattern) with the prefix stripped, or None.
    fn parse_regex_prefix(query: &str) -> Option<String> {
        let q = query.trim();
        if let Some(rest) = q.strip_prefix("regex:") {
            Some(rest.trim().to_string())
        } else if let Some(rest) = q.strip_prefix("r:") {
            Some(rest.trim().to_string())
        } else {
            None
        }
    }

    /// Build the Everything search query string, handling path context, regex, and raw operators.
    /// Returns the processed query string for the HTTP API.
    fn build_http_query(query: &str) -> String {
        // Regex mode: pass regex: prefix through to Everything
        if let Some(pattern) = Self::parse_regex_prefix(query) {
            return format!("regex:{}", pattern);
        }

        if let Some((path, remainder)) = Self::parse_path_context(query) {
            let search_part = if remainder.is_empty() {
                "*".to_string()
            } else if Self::is_raw_query(&remainder) {
                remainder
            } else {
                Self::wildcardify(&remainder)
            };
            format!("\"{}\" {}", path, search_part)
        } else if Self::is_raw_query(query) {
            query.trim().to_string()
        } else {
            Self::wildcardify(query)
        }
    }

    /// Build es.exe args for a query, handling path context, regex, and raw operators.
    fn build_search_args(query: &str, max_results: usize, extra_flags: &[&str]) -> Vec<String> {
        let max_str = max_results.to_string();
        let mut args: Vec<String> = vec!["-n".to_string(), max_str];

        for flag in extra_flags {
            args.push(flag.to_string());
        }

        // Regex mode: strip prefix, pass -r flag and raw pattern
        if let Some(pattern) = Self::parse_regex_prefix(query) {
            args.push("-r".to_string());
            args.push(pattern);
            return args;
        }

        if let Some((path, remainder)) = Self::parse_path_context(query) {
            args.push("-path".to_string());
            args.push(path);
            if remainder.is_empty() {
                args.push("*".to_string());
            } else if Self::is_raw_query(&remainder) {
                args.push(remainder);
            } else {
                args.push(Self::wildcardify(&remainder));
            }
        } else if Self::is_raw_query(query) {
            // Pass through raw — user is using Everything operators
            args.push(query.trim().to_string());
        } else {
            args.push(Self::wildcardify(query));
        }

        args
    }

    /// Check if es.exe is available, return error result if not.
    fn unavailable_result() -> Vec<SearchResult> {
        vec![SearchResult {
            category: "Files".to_string(),
            title: "Everything search not available".to_string(),
            subtitle: "es.exe not found and HTTP API unreachable — install Everything"
                .to_string(),
            action: ResultAction::OpenUrl {
                url: "https://www.voidtools.com/downloads/".to_string(),
            },
            icon: "alert".to_string(),
        }]
    }

    // ---------------------------------------------------------------
    // Combined HTTP search with es.exe fallback
    // ---------------------------------------------------------------

    /// Search files and directories in a single HTTP request, splitting by type.
    /// Falls back to es.exe if HTTP is unavailable.
    pub fn search_all(query: &str, max_per_type: usize) -> (Vec<SearchResult>, Vec<SearchResult>) {
        let http_query = Self::build_http_query(query);

        // Try HTTP first — single request for both files and dirs
        match Self::query_http(&http_query, max_per_type * 2, "date_modified", false) {
            Ok(response) => {
                let mut files = Vec::new();
                let mut dirs = Vec::new();
                for r in response.results {
                    let full_path = if r.path.is_empty() {
                        r.name.clone()
                    } else {
                        format!("{}\\{}", r.path, r.name)
                    };
                    if r.result_type == "folder" && dirs.len() < max_per_type {
                        let dirname = Path::new(&full_path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        dirs.push(SearchResult {
                            category: "Directories".to_string(),
                            title: dirname,
                            subtitle: full_path.clone(),
                            action: ResultAction::OpenFile {
                                path: full_path,
                            },
                            icon: "folder".to_string(),
                        });
                    } else if r.result_type == "file" && files.len() < max_per_type {
                        let filename = Path::new(&full_path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        files.push(SearchResult {
                            category: "Files".to_string(),
                            title: filename,
                            subtitle: full_path.clone(),
                            action: ResultAction::OpenFile {
                                path: full_path,
                            },
                            icon: "file".to_string(),
                        });
                    }
                }
                (files, dirs)
            }
            Err(e) => {
                eprintln!("Everything HTTP failed, falling back to es.exe: {}", e);
                let files = Self::search_files_es(query, max_per_type);
                let dirs = Self::search_dirs_es(query, max_per_type);
                (files, dirs)
            }
        }
    }

    // ---------------------------------------------------------------
    // Public search methods (HTTP primary, es.exe fallback)
    // ---------------------------------------------------------------

    /// Search for files only (no directories).
    pub fn search_files(query: &str, max_results: usize) -> Vec<SearchResult> {
        let http_query = format!("file: {}", Self::build_http_query(query));
        match Self::query_http(&http_query, max_results, "date_modified", false) {
            Ok(response) => Self::format_http_file_results(response.results),
            Err(e) => {
                eprintln!("Everything HTTP failed for files, falling back to es.exe: {}", e);
                Self::search_files_es(query, max_results)
            }
        }
    }

    /// Search for directories only.
    pub fn search_dirs(query: &str, max_results: usize) -> Vec<SearchResult> {
        let http_query = format!("folder: {}", Self::build_http_query(query));
        match Self::query_http(&http_query, max_results, "date_modified", false) {
            Ok(response) => Self::format_http_dir_results(response.results),
            Err(e) => {
                eprintln!("Everything HTTP failed for dirs, falling back to es.exe: {}", e);
                Self::search_dirs_es(query, max_results)
            }
        }
    }

    /// General search (files and dirs) — used by tests and expand.
    pub fn search(query: &str, max_results: usize) -> Vec<SearchResult> {
        let http_query = Self::build_http_query(query);
        match Self::query_http(&http_query, max_results, "date_modified", false) {
            Ok(response) => Self::format_http_file_results(response.results),
            Err(e) => {
                eprintln!("Everything HTTP failed for search, falling back to es.exe: {}", e);
                Self::search_es(query, max_results)
            }
        }
    }

    /// Search for applications (.lnk in Start Menu, .exe in Program Files).
    pub fn search_apps(query: &str, max_results: usize) -> Vec<SearchResult> {
        let wildcard = Self::wildcardify(query);

        // Try HTTP: search for .lnk and .exe in relevant paths
        let http_query = format!(
            "{}.lnk | {}.exe \"C:\\ProgramData\\Microsoft\\Windows\\Start Menu\" | \"C:\\Users\\Brian\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\" | \"C:\\Program Files\" | \"C:\\Program Files (x86)\"",
            wildcard, wildcard
        );
        match Self::query_http(&http_query, max_results * 2, "date_modified", false) {
            Ok(response) => {
                let mut results = Vec::new();
                let mut seen = std::collections::HashSet::new();
                for r in response.results {
                    let full_path = if r.path.is_empty() {
                        r.name.clone()
                    } else {
                        format!("{}\\{}", r.path, r.name)
                    };
                    let stem = Path::new(&full_path)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase();
                    if seen.insert(stem.clone()) {
                        results.push(SearchResult {
                            category: "Apps".to_string(),
                            title: Path::new(&full_path)
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            subtitle: full_path.clone(),
                            action: ResultAction::LaunchApp { path: full_path },
                            icon: "app".to_string(),
                        });
                    }
                    if results.len() >= max_results {
                        break;
                    }
                }
                results
            }
            Err(e) => {
                eprintln!("Everything HTTP failed for apps, falling back to es.exe: {}", e);
                Self::search_apps_es(query, max_results)
            }
        }
    }

    // ---------------------------------------------------------------
    // es.exe fallback methods
    // ---------------------------------------------------------------

    fn search_files_es(query: &str, max_results: usize) -> Vec<SearchResult> {
        if Self::find_es_exe().is_none() {
            return Self::unavailable_result();
        }

        let args =
            Self::build_search_args(query, max_results, &["-a-d", "-sort-date-modified-descending"]);
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match Self::run_es(&arg_refs) {
            Ok(paths) => Self::format_file_results(paths),
            Err(e) => {
                eprintln!("Everything file search error: {}", e);
                vec![]
            }
        }
    }

    fn search_dirs_es(query: &str, max_results: usize) -> Vec<SearchResult> {
        if Self::find_es_exe().is_none() {
            return vec![];
        }

        let args = Self::build_search_args(query, max_results, &["-ad"]);
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match Self::run_es(&arg_refs) {
            Ok(paths) => Self::format_dir_results(paths),
            Err(e) => {
                eprintln!("Everything dir search error: {}", e);
                vec![]
            }
        }
    }

    fn search_es(query: &str, max_results: usize) -> Vec<SearchResult> {
        if Self::find_es_exe().is_none() {
            return Self::unavailable_result();
        }
        let args = Self::build_search_args(query, max_results, &[]);
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match Self::run_es(&arg_refs) {
            Ok(paths) => Self::format_file_results(paths),
            Err(e) => {
                eprintln!("Everything search error: {}", e);
                vec![]
            }
        }
    }

    fn search_apps_es(query: &str, max_results: usize) -> Vec<SearchResult> {
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
            for prog_dir in &["C:\\Program Files", "C:\\Program Files (x86)"] {
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

    // ---------------------------------------------------------------
    // Result formatting
    // ---------------------------------------------------------------

    fn format_http_file_results(results: Vec<EverythingHttpResult>) -> Vec<SearchResult> {
        results
            .into_iter()
            .map(|r| {
                let full_path = if r.path.is_empty() {
                    r.name.clone()
                } else {
                    format!("{}\\{}", r.path, r.name)
                };
                let filename = Path::new(&full_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                SearchResult {
                    category: "Files".to_string(),
                    title: filename,
                    subtitle: full_path.clone(),
                    action: ResultAction::OpenFile { path: full_path },
                    icon: "file".to_string(),
                }
            })
            .collect()
    }

    fn format_http_dir_results(results: Vec<EverythingHttpResult>) -> Vec<SearchResult> {
        results
            .into_iter()
            .map(|r| {
                let full_path = if r.path.is_empty() {
                    r.name.clone()
                } else {
                    format!("{}\\{}", r.path, r.name)
                };
                let dirname = Path::new(&full_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                SearchResult {
                    category: "Directories".to_string(),
                    title: dirname,
                    subtitle: full_path.clone(),
                    action: ResultAction::OpenFile { path: full_path },
                    icon: "folder".to_string(),
                }
            })
            .collect()
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

    /// Complete a partial path to matching directories (for Tab completion).
    pub fn complete_path(partial: &str, max: usize) -> Vec<String> {
        // Try HTTP first
        let query = format!("folder: {}*", partial);
        if let Ok(response) = Self::query_http(&query, max, "name", true) {
            let paths: Vec<String> = response
                .results
                .into_iter()
                .map(|r| {
                    if r.path.is_empty() {
                        r.name
                    } else {
                        format!("{}\\{}", r.path, r.name)
                    }
                })
                .collect();
            if !paths.is_empty() {
                return paths;
            }
        }

        // Fall back to es.exe
        if Self::find_es_exe().is_none() {
            return vec![];
        }
        let es_query = format!("{}*", partial);
        let max_str = max.to_string();
        match Self::run_es(&["-n", &max_str, "-ad", &es_query]) {
            Ok(paths) => paths,
            Err(_) => vec![],
        }
    }

    pub fn format_results(paths: Vec<String>) -> Vec<SearchResult> {
        Self::format_file_results(paths)
    }
}
