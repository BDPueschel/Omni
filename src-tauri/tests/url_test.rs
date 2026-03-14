use omni_lib::providers::url::UrlProvider;

#[test]
fn test_detects_https_url() {
    let results = UrlProvider::evaluate("https://example.com");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].category, "URL");
}

#[test]
fn test_detects_http_url() {
    let results = UrlProvider::evaluate("http://example.com");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_detects_www_url() {
    let results = UrlProvider::evaluate("www.example.com");
    assert_eq!(results.len(), 1);
    if let omni_lib::providers::ResultAction::OpenUrl { url } = &results[0].action {
        assert!(url.starts_with("https://"));
    } else {
        panic!("Expected OpenUrl action");
    }
}

#[test]
fn test_ignores_plain_text() {
    let results = UrlProvider::evaluate("hello world");
    assert!(results.is_empty());
}

#[test]
fn test_ignores_file_path() {
    let results = UrlProvider::evaluate("C:\\Users\\test\\file.txt");
    assert!(results.is_empty());
}
