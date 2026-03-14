use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OmniConfig {
    pub hotkey: String,
    pub max_results_per_category: usize,
    pub search_engine: String,
    pub start_with_windows: bool,
    pub theme_opacity: u8,
}

impl Default for OmniConfig {
    fn default() -> Self {
        Self {
            hotkey: "Alt+Space".to_string(),
            max_results_per_category: 5,
            search_engine: "google".to_string(),
            start_with_windows: true,
            theme_opacity: 80,
        }
    }
}

impl OmniConfig {
    pub fn config_path() -> PathBuf {
        let app_data = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        app_data.join("Omni").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }
}
