use omni_lib::providers::web_search::WebSearchProvider;

#[test]
fn test_generates_google_and_duckduckgo() {
    let results = WebSearchProvider::evaluate("rust programming", "google");
    assert_eq!(results.len(), 2);
    assert!(results[0].title.contains("Google"));
    assert!(results[1].title.contains("DuckDuckGo"));
}

#[test]
fn test_duckduckgo_preferred() {
    let results = WebSearchProvider::evaluate("test query", "duckduckgo");
    assert_eq!(results[0].title, "Search DuckDuckGo for \"test query\"");
    assert_eq!(results[1].title, "Search Google for \"test query\"");
}

#[test]
fn test_empty_query() {
    let results = WebSearchProvider::evaluate("", "google");
    assert!(results.is_empty());
}

#[test]
fn test_url_encoding() {
    let results = WebSearchProvider::evaluate("hello world", "google");
    if let omni_lib::providers::ResultAction::WebSearch { url } = &results[0].action {
        assert!(url.contains("hello") && url.contains("world"));
        assert!(!url.contains(' ')); // spaces should be encoded
    }
}
