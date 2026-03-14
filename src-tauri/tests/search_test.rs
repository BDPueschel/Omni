use omni_lib::providers::apps::AppEntry;
use omni_lib::search::search_query;

#[test]
fn test_math_query_returns_math_and_suppresses_web() {
    let apps = vec![];
    let results = search_query("2 + 3", &apps, 5, "google");
    assert!(results.iter().any(|r| r.category == "Math"));
    assert!(results.iter().all(|r| r.category != "Web"));
}

#[test]
fn test_url_query_suppresses_web() {
    let apps = vec![];
    let results = search_query("https://example.com", &apps, 5, "google");
    assert!(results.iter().any(|r| r.category == "URL"));
    assert!(results.iter().all(|r| r.category != "Web"));
}

#[test]
fn test_text_query_includes_web_fallback() {
    let apps = vec![];
    let results = search_query("some random text", &apps, 5, "google");
    assert!(results.iter().any(|r| r.category == "Web"));
}

#[test]
fn test_app_match() {
    let apps = vec![AppEntry {
        name: "Firefox".to_string(),
        path: "C:\\path\\firefox.lnk".to_string(),
    }];
    let results = search_query("fire", &apps, 5, "google");
    assert!(results.iter().any(|r| r.category == "Apps" && r.title == "Firefox"));
}

#[test]
fn test_system_command_match() {
    let apps = vec![];
    let results = search_query("lock", &apps, 5, "google");
    assert!(results.iter().any(|r| r.category == "System"));
}

#[test]
fn test_empty_query_returns_empty() {
    let apps = vec![];
    let results = search_query("", &apps, 5, "google");
    assert!(results.is_empty());
}

#[test]
fn test_unknown_system_command() {
    let result = omni_lib::search::execute_system_command_dry("nonexistent");
    assert!(result.is_err());
}
