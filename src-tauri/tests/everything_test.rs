use omni_lib::providers::everything::{filetime_to_unix, EverythingProvider, EverythingStatus};

#[test]
fn test_filetime_to_unix_epoch() {
    // 2024-01-01 00:00:00 UTC = FILETIME 133485408000000000
    let ft: u64 = 133485408000000000;
    let epoch = filetime_to_unix(ft);
    assert_eq!(epoch, 1704067200);
}

#[test]
fn test_filetime_to_unix_epoch_zero() {
    // FILETIME before Unix epoch should clamp to 0
    let epoch = filetime_to_unix(0);
    assert_eq!(epoch, 0);
}

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
