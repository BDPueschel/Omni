use super::{ResultAction, SearchResult};
use regex::Regex;

pub struct UrlProvider;

impl UrlProvider {
    pub fn evaluate(input: &str) -> Vec<SearchResult> {
        let trimmed = input.trim();
        let url_pattern = Regex::new(r"^(https?://[^\s]+|www\.[^\s]+\.[^\s]+)$").unwrap();
        if !url_pattern.is_match(trimmed) {
            return vec![];
        }
        let url = if trimmed.starts_with("www.") {
            format!("https://{}", trimmed)
        } else {
            trimmed.to_string()
        };
        vec![SearchResult {
            category: "URL".to_string(),
            title: format!("Open {}", trimmed),
            subtitle: "Open in default browser".to_string(),
            action: ResultAction::OpenUrl { url },
            icon: "globe".to_string(),
            size: None,
            date_modified: None,
        }]
    }
}
