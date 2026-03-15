use omni_lib::providers::everything::{EverythingProvider, EverythingStatus};

#[test]
fn test_status_detection_when_missing() {
    let status = EverythingProvider::check_status_at_path("C:\\nonexistent\\es.exe");
    assert_eq!(status, EverythingStatus::NotInstalled);
}

#[test]
fn test_format_results() {
    let paths = vec![
        "C:\\Users\\Brian\\Documents\\notes.txt".to_string(),
        "C:\\Projects\\readme.md".to_string(),
    ];
    let results = EverythingProvider::format_results(paths);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].category, "Files");
    assert!(results[0].title.contains("notes.txt"));
    assert_eq!(results[1].subtitle, "C:\\Projects\\readme.md");
}

#[test]
#[ignore] // Requires Everything 1.5a to be running + es.exe in path
fn test_live_search() {
    let results = EverythingProvider::search("Cargo.toml", 5);
    assert!(!results.is_empty());
}

#[test]
#[ignore] // Requires Everything 1.5a to be running + es.exe in path
fn test_live_app_search() {
    let results = EverythingProvider::search_apps("notepad", 5);
    assert!(!results.is_empty());
    assert_eq!(results[0].category, "Apps");
}
