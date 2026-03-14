use super::{ResultAction, SearchResult};
use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
}

pub struct AppProvider;

impl AppProvider {
    /// Scan Start Menu directories for .lnk shortcut files.
    /// Note: Most UWP/Store apps also place shortcuts in the Start Menu,
    /// so this covers the majority of installed apps.
    pub fn scan_start_menu() -> Vec<AppEntry> {
        let mut apps = Vec::new();
        let start_menu_paths: Vec<PathBuf> = vec![
            dirs::data_dir()
                .unwrap_or_default()
                .join("Microsoft\\Windows\\Start Menu\\Programs"),
            PathBuf::from(std::env::var("ProgramData").unwrap_or_default())
                .join("Microsoft\\Windows\\Start Menu\\Programs"),
        ];
        for base in start_menu_paths {
            if base.exists() {
                Self::scan_directory(&base, &mut apps);
            }
        }
        apps.dedup_by(|a, b| a.name == b.name);
        apps
    }

    fn scan_directory(dir: &Path, apps: &mut Vec<AppEntry>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::scan_directory(&path, apps);
                } else if path.extension().is_some_and(|ext| ext == "lnk") {
                    let name = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    apps.push(AppEntry {
                        name,
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }

    /// Fuzzy search the app list, returning up to `max` results.
    pub fn search(apps: &[AppEntry], query: &str, max: usize) -> Vec<SearchResult> {
        let query = query.trim();
        if query.is_empty() {
            return vec![];
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Atom::new(
            query,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut buf = Vec::new();
        let mut scored: Vec<(usize, u16)> = apps
            .iter()
            .enumerate()
            .filter_map(|(i, app)| {
                let haystack = Utf32Str::new(app.name.as_str(), &mut buf);
                pattern.score(haystack, &mut matcher).map(|score| (i, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.truncate(max);

        scored
            .iter()
            .map(|(i, _score)| {
                let app = &apps[*i];
                SearchResult {
                    category: "Apps".to_string(),
                    title: app.name.clone(),
                    subtitle: app.path.clone(),
                    action: ResultAction::LaunchApp {
                        path: app.path.clone(),
                    },
                    icon: "app".to_string(),
                }
            })
            .collect()
    }
}
