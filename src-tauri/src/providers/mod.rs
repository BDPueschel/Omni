pub mod apps;
pub mod everything;
pub mod math;
pub mod url;
pub mod system;
pub mod web_search;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub category: String,
    pub title: String,
    pub subtitle: String,
    pub action: ResultAction,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ResultAction {
    #[serde(rename = "copy")]
    Copy { text: String },
    #[serde(rename = "open_file")]
    OpenFile { path: String },
    #[serde(rename = "open_url")]
    OpenUrl { url: String },
    #[serde(rename = "launch_app")]
    LaunchApp { path: String },
    #[serde(rename = "system_command")]
    SystemCommand { command: String },
    #[serde(rename = "web_search")]
    WebSearch { url: String },
}
