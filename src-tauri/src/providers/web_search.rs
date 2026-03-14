use super::{ResultAction, SearchResult};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

pub struct WebSearchProvider;

impl WebSearchProvider {
    pub fn evaluate(input: &str, preferred_engine: &str) -> Vec<SearchResult> {
        let query = input.trim();
        if query.is_empty() {
            return vec![];
        }
        let encoded = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
        let google = SearchResult {
            category: "Web".to_string(),
            title: format!("Search Google for \"{}\"", query),
            subtitle: "Open in browser".to_string(),
            action: ResultAction::WebSearch { url: format!("https://www.google.com/search?q={}", encoded) },
            icon: "search".to_string(),
        };
        let duckduckgo = SearchResult {
            category: "Web".to_string(),
            title: format!("Search DuckDuckGo for \"{}\"", query),
            subtitle: "Open in browser".to_string(),
            action: ResultAction::WebSearch { url: format!("https://duckduckgo.com/?q={}", encoded) },
            icon: "search".to_string(),
        };
        if preferred_engine == "duckduckgo" {
            vec![duckduckgo, google]
        } else {
            vec![google, duckduckgo]
        }
    }
}
