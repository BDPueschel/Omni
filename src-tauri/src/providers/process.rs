use super::{ResultAction, SearchResult};
use std::collections::HashMap;

/// System-critical processes that should never be shown.
const BLOCKED_PROCESSES: &[&str] = &[
    "system", "csrss", "smss", "lsass", "services", "wininit", "svchost",
    "system idle process",
];

pub struct ProcessProvider;

/// A single parsed process entry from tasklist output.
#[derive(Debug, Clone)]
pub struct ProcessEntry {
    pub name: String,
    pub pid: u32,
    pub memory_kb: u64,
}

/// Parse a single CSV line from `tasklist /FO CSV /NH`.
/// Lines look like: "chrome.exe","12345","Console","1","150,000 K"
pub fn parse_tasklist_line(line: &str) -> Option<ProcessEntry> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let fields = parse_csv_fields(line)?;
    if fields.len() < 5 {
        return None;
    }

    let name = fields[0].clone();
    let pid: u32 = fields[1].parse().ok()?;
    let mem_str = &fields[4];
    let memory_kb = parse_memory(mem_str);

    Some(ProcessEntry {
        name,
        pid,
        memory_kb,
    })
}

/// Parse CSV fields, handling quoted values.
fn parse_csv_fields(line: &str) -> Option<Vec<String>> {
    let mut fields = Vec::new();
    let mut chars = line.chars().peekable();

    while chars.peek().is_some() {
        // Skip leading whitespace
        while chars.peek() == Some(&' ') {
            chars.next();
        }

        if chars.peek() == Some(&'"') {
            // Quoted field
            chars.next(); // consume opening quote
            let mut field = String::new();
            loop {
                match chars.next() {
                    Some('"') => {
                        // Check for escaped quote
                        if chars.peek() == Some(&'"') {
                            chars.next();
                            field.push('"');
                        } else {
                            break;
                        }
                    }
                    Some(c) => field.push(c),
                    None => break,
                }
            }
            fields.push(field);
            // Skip comma separator
            if chars.peek() == Some(&',') {
                chars.next();
            }
        } else {
            // Unquoted field
            let mut field = String::new();
            while let Some(&c) = chars.peek() {
                if c == ',' {
                    chars.next();
                    break;
                }
                field.push(c);
                chars.next();
            }
            fields.push(field);
        }
    }

    if fields.is_empty() {
        None
    } else {
        Some(fields)
    }
}

/// Parse memory string like "150,000 K" or "150.000 K" to KB.
fn parse_memory(s: &str) -> u64 {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    cleaned.parse().unwrap_or(0)
}

/// Format KB to a human-readable string.
fn format_memory(kb: u64) -> String {
    if kb >= 1_048_576 {
        format!("{:.1} GB", kb as f64 / 1_048_576.0)
    } else if kb >= 1024 {
        format!("{:.1} MB", kb as f64 / 1024.0)
    } else {
        format!("{} KB", kb)
    }
}

impl ProcessProvider {
    pub fn evaluate(input: &str) -> Vec<SearchResult> {
        let query_lower = input.trim().to_lowercase();
        if !query_lower.starts_with("kill ") {
            return vec![];
        }

        let search_term = query_lower.strip_prefix("kill ").unwrap().trim();
        if search_term.is_empty() {
            return vec![];
        }

        let entries = match Self::get_process_list() {
            Some(entries) => entries,
            None => return vec![],
        };

        Self::build_results(&entries, search_term)
    }

