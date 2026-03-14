use omni_lib::providers::apps::{AppEntry, AppProvider};

#[test]
fn test_fuzzy_match_exact() {
    let apps = vec![
        AppEntry { name: "Firefox".to_string(), path: "C:\\Program Files\\Mozilla Firefox\\firefox.exe".to_string() },
        AppEntry { name: "FileZilla".to_string(), path: "C:\\Program Files\\FileZilla\\filezilla.exe".to_string() },
    ];
    let results = AppProvider::search(&apps, "firefox", 5);
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("Firefox"));
}

#[test]
fn test_fuzzy_match_partial() {
    let apps = vec![
        AppEntry { name: "Firefox".to_string(), path: "C:\\path\\firefox.exe".to_string() },
        AppEntry { name: "FileZilla".to_string(), path: "C:\\path\\filezilla.exe".to_string() },
        AppEntry { name: "Notepad".to_string(), path: "C:\\path\\notepad.exe".to_string() },
    ];
    let results = AppProvider::search(&apps, "fi", 5);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_empty_query() {
    let apps = vec![AppEntry { name: "Firefox".to_string(), path: "C:\\path\\firefox.exe".to_string() }];
    let results = AppProvider::search(&apps, "", 5);
    assert!(results.is_empty());
}

#[test]
fn test_max_results() {
    let apps: Vec<AppEntry> = (0..20).map(|i| AppEntry {
        name: format!("App{}", i),
        path: format!("C:\\path\\app{}.exe", i),
    }).collect();
    let results = AppProvider::search(&apps, "App", 5);
    assert_eq!(results.len(), 5);
}
