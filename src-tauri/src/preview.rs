use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub file_type: String,
    pub content: String,
    pub filename: String,
    pub size: String,
    pub modified: String,
    pub extension: String,
}

const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "rs", "py", "js", "ts", "jsx", "tsx", "json", "toml", "yaml", "yml", "csv",
    "log", "xml", "html", "css", "sh", "bat", "ps1", "cfg", "ini", "env", "gitignore",
    "dockerfile", "makefile", "c", "cpp", "h", "go", "java", "rb", "php", "sql", "r",
];

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "bmp", "webp", "ico",
];

const MAX_IMAGE_SIZE: u64 = 5 * 1024 * 1024; // 5 MB
const MAX_TEXT_LINES: usize = 100;

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_modified(modified: std::time::SystemTime) -> String {
    let duration = modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs() as i64;

    // Simple date formatting without chrono
    // Calculate from unix timestamp
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    // Calculate year/month/day from days since epoch
    let mut y = 1970i64;
    let mut remaining_days = days;

    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    let mut m = 0usize;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining_days < md {
            m = i;
            break;
        }
        remaining_days -= md;
    }

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        y,
        m + 1,
        remaining_days + 1,
        hours,
        minutes
    )
}

#[tauri::command]
pub fn preview_file(path: String) -> Result<FilePreview, String> {
    let p = Path::new(&path);

    if !p.exists() {
        return Err("File not found".to_string());
    }

    let metadata = fs::metadata(p).map_err(|e| e.to_string())?;
    let size = format_size(metadata.len());
    let modified = metadata
        .modified()
        .map(format_modified)
        .unwrap_or_else(|_| "Unknown".to_string());

    let filename = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let extension = p
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // Also check filename itself for extensionless special files
    let ext_check = if extension.is_empty() {
        filename.to_lowercase()
    } else {
        extension.clone()
    };

    let file_type;
    let content;

    if IMAGE_EXTENSIONS.contains(&ext_check.as_str()) {
        file_type = "image".to_string();
        if metadata.len() > MAX_IMAGE_SIZE {
            content = String::new();
        } else if ext_check == "svg" {
            // Return SVG as text
            content = fs::read_to_string(p).unwrap_or_else(|_| {
                String::from_utf8_lossy(&fs::read(p).unwrap_or_default()).to_string()
            });
        } else {
            let bytes = fs::read(p).map_err(|e| e.to_string())?;
            let mime = match ext_check.as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "bmp" => "image/bmp",
                "webp" => "image/webp",
                "ico" => "image/x-icon",
                _ => "application/octet-stream",
            };
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            content = format!("data:{};base64,{}", mime, b64);
        }
    } else if TEXT_EXTENSIONS.contains(&ext_check.as_str()) {
        file_type = "text".to_string();
        let raw = fs::read(p).map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&raw);
        let lines: Vec<&str> = text.lines().take(MAX_TEXT_LINES).collect();
        content = lines.join("\n");
    } else {
        file_type = "binary".to_string();
        content = String::new();
    }

    Ok(FilePreview {
        file_type,
        content,
        filename,
        size,
        modified,
        extension,
    })
}
