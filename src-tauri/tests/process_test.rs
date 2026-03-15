use omni_lib::providers::process::{parse_tasklist_line, ProcessProvider};

#[test]
fn test_no_kill_prefix_returns_empty() {
    let results = ProcessProvider::evaluate("firefox");
    assert!(results.is_empty());
}

#[test]
fn test_kill_empty_name_returns_empty() {
    let results = ProcessProvider::evaluate("kill ");
    assert!(results.is_empty());
}

#[test]
fn test_kill_whitespace_only_returns_empty() {
    let results = ProcessProvider::evaluate("kill   ");
    assert!(results.is_empty());
}

#[test]
fn test_parse_csv_line() {
    let line = r#""chrome.exe","12345","Console","1","150,000 K""#;
    let entry = parse_tasklist_line(line).unwrap();
    assert_eq!(entry.name, "chrome.exe");
    assert_eq!(entry.pid, 12345);
    assert_eq!(entry.memory_kb, 150000);
}

#[test]
fn test_parse_csv_line_dot_thousands() {
    let line = r#""notepad.exe","999","Console","1","12.345 K""#;
    let entry = parse_tasklist_line(line).unwrap();
    assert_eq!(entry.name, "notepad.exe");
    assert_eq!(entry.pid, 999);
    assert_eq!(entry.memory_kb, 12345);
}

#[test]
fn test_parse_csv_empty_line() {
    assert!(parse_tasklist_line("").is_none());
}

#[test]
fn test_build_results_deduplication() {
    use omni_lib::providers::process::ProcessEntry;
    let entries = vec![
        ProcessEntry { name: "chrome.exe".to_string(), pid: 100, memory_kb: 50000 },
        ProcessEntry { name: "chrome.exe".to_string(), pid: 101, memory_kb: 30000 },
        ProcessEntry { name: "chrome.exe".to_string(), pid: 102, memory_kb: 20000 },
    ];
    let results = ProcessProvider::build_results(&entries, "chrome");
    assert_eq!(results.len(), 1);
    assert!(results[0].subtitle.contains("3 instances"));
    assert_eq!(results[0].category, "Processes");
}

#[test]
fn test_build_results_filters_svchost() {
    use omni_lib::providers::process::ProcessEntry;
    let entries = vec![
        ProcessEntry { name: "svchost.exe".to_string(), pid: 1, memory_kb: 1000 },
        ProcessEntry { name: "csrss.exe".to_string(), pid: 2, memory_kb: 500 },
        ProcessEntry { name: "notepad.exe".to_string(), pid: 3, memory_kb: 2000 },
    ];
    let results = ProcessProvider::build_results(&entries, "");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "notepad.exe");
}

#[test]
fn test_build_results_case_insensitive_search() {
    use omni_lib::providers::process::ProcessEntry;
    let entries = vec![
        ProcessEntry { name: "Chrome.exe".to_string(), pid: 100, memory_kb: 50000 },
    ];
    let results = ProcessProvider::build_results(&entries, "chrome");
    assert_eq!(results.len(), 1);
}

#[test]
#[ignore] // Requires actual running processes
fn test_kill_chrome_returns_results() {
    let results = ProcessProvider::evaluate("kill chrome");
    // Only passes if Chrome is running
    assert!(!results.is_empty());
    assert_eq!(results[0].category, "Processes");
}