    fn get_process_list() -> Option<Vec<ProcessEntry>> {
        let output = std::process::Command::new("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let entries: Vec<ProcessEntry> = stdout
            .lines()
            .filter_map(parse_tasklist_line)
            .collect();

        Some(entries)
    }

    /// Build deduplicated, filtered results from process entries.
    pub fn build_results(entries: &[ProcessEntry], search_term: &str) -> Vec<SearchResult> {
        // Group by lowercase process name
        let mut groups: HashMap<String, (String, Vec<u32>, u64)> = HashMap::new();

        for entry in entries {
            let key = entry.name.to_lowercase();

            // Filter out blocked processes
            let name_no_ext = key.strip_suffix(".exe").unwrap_or(&key);
            if BLOCKED_PROCESSES.contains(&name_no_ext) {
                continue;
            }

            let group = groups.entry(key).or_insert_with(|| {
                (entry.name.clone(), Vec::new(), 0)
            });
            group.1.push(entry.pid);
            group.2 += entry.memory_kb;
        }

        // Filter by search term (case-insensitive contains)
        let mut results: Vec<SearchResult> = groups
            .iter()
            .filter(|(key, _)| key.contains(search_term))
            .map(|(_, (name, pids, total_mem))| {
                let instance_count = pids.len();
                let pid = pids[0]; // Use first PID for the action

                let instances_str = if instance_count > 1 {
                    format!(" ({} instances)", instance_count)
                } else {
                    String::new()
                };

                let subtitle = format!(
                    "{}{}",
                    format_memory(*total_mem),
                    instances_str,
                );

                SearchResult {
                    category: "Processes".to_string(),
                    title: name.clone(),
                    subtitle,
                    action: ResultAction::KillProcess {
                        pid,
                        name: name.clone(),
                    },
                    icon: "process".to_string(),
                    size: None,
                    date_modified: None,
                }
            })
            .collect();

        // Sort by total memory descending (heaviest processes first)
        results.sort_by(|a, b| b.subtitle.cmp(&a.subtitle));

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tasklist_line_basic() {
        let line = r#""chrome.exe","12345","Console","1","150,000 K""#;
        let entry = parse_tasklist_line(line).unwrap();
        assert_eq!(entry.name, "chrome.exe");
        assert_eq!(entry.pid, 12345);
        assert_eq!(entry.memory_kb, 150000);
    }

    #[test]
    fn test_parse_tasklist_line_dot_separator() {
        // Some locales use dots as thousand separators
        let line = r#""notepad.exe","5678","Console","1","12.345 K""#;
        let entry = parse_tasklist_line(line).unwrap();
        assert_eq!(entry.name, "notepad.exe");
        assert_eq!(entry.pid, 5678);
        assert_eq!(entry.memory_kb, 12345);
    }

    #[test]
    fn test_parse_empty_line() {
        assert!(parse_tasklist_line("").is_none());
        assert!(parse_tasklist_line("   ").is_none());
    }

    #[test]
    fn test_build_results_filters_blocked() {
        let entries = vec![
            ProcessEntry { name: "svchost.exe".to_string(), pid: 1, memory_kb: 1000 },
            ProcessEntry { name: "chrome.exe".to_string(), pid: 2, memory_kb: 5000 },
        ];
        let results = ProcessProvider::build_results(&entries, "");
        // svchost should be filtered out
        assert!(results.iter().all(|r| r.title != "svchost.exe"));
        assert!(results.iter().any(|r| r.title == "chrome.exe"));
    }

    #[test]
    fn test_build_results_deduplicates() {
        let entries = vec![
            ProcessEntry { name: "chrome.exe".to_string(), pid: 100, memory_kb: 5000 },
            ProcessEntry { name: "chrome.exe".to_string(), pid: 101, memory_kb: 3000 },
            ProcessEntry { name: "chrome.exe".to_string(), pid: 102, memory_kb: 2000 },
        ];
        let results = ProcessProvider::build_results(&entries, "chrome");
        assert_eq!(results.len(), 1);
        assert!(results[0].subtitle.contains("3 instances"));
    }

    #[test]
    fn test_build_results_no_match() {
        let entries = vec![
            ProcessEntry { name: "chrome.exe".to_string(), pid: 100, memory_kb: 5000 },
        ];
        let results = ProcessProvider::build_results(&entries, "firefox");
        assert!(results.is_empty());
    }

    #[test]
    fn test_format_memory() {
        assert_eq!(format_memory(500), "500 KB");
        assert_eq!(format_memory(2048), "2.0 MB");
        assert_eq!(format_memory(1_572_864), "1.5 GB");
    }
}
