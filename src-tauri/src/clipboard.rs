use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};

static LAST_CLIP: OnceLock<Mutex<String>> = OnceLock::new();

fn get_last_clip() -> &'static Mutex<String> {
    LAST_CLIP.get_or_init(|| Mutex::new(String::new()))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipboardEntry {
    pub id: u32,
    pub content: String,
    pub preview: String,
    pub timestamp: String,
    pub pinned: bool,
}

/// Read the current clipboard text using Win32 API.
fn read_clipboard_text() -> Option<String> {
    unsafe {
        // CF_UNICODETEXT = 13
        if OpenClipboard(None).is_ok() {
            let handle = GetClipboardData(13);
            let result = if let Ok(h) = handle {
                let hglobal = HGLOBAL(h.0);
                let ptr = GlobalLock(hglobal);
                if !ptr.is_null() {
                    let wide = ptr as *const u16;
                    // Find null terminator
                    let mut len = 0;
                    while *wide.add(len) != 0 {
                        len += 1;
                    }
                    let slice = std::slice::from_raw_parts(wide, len);
                    let text = String::from_utf16_lossy(slice);
                    let _ = GlobalUnlock(hglobal);
                    Some(text)
                } else {
                    None
                }
            } else {
                None
            };
            let _ = CloseClipboard();
            result
        } else {
            None
        }
    }
}

/// Start the clipboard monitoring thread.
pub fn start_monitor() {
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_millis(500));

            if let Some(text) = read_clipboard_text() {
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    let mut last = get_last_clip().lock().unwrap();
                    if *last != trimmed {
                        *last = trimmed.clone();
                        store_clip(&trimmed);
                    }
                }
            }
        }
    });
}

fn db_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Omni")
        .join("usage.db")
}

fn with_db<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&Connection) -> Option<T>,
{
    let path = db_path();
    let conn = Connection::open(&path).ok()?;
    ensure_schema(&conn);
    f(&conn)
}

fn ensure_schema(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS clipboard (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            preview TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            pinned INTEGER NOT NULL DEFAULT 0
        );",
    )
    .ok();
}

fn store_clip(text: &str) {
    with_db(|conn| {
        let preview: String = text.chars().take(100).collect();

        // Don't store duplicates — just update timestamp
        conn.execute(
            "INSERT INTO clipboard (content, preview)
             SELECT ?1, ?2
             WHERE NOT EXISTS (SELECT 1 FROM clipboard WHERE content = ?1)",
            rusqlite::params![text, preview],
        )
        .ok();

        // Update timestamp if it was a duplicate
        conn.execute(
            "UPDATE clipboard SET timestamp = datetime('now') WHERE content = ?1",
            rusqlite::params![text],
        )
        .ok();

        // Keep only last 100 entries (non-pinned get pruned first)
        conn.execute(
            "DELETE FROM clipboard WHERE id NOT IN (
                SELECT id FROM clipboard ORDER BY pinned DESC, timestamp DESC LIMIT 100
            )",
            [],
        )
        .ok();

        Some(())
    });
}

/// Internal function for use from search.rs (no Tauri command wrapper).
pub fn get_clipboard_history_internal(query: &str, limit: u32) -> Vec<ClipboardEntry> {
    with_db(|conn| {
        let query = query.trim();
        if query.is_empty() {
            let mut stmt = conn
                .prepare(
                    "SELECT id, content, preview, timestamp, pinned FROM clipboard
                     ORDER BY pinned DESC, timestamp DESC
                     LIMIT ?1",
                )
                .ok()?;
            let results: Vec<ClipboardEntry> = stmt
                .query_map([limit], |row| {
                    Ok(ClipboardEntry {
                        id: row.get(0)?,
                        content: row.get(1)?,
                        preview: row.get(2)?,
                        timestamp: row.get(3)?,
                        pinned: row.get::<_, i32>(4)? != 0,
                    })
                })
                .ok()?
                .filter_map(|r| r.ok())
                .collect();
            Some(results)
        } else {
            let pattern = format!("%{}%", query);
            let mut stmt = conn
                .prepare(
                    "SELECT id, content, preview, timestamp, pinned FROM clipboard
                     WHERE content LIKE ?1
                     ORDER BY pinned DESC, timestamp DESC
                     LIMIT ?2",
                )
                .ok()?;
            let results: Vec<ClipboardEntry> = stmt
                .query_map(rusqlite::params![pattern, limit], |row| {
                    Ok(ClipboardEntry {
                        id: row.get(0)?,
                        content: row.get(1)?,
                        preview: row.get(2)?,
                        timestamp: row.get(3)?,
                        pinned: row.get::<_, i32>(4)? != 0,
                    })
                })
                .ok()?
                .filter_map(|r| r.ok())
                .collect();
            Some(results)
        }
    })
    .unwrap_or_default()
}

#[tauri::command]
pub fn get_clipboard_history(query: String, limit: u32) -> Vec<ClipboardEntry> {
    get_clipboard_history_internal(&query, limit)
}

#[tauri::command]
pub fn delete_clipboard_entry(id: u32) -> Result<(), String> {
    with_db(|conn| {
        conn.execute("DELETE FROM clipboard WHERE id = ?1", [id])
            .ok();
        Some(())
    })
    .ok_or_else(|| "Failed to delete clipboard entry".to_string())
}

#[tauri::command]
pub fn pin_clipboard_entry(id: u32) -> Result<(), String> {
    with_db(|conn| {
        conn.execute(
            "UPDATE clipboard SET pinned = CASE WHEN pinned = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            [id],
        )
        .ok();
        Some(())
    })
    .ok_or_else(|| "Failed to pin clipboard entry".to_string())
}

#[tauri::command]
pub fn clear_clipboard_history() -> Result<(), String> {
    with_db(|conn| {
        conn.execute("DELETE FROM clipboard WHERE pinned = 0", [])
            .ok();
        Some(())
    })
    .ok_or_else(|| "Failed to clear clipboard history".to_string())
}
