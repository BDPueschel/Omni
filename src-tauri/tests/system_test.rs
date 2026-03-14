use omni_lib::providers::system::SystemProvider;

#[test]
fn test_lock_command() {
    let results = SystemProvider::evaluate("lock");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].category, "System");
    assert!(results[0].title.contains("Lock"));
}

#[test]
fn test_lock_screen_alias() {
    let results = SystemProvider::evaluate("lock screen");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_shutdown_command() {
    let results = SystemProvider::evaluate("shut");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("Shutdown"));
}

#[test]
fn test_no_match() {
    let results = SystemProvider::evaluate("firefox");
    assert!(results.is_empty());
}

#[test]
fn test_partial_match() {
    let results = SystemProvider::evaluate("re");
    assert!(results.len() >= 2); // restart, recycle bin
}

#[test]
fn test_sleep_command() {
    let results = SystemProvider::evaluate("sleep");
    assert_eq!(results.len(), 1);
    assert!(results[0].title.contains("Sleep"));
}
