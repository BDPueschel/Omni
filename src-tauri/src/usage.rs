use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

static DB: OnceLock<Mutex<Connection>> = OnceLock::new();

fn db_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Omni")
        .join("usage.db")
}

fn get_db() -> &'static Mutex<Connection> {
    DB.get_or_init(|| {
        let path = db_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path).expect("Failed to open usage database");
        init_schema(&conn);
        Mutex::new(conn)
    })
}

fn init_schema(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS usage (
            query TEXT NOT NULL,
            result_path TEXT NOT NULL,
            category TEXT NOT NULL,
            title TEXT NOT NULL,
            count INTEGER NOT NULL DEFAULT 1,
            last_used TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (query, result_path)
        );
        CREATE INDEX IF NOT EXISTS idx_usage_query ON usage(query);
        CREATE INDEX IF NOT EXISTS idx_usage_count ON usage(count DESC);",
    )
    .expect("Failed to create usage table");
}

/// Record that the user selected a result for a given query.
pub fn record_usage(query: &str, result_path: &str, category: &str, title: &str) {
    record_usage_with(
        &get_db().lock().unwrap(),
        query,
        result_path,
        category,
        title,
    );
}

/// Record usage with a specific connection (for testability).
pub fn record_usage_with(
    conn: &Connection,
    query: &str,
    result_path: &str,
    category: &str,
    title: &str,
) {
    let normalized_query = query.trim().to_lowercase();
    conn.execute(
        "INSERT INTO usage (query, result_path, category, title, count, last_used)
         VALUES (?1, ?2, ?3, ?4, 1, datetime('now'))
         ON CONFLICT(query, result_path) DO UPDATE SET
            count = count + 1,
            last_used = datetime('now'),
            title = ?4",
        rusqlite::params![normalized_query, result_path, category, title],
    )
    .ok();
}

/// Get boosted results for a query — returns Vec<(result_path, category, title, count)>
pub fn get_usage(query: &str) -> Vec<(String, String, String, u32)> {
    get_usage_with(&get_db().lock().unwrap(), query)
}

/// Get usage with a specific connection (for testability).
pub fn get_usage_with(conn: &Connection, query: &str) -> Vec<(String, String, String, u32)> {
    let normalized = query.trim().to_lowercase();
    let mut stmt = conn
        .prepare(
            "SELECT result_path, category, title, count FROM usage
         WHERE query = ?1
         ORDER BY count DESC
         LIMIT 20",
        )
        .unwrap();

    stmt.query_map([&normalized], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, u32>(3)?,
        ))
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

/// Get globally frequent items (across all queries) for showing when search is empty or for boosting.
pub fn get_frequent(limit: usize) -> Vec<(String, String, String, u32)> {
    get_frequent_with(&get_db().lock().unwrap(), limit)
}

/// Get frequent items with a specific connection (for testability).
pub fn get_frequent_with(conn: &Connection, limit: usize) -> Vec<(String, String, String, u32)> {
    let mut stmt = conn
        .prepare(
            "SELECT result_path, category, title, count FROM usage
         WHERE count >= 3
         ORDER BY count DESC, last_used DESC
         LIMIT ?1",
        )
        .unwrap();

    stmt.query_map([limit as u32], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, u32>(3)?,
        ))
    })
    .unwrap()
    .filter_map(|r| r.ok())
    .collect()
}

/// Clear all usage data.
pub fn clear_usage() {
    clear_usage_with(&get_db().lock().unwrap());
}

/// Clear usage with a specific connection (for testability).
pub fn clear_usage_with(conn: &Connection) {
    conn.execute("DELETE FROM usage", []).ok();
}

/// Create an in-memory connection with the usage schema (for tests).
pub fn test_connection() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn);
    conn
}
