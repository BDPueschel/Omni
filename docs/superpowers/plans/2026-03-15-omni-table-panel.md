# Omni Table Panel — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an Everything-style sortable column view that opens as a side panel (Ctrl+T) next to the existing result list.

**Architecture:** Backend adds optional `size`/`date_modified` fields to `SearchResult` and a new `search_table` Tauri command that queries Everything with metadata columns. Frontend adds a `TablePanel.tsx` component, new state in `App.tsx` for table mode, window resize logic to widen/restore, and focus management between panels.

**Tech Stack:** Rust (Tauri v2 backend), Preact (frontend), Everything HTTP API, CSS

**Spec:** `docs/superpowers/specs/2026-03-15-omni-table-panel-design.md`

---

## Chunk 1: Backend — SearchResult metadata + search_table command

### Task 1: Add optional metadata fields to SearchResult

**Files:**
- Modify: `src-tauri/src/providers/mod.rs:14-21`
- Modify: `src-tauri/tests/everything_test.rs`

- [ ] **Step 1: Update SearchResult struct**

In `src-tauri/src/providers/mod.rs`, add the two optional fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub category: String,
    pub title: String,
    pub subtitle: String,
    pub action: ResultAction,
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_modified: Option<u64>,
}
```

- [ ] **Step 2: Fix all existing SearchResult constructions**

Every place that constructs a `SearchResult` now needs `size: None, date_modified: None`. These files all construct `SearchResult` literals:

- `src-tauri/src/providers/everything.rs` — `search_all()`, `search_apps()`, `format_http_file_results()`, `format_http_dir_results()`, `format_file_results()`, `format_app_results()`, `format_dir_results()`, `unavailable_result()`
- `src-tauri/src/providers/math.rs`
- `src-tauri/src/providers/system.rs`
- `src-tauri/src/providers/units.rs`
- `src-tauri/src/providers/currency.rs`
- `src-tauri/src/providers/url.rs`
- `src-tauri/src/providers/web_search.rs`
- `src-tauri/src/providers/color.rs`
- `src-tauri/src/providers/process.rs`
- `src-tauri/src/search.rs` — `search_query()` (clipboard), `get_frequent_items()`

Add `size: None, date_modified: None` to every `SearchResult { ... }` literal in these files.

- [ ] **Step 3: Build to verify no compile errors**

Run: `cd src-tauri && cargo build 2>&1 | tail -5`
Expected: compiles successfully (warnings OK)

- [ ] **Step 4: Run existing tests to verify nothing broke**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: all existing tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/providers/mod.rs src-tauri/src/providers/*.rs src-tauri/src/search.rs
git commit -m "feat: add optional size/date_modified fields to SearchResult"
```

---

### Task 2: Update Everything HTTP API to return metadata

**Files:**
- Modify: `src-tauri/src/providers/everything.rs:16-33` (HTTP types)
- Modify: `src-tauri/src/providers/everything.rs:101-153` (`query_http`)
- Test: `src-tauri/tests/everything_test.rs`

- [ ] **Step 1: Write test for FILETIME-to-epoch conversion**

Add to `src-tauri/tests/everything_test.rs`:

```rust
#[test]
fn test_filetime_to_unix_epoch() {
    use omni_lib::providers::everything::filetime_to_unix;
    // 2024-01-01 00:00:00 UTC = FILETIME 133481856000000000
    let ft: u64 = 133481856000000000;
    let epoch = filetime_to_unix(ft);
    assert_eq!(epoch, 1704067200);
}

#[test]
fn test_filetime_to_unix_epoch_zero() {
    use omni_lib::providers::everything::filetime_to_unix;
    // FILETIME before Unix epoch should clamp to 0
    let epoch = filetime_to_unix(0);
    assert_eq!(epoch, 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test filetime_to_unix 2>&1 | tail -10`
Expected: FAIL — `filetime_to_unix` not found

- [ ] **Step 3: Implement filetime_to_unix and update HTTP types**

In `src-tauri/src/providers/everything.rs`:

Add the public conversion function (above the `EverythingProvider` impl):

```rust
/// Convert Windows FILETIME (100-ns intervals since 1601-01-01) to Unix epoch seconds.
pub fn filetime_to_unix(ft: u64) -> u64 {
    // Difference between 1601-01-01 and 1970-01-01 in 100-ns intervals
    const EPOCH_DIFF: u64 = 116444736000000000;
    if ft <= EPOCH_DIFF {
        return 0;
    }
    (ft - EPOCH_DIFF) / 10_000_000
}
```

Update `EverythingHttpResult` to include optional metadata:

```rust
#[derive(Debug, serde::Deserialize)]
struct EverythingHttpResult {
    #[serde(rename = "type")]
    result_type: String,
    name: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    date_modified: Option<u64>,
}
```

Update `query_http()` — add `&size_column=1&date_modified_column=1` to the URL format string:

```rust
let url = format!(
    "/?s={}&c={}&j=1&path_column=1&size_column=1&date_modified_column=1&sort={}&ascending={}",
    encoded_query, max_results, sort, if ascending { 1 } else { 0 }
);
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test filetime_to_unix 2>&1 | tail -10`
Expected: both tests PASS

- [ ] **Step 5: Populate metadata in format functions**

Update `format_http_file_results()` and `format_http_dir_results()` in `everything.rs` to pass through the metadata:

In `format_http_file_results()`, change the `SearchResult` construction to:
```rust
SearchResult {
    category: "Files".to_string(),
    title: filename,
    subtitle: full_path.clone(),
    action: ResultAction::OpenFile { path: full_path },
    icon: "file".to_string(),
    size: r.size,
    date_modified: r.date_modified.map(filetime_to_unix),
}
```

Same pattern for `format_http_dir_results()` (with `category: "Directories"`, `icon: "folder"`).

Also update `search_all()` where it builds `SearchResult` inline (lines 356-364 for dirs, 371-379 for files) — same pattern: add `size: r.size, date_modified: r.date_modified.map(filetime_to_unix)`.

And update `search_apps()` (line 458-468) — apps don't need metadata: `size: None, date_modified: None`.

- [ ] **Step 6: Build and run all tests**

Run: `cd src-tauri && cargo build && cargo test 2>&1 | tail -20`
Expected: compiles, all tests pass

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/providers/everything.rs src-tauri/tests/everything_test.rs
git commit -m "feat: populate size/date_modified from Everything HTTP API"
```

---

### Task 3: Add search_table Tauri command

**Files:**
- Modify: `src-tauri/src/search.rs`
- Modify: `src-tauri/src/lib.rs:122-153` (invoke_handler)
- Test: `src-tauri/tests/search_test.rs`

- [ ] **Step 1: Write test for search_table_query**

Add to `src-tauri/tests/search_test.rs`:

```rust
#[test]
#[ignore] // Requires Everything 1.5a to be running
fn test_search_table_query_returns_only_files_and_dirs() {
    // search_table_query should return only Files and Directories categories
    let results = omni_lib::search::search_table_query("test", 100, "date_modified", false);
    for r in &results {
        assert!(
            r.category == "Files" || r.category == "Directories",
            "Unexpected category: {}",
            r.category
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test search_table_query 2>&1 | tail -10`
Expected: FAIL — `search_table_query` not found

- [ ] **Step 3: Implement search_table_query and search_table command**

In `src-tauri/src/search.rs`, add below the existing `search` command:

```rust
/// Table panel search — files and directories only, with metadata, sortable.
pub fn search_table_query(
    query: &str,
    max: usize,
    sort_by: &str,
    ascending: bool,
) -> Vec<SearchResult> {
    let query = query.trim();
    if query.is_empty() {
        return vec![];
    }

    // Map frontend sort names to Everything HTTP API sort values.
    // Everything 1.5a HTTP API accepts these sort parameter names.
    // If sorting doesn't work for a column, verify against the Everything HTTP API docs
    // at https://www.voidtools.com/support/everything/http/ and update the mapping.
    let sort = match sort_by {
        "name" => "name",
        "path" => "path",
        "size" => "size",
        "date_modified" => "date_modified",
        _ => "date_modified",
    };

    let http_query = EverythingProvider::build_http_query(query);
    match EverythingProvider::query_http_public(&http_query, max, sort, ascending) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("search_table HTTP error: {}", e);
            // Fallback: use regular search, filter to files/dirs
            let (files, dirs) = EverythingProvider::search_all(query, max / 2);
            let mut combined = files;
            combined.extend(dirs);
            combined
        }
    }
}

#[tauri::command]
pub fn search_table(
    query: &str,
    sort_by: &str,
    ascending: bool,
) -> Vec<SearchResult> {
    search_table_query(query, 100, sort_by, ascending)
}
```

- [ ] **Step 4: Add query_http_public to EverythingProvider**

In `src-tauri/src/providers/everything.rs`, add a public wrapper that returns `Vec<SearchResult>` (since `query_http` returns raw `EverythingResponse`):

```rust
/// Public entry point for table panel — returns formatted file/dir results with metadata.
pub fn query_http_public(
    query: &str,
    max_results: usize,
    sort: &str,
    ascending: bool,
) -> Result<Vec<SearchResult>, String> {
    let response = Self::query_http(query, max_results, sort, ascending)?;
    let mut results = Vec::new();
    for r in response.results {
        let full_path = if r.path.is_empty() {
            r.name.clone()
        } else {
            format!("{}\\{}", r.path, r.name)
        };
        let filename = Path::new(&full_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if r.result_type == "folder" {
            results.push(SearchResult {
                category: "Directories".to_string(),
                title: filename,
                subtitle: full_path.clone(),
                action: ResultAction::OpenFile { path: full_path },
                icon: "folder".to_string(),
                size: r.size,
                date_modified: r.date_modified.map(filetime_to_unix),
            });
        } else {
            results.push(SearchResult {
                category: "Files".to_string(),
                title: filename,
                subtitle: full_path.clone(),
                action: ResultAction::OpenFile { path: full_path },
                icon: "file".to_string(),
                size: r.size,
                date_modified: r.date_modified.map(filetime_to_unix),
            });
        }
    }
    Ok(results)
}
```

Also make `build_http_query` public (change `fn build_http_query` to `pub fn build_http_query`).

- [ ] **Step 5: Register the command in lib.rs**

In `src-tauri/src/lib.rs`, add `search::search_table` to the `invoke_handler` list (after `search::search`).

- [ ] **Step 6: Build and run tests**

Run: `cd src-tauri && cargo build && cargo test 2>&1 | tail -20`
Expected: compiles, all tests pass (the new test may need `#[ignore]` if Everything isn't running — add the ignore attribute with a comment)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/search.rs src-tauri/src/providers/everything.rs src-tauri/src/lib.rs src-tauri/tests/search_test.rs
git commit -m "feat: add search_table command for table panel"
```

---

## Chunk 2: Frontend — TablePanel component + styling

### Task 4: Create TablePanel component

**Files:**
- Create: `src/components/TablePanel.tsx`

- [ ] **Step 1: Create TablePanel.tsx**

```tsx
import { useState } from "preact/hooks";

interface TableResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
  size?: number;
  date_modified?: number;
}

interface Props {
  results: TableResult[];
  selectedIndex: number;
  multiSelected: Set<number>;
  onSelect: (index: number) => void;
  onExecute: (index: number) => void;
  onSortChange: (column: string, ascending: boolean) => void;
}

type SortColumn = "name" | "path" | "size" | "date_modified";

export function TablePanel({ results, selectedIndex, multiSelected, onSelect, onExecute, onSortChange }: Props) {
  const [sortColumn, setSortColumn] = useState<SortColumn>("date_modified");
  const [sortAscending, setSortAscending] = useState(false);

  const handleHeaderClick = (col: SortColumn) => {
    if (col === sortColumn) {
      const newDir = !sortAscending;
      setSortAscending(newDir);
      onSortChange(col, newDir);
    } else {
      setSortColumn(col);
      setSortAscending(col === "name" || col === "path"); // alpha defaults asc, others desc
      onSortChange(col, col === "name" || col === "path");
    }
  };

  const sortIndicator = (col: SortColumn) => {
    if (col !== sortColumn) return null;
    return <span class="sort-arrow">{sortAscending ? "\u25B2" : "\u25BC"}</span>;
  };

  return (
    <div class="table-panel">
      <div class="table-header">
        <div class="table-col col-name" onClick={() => handleHeaderClick("name")}>
          Name {sortIndicator("name")}
        </div>
        <div class="table-col col-path" onClick={() => handleHeaderClick("path")}>
          Path {sortIndicator("path")}
        </div>
        <div class="table-col col-size" onClick={() => handleHeaderClick("size")}>
          Size {sortIndicator("size")}
        </div>
        <div class="table-col col-date" onClick={() => handleHeaderClick("date_modified")}>
          Modified {sortIndicator("date_modified")}
        </div>
      </div>
      <div class="table-body">
        {results.length === 0 ? (
          <div class="table-empty">No file results</div>
        ) : (
          results.map((r, i) => (
            <div
              key={`${r.subtitle}-${i}`}
              class={`table-row ${i === selectedIndex ? "selected" : ""} ${multiSelected.has(i) ? "multi-selected" : ""}`}
              onClick={() => onSelect(i)}
              onDblClick={() => onExecute(i)}
            >
              <div class="table-col col-name">
                <span class="table-icon">{getFileIcon(r.title)}</span>
                {r.title}
              </div>
              <div class="table-col col-path">{getParentPath(r.subtitle)}</div>
              <div class="table-col col-size">{formatSize(r.size)}</div>
              <div class="table-col col-date">{formatDate(r.date_modified)}</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function getFileIcon(filename: string): string {
  const ext = filename.split(".").pop()?.toLowerCase() || "";
  const iconMap: Record<string, string> = {
    exe: "\u2B21", pdf: "PDF", txt: "TXT", md: "MD",
    jpg: "IMG", jpeg: "IMG", png: "IMG", gif: "GIF", svg: "SVG",
    mp3: "\u266A", wav: "\u266A", mp4: "\u25B6",
    zip: "ZIP", rar: "RAR", "7z": "7Z",
    py: "PY", rs: "RS", js: "JS", ts: "TS",
    json: "{}", html: "<>", css: "#",
  };
  return iconMap[ext] || "\u25CF";
}

function getParentPath(fullPath: string): string {
  const lastSep = fullPath.lastIndexOf("\\");
  return lastSep > 0 ? fullPath.substring(0, lastSep) : fullPath;
}

function formatSize(bytes?: number): string {
  if (bytes == null) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatDate(epoch?: number): string {
  if (epoch == null) return "—";
  const date = new Date(epoch * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays === 1) return "yesterday";
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toISOString().slice(0, 10); // YYYY-MM-DD
}

export type { TableResult };
```

- [ ] **Step 2: Verify it compiles**

Run: `cd "C:/Users/Brian/Code Projects/Omni" && npx tsc --noEmit 2>&1 | tail -10`
Expected: no type errors (or only pre-existing ones)

- [ ] **Step 3: Commit**

```bash
git add src/components/TablePanel.tsx
git commit -m "feat: add TablePanel component with sortable columns"
```

---

### Task 5: Add table panel CSS

**Files:**
- Modify: `src/styles.css` (append after the help overlay styles, before EOF)

- [ ] **Step 1: Add table panel styles**

Append to end of `src/styles.css`:

```css
/* Table panel (Ctrl+T) */
.omni-split {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.omni-split > .results-container {
  width: 600px;
  flex-shrink: 0;
}

.table-divider {
  width: 1px;
  background: rgba(255, 255, 255, 0.08);
  flex-shrink: 0;
}

.panel-active {
  border-top: 2px solid rgba(130, 180, 255, 0.5);
}

.panel-inactive {
  border-top: 2px solid transparent;
  opacity: 0.7;
}

.table-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.table-header {
  display: flex;
  padding: 6px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  color: rgba(255, 255, 255, 0.4);
  flex-shrink: 0;
  cursor: pointer;
  user-select: none;
}

.table-header .table-col:hover {
  color: rgba(255, 255, 255, 0.7);
}

.sort-arrow {
  font-size: 9px;
  margin-left: 4px;
  color: rgba(130, 180, 255, 0.7);
}

.table-body {
  flex: 1;
  overflow-y: auto;
  overflow-y: overlay;
}

.table-body::-webkit-scrollbar {
  width: 0;
  background: transparent;
}

.table-row {
  display: flex;
  padding: 5px 8px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: background 0.1s;
}

.table-row:hover,
.table-row.selected {
  background: rgba(255, 255, 255, 0.08);
}

.table-row.multi-selected {
  background: rgba(130, 180, 255, 0.08);
  border-left: 2px solid rgba(130, 180, 255, 0.5);
}

.table-row.multi-selected.selected {
  background: rgba(130, 180, 255, 0.15);
}

.table-col {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding: 0 4px;
}

.col-name {
  width: 35%;
  flex-shrink: 0;
}

.col-path {
  width: 35%;
  flex-shrink: 0;
  color: rgba(255, 255, 255, 0.35);
}

.col-size {
  width: 12%;
  flex-shrink: 0;
  text-align: right;
  color: rgba(255, 255, 255, 0.4);
}

.col-date {
  width: 18%;
  flex-shrink: 0;
  text-align: right;
  color: rgba(255, 255, 255, 0.4);
}

.table-icon {
  font-size: 9px;
  font-weight: 700;
  margin-right: 6px;
  color: rgba(255, 255, 255, 0.4);
}

.table-empty {
  text-align: center;
  padding: 24px;
  color: rgba(255, 255, 255, 0.3);
  font-size: 13px;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/styles.css
git commit -m "feat: add table panel CSS styles"
```

---

## Chunk 3: Frontend — App.tsx integration, key handling, window resize

### Task 6: Add table state and Ctrl+T toggle to App.tsx

**Files:**
- Modify: `src/App.tsx`

- [ ] **Step 1: Add imports and state**

At the top of `src/App.tsx`, add the import (after the PreviewPanel import):

```tsx
import { TablePanel } from "./components/TablePanel";
import type { TableResult } from "./components/TablePanel";
```

Inside the `App()` function, after the existing state declarations (after `multiSelected` state, around line 59), add:

```tsx
const [tableOpen, setTableOpen] = useState(false);
const [activePanel, setActivePanel] = useState<"results" | "table">("results");
const [tableSelectedIndex, setTableSelectedIndex] = useState(0);
const [tableResults, setTableResults] = useState<TableResult[]>([]);
const [tableMultiSelected, setTableMultiSelected] = useState<Set<number>>(new Set());
const [originalWindowPos, setOriginalWindowPos] = useState<{ x: number; y: number } | null>(null);
```

- [ ] **Step 2: Add table fetch helper**

After the state declarations, add a helper to fetch table results:

```tsx
const fetchTableResults = useCallback(async (q: string, sortBy = "date_modified", asc = false) => {
  if (!q.trim()) return;
  try {
    const res = await invoke<TableResult[]>("search_table", { query: q, sortBy, ascending: asc });
    setTableResults(res);
    setTableSelectedIndex(0);
    setTableMultiSelected(new Set());
  } catch (e) {
    console.error("Table search error:", e);
  }
}, []);
```

- [ ] **Step 3: Add table toggle function**

Add a toggle function:

```tsx
const toggleTable = useCallback(async () => {
  const hasFileResults = flatResults.some(r => r.category === "Files" || r.category === "Directories");
  if (!hasFileResults && !tableOpen) return; // guard: no file results to show

  if (tableOpen) {
    // Close table — restore window
    setTableOpen(false);
    setActivePanel("results");
    setTableMultiSelected(new Set());
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const { LogicalSize } = await import("@tauri-apps/api/dpi");
      const win = getCurrentWindow();
      await win.setSize(new LogicalSize(600, win.innerSize ? (await win.innerSize()).height / (window.devicePixelRatio || 1) : 500));
      if (originalWindowPos) {
        const { LogicalPosition } = await import("@tauri-apps/api/dpi");
        await win.setPosition(new LogicalPosition(originalWindowPos.x, originalWindowPos.y));
        setOriginalWindowPos(null);
      }
    } catch (e) {
      console.error("Table close resize error:", e);
    }
  } else {
    // Open table — widen window
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const { LogicalSize, LogicalPosition } = await import("@tauri-apps/api/dpi");
      const win = getCurrentWindow();
      const pos = await win.outerPosition();
      const scale = window.devicePixelRatio || 1;
      const logicalX = pos.x / scale;
      const logicalY = pos.y / scale;
      setOriginalWindowPos({ x: logicalX, y: logicalY });

      const maxW = window.screen.availWidth * 0.85 / scale;
      const targetW = Math.min(1200, maxW);

      // Shift left if would overflow right edge
      const screenW = window.screen.availWidth / scale;
      let newX = logicalX;
      if (logicalX + targetW > screenW) {
        newX = Math.max(0, screenW - targetW);
        await win.setPosition(new LogicalPosition(newX, logicalY));
      }

      const currentSize = await win.innerSize();
      const currentH = currentSize.height / scale;
      await win.setSize(new LogicalSize(targetW, currentH));
    } catch (e) {
      console.error("Table open resize error:", e);
    }
    setTableOpen(true);
    setActivePanel("table");
    fetchTableResults(query);
  }
}, [tableOpen, flatResults, query, originalWindowPos, fetchTableResults]);
```

- [ ] **Step 4: Commit**

```bash
git add src/App.tsx
git commit -m "feat: add table state, fetch helper, and toggle function"
```

---

### Task 7: Add key handling for table panel in App.tsx

**Files:**
- Modify: `src/App.tsx` (inside `handleKeyDown`)

- [ ] **Step 1: Add table panel key handler block**

In `handleKeyDown`, after the context menu block (after line 321 `}`) and before the `getCategoryBounds` helper (line 323), add a new block for table panel navigation:

```tsx
// Table panel is focused — handle its navigation
if (tableOpen && activePanel === "table") {
  switch (e.key) {
    case "ArrowDown":
      e.preventDefault();
      if (e.shiftKey) {
        setTableMultiSelected(prev => new Set([...prev, tableSelectedIndex]));
      }
      setTableSelectedIndex(i => Math.min(i + 1, tableResults.length - 1));
      return;
    case "ArrowUp":
      e.preventDefault();
      if (e.shiftKey) {
        setTableMultiSelected(prev => new Set([...prev, tableSelectedIndex]));
      }
      setTableSelectedIndex(i => Math.max(i - 1, 0));
      return;
    case "Home":
      e.preventDefault();
      setTableSelectedIndex(0);
      return;
    case "End":
      e.preventDefault();
      setTableSelectedIndex(tableResults.length - 1);
      return;
    case "Enter":
      e.preventDefault();
      if (tableMultiSelected.size > 0) {
        const indices = new Set([...tableMultiSelected, tableSelectedIndex]);
        const paths = [...indices].map(i => tableResults[i]?.subtitle).filter(Boolean);
        invoke("batch_open", { paths });
        setTableMultiSelected(new Set());
      } else {
        const tr = tableResults[tableSelectedIndex];
        if (tr) {
          invoke("execute_action", { action: tr.action });
          invoke("record_selection", { query, resultPath: tr.subtitle, category: tr.category, title: tr.title });
        }
      }
      return;
    case "ArrowRight":
      if (e.shiftKey && tableResults.length > 0) {
        e.preventDefault();
        // Open context menu for table result — reuse existing context menu by setting contextMenuIndex
        // We map table selection to the matching flatResults index
        const tr = tableResults[tableSelectedIndex];
        if (tr) {
          const flatIdx = flatResults.findIndex(r => r.subtitle === tr.subtitle);
          if (flatIdx >= 0) {
            setContextMenuIndex(flatIdx);
            setContextActionIndex(0);
          }
        }
      }
      return;
    case " ":
      if (e.ctrlKey) {
        e.preventDefault();
        if (previewData) {
          setPreviewData(null);
        } else {
          const tr = tableResults[tableSelectedIndex];
          if (tr) {
            invoke<FilePreview>("preview_file", { path: tr.subtitle })
              .then(preview => setPreviewData(preview))
              .catch(err => console.error("Preview error:", err));
          }
        }
      }
      return;
    case "Escape":
      e.preventDefault();
      if (tableMultiSelected.size > 0) {
        setTableMultiSelected(new Set());
      } else {
        toggleTable();
      }
      return;
    case "1": case "2": case "3": case "4":
      if (e.ctrlKey) {
        e.preventDefault();
        const cols: Array<"name" | "path" | "size" | "date_modified"> = ["name", "path", "size", "date_modified"];
        const col = cols[parseInt(e.key) - 1];
        // Trigger sort via the TablePanel's onSortChange
        // We'll re-fetch directly here
        fetchTableResults(query, col, col === "name" || col === "path");
      }
      return;
    default:
      // Let typing fall through to the search input
      return;
  }
}
```

- [ ] **Step 2: Add Ctrl+T and Ctrl+Tab handlers**

In the main `switch (e.key)` block (the "Normal result navigation" section), add these cases. Add before the `"Escape"` case:

```tsx
case "t":
case "T":
  if (e.ctrlKey) {
    e.preventDefault();
    toggleTable();
  }
  break;
case "Tab":
  // Existing Tab handler...
  // MODIFY: Add Ctrl+Tab check at the top of the Tab case
```

Also modify the existing `case "e"` / `case "E"` Ctrl+E handler. Add a guard at the top to suppress expanding Files/Directories when the table is open:

```tsx
case "e":
case "E":
  if (e.ctrlKey && flatResults.length > 0) {
    e.preventDefault();
    // When table is open, suppress expanding Files/Directories (table already shows them)
    if (tableOpen) {
      const cat = getSelectedCategory();
      if (cat === "Files" || cat === "Directories") break;
    }
    expandCategory();
  }
  break;
```

For Ctrl+Tab, modify the existing `case "Tab":` block. At the very beginning of the Tab case (line 459), before `e.preventDefault()`, add:

```tsx
if (e.ctrlKey && tableOpen) {
  e.preventDefault();
  setActivePanel(p => p === "results" ? "table" : "results");
  // Clear multi-select on panel switch
  if (activePanel === "results") {
    setMultiSelected(new Set());
  } else {
    setTableMultiSelected(new Set());
  }
  break;
}
```

- [ ] **Step 3: Update handleInput to refresh table on query change**

In the `handleInput` callback, after the debounced search call (inside the `setTimeout`, after `setResults(res)` around line 95), add:

```tsx
if (tableOpen) {
  fetchTableResults(value);
}
```

Also, in the `if (!value.trim())` early-return block (around line 84), add before the `return`:

```tsx
if (tableOpen) {
  // Auto-close table on empty query — must call toggleTable to restore window size/position
  toggleTable();
}
```

- [ ] **Step 4: Update the dependency arrays**

Update the `handleKeyDown` dependency array (line 555) to include the new state:
Add `tableOpen, activePanel, tableSelectedIndex, tableResults, tableMultiSelected, toggleTable, fetchTableResults` to the array.

Update the `handleInput` dependency array to include `tableOpen, fetchTableResults, toggleTable`.

- [ ] **Step 5: Commit**

```bash
git add src/App.tsx
git commit -m "feat: add table key handling, Ctrl+T toggle, Ctrl+Tab panel cycling"
```

---

### Task 8: Update App.tsx layout and window resize

**Files:**
- Modify: `src/App.tsx` (JSX return and resize effect)

- [ ] **Step 1: Update the JSX layout**

Replace the main JSX return section. The key change: when `tableOpen` is true, wrap the results-container and table panel in an `omni-split` flex container.

In the return JSX, replace the section from `{previewData ? (` through to `{showHelp && (` with:

```tsx
{tableOpen ? (
  <div class="omni-split">
    <div class={`results-container ${activePanel === "results" ? "panel-active" : "panel-inactive"}`}>
      {grouped.map((group) => {
        const startIndex = globalIndex;
        globalIndex += group.results.length;
        return (
          <ResultGroup
            key={group.category}
            category={group.category}
            results={group.results}
            selectedIndex={activePanel === "results" ? selectedIndex : -1}
            globalStartIndex={startIndex}
            onExecute={executeResult}
            isActive={activePanel === "results" && group.category === activeCategory}
            isExpanded={group.category === expandedCategory}
            multiSelected={activePanel === "results" ? multiSelected : new Set()}
          />
        );
      })}
      {copiedFlash && <div class="copied-flash">Copied!</div>}
    </div>
    <div class="table-divider" />
    <div class={activePanel === "table" ? "panel-active" : "panel-inactive"} style={{ display: "flex", flex: 1, minWidth: 0 }}>
      {previewData ? (
        <div class="results-container" style={{ flex: 1 }}>
          <PreviewPanel preview={previewData} />
        </div>
      ) : (
        <TablePanel
          results={tableResults}
          selectedIndex={tableSelectedIndex}
          multiSelected={tableMultiSelected}
          onSelect={(i) => { setTableSelectedIndex(i); setActivePanel("table"); }}
          onExecute={(i) => {
            const tr = tableResults[i];
            if (tr) {
              invoke("execute_action", { action: tr.action });
              invoke("record_selection", { query, resultPath: tr.subtitle, category: tr.category, title: tr.title });
            }
          }}
          onSortChange={(col, asc) => fetchTableResults(query, col, asc)}
        />
      )}
    </div>
  </div>
) : previewData ? (
  <div class="results-container">
    <PreviewPanel preview={previewData} />
  </div>
) : contextMenuIndex !== null && contextMenuIndex === -1 ? (
  /* ... existing batch context menu JSX — keep unchanged ... */
```

Keep all the remaining conditional branches (batch context menu, single context menu, normal results, empty state) exactly as they are now.

- [ ] **Step 2: Update the window resize useEffect**

In the resize `useEffect` (around line 584), add a guard: if `tableOpen` is true, skip the resize logic (the table toggle function handles its own sizing):

At the top of the async function inside the effect, add:

```tsx
if (tableOpen) return; // Table toggle manages its own window size
```

Add `tableOpen` to the effect dependency array.

- [ ] **Step 3: Update the status bar**

Update the status bar section (around line 752) to show table selection when appropriate:

```tsx
{flatResults.length > 0 && !showHelp && !previewData && contextMenuIndex === null && (
  <div class="status-bar">
    {tableOpen && activePanel === "table"
      ? (tableMultiSelected.size > 0
          ? `${tableMultiSelected.size + 1} items selected \u00b7 Shift+\u2192 for batch actions`
          : tableResults[tableSelectedIndex]?.subtitle || "")
      : multiSelected.size > 0
        ? `${multiSelected.size + 1} items selected \u00b7 Shift+\u2192 for batch actions`
        : flatResults[selectedIndex]?.subtitle || ""}
  </div>
)}
```

- [ ] **Step 4: Update help overlay**

Add Ctrl+T to the help overlay keyboard shortcuts (in the Actions help-section):

```tsx
<div class="help-row"><kbd>Ctrl+T</kbd><span>Table view (file columns)</span></div>
```

- [ ] **Step 5: Update clear-query listener**

In the `listen("clear-query", ...)` callback (around line 620), also reset table state:

```tsx
setTableOpen(false);
setActivePanel("results");
setTableResults([]);
setTableMultiSelected(new Set());
```

- [ ] **Step 6: Verify the app compiles**

Run: `cd "C:/Users/Brian/Code Projects/Omni" && npx tsc --noEmit 2>&1 | tail -10`
Expected: no type errors

- [ ] **Step 7: Commit**

```bash
git add src/App.tsx
git commit -m "feat: integrate table panel into App layout with split view"
```

---

### Task 9: Update SearchResult interface in frontend

**Files:**
- Modify: `src/App.tsx:10-16` (SearchResult interface)

- [ ] **Step 1: Add optional fields to the frontend SearchResult interface**

In `src/App.tsx`, update the `SearchResult` interface:

```tsx
interface SearchResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
  size?: number;
  date_modified?: number;
}
```

This matches the backend's serialization (optional fields are omitted from JSON when `None`).

- [ ] **Step 2: Commit**

```bash
git add src/App.tsx
git commit -m "feat: add size/date_modified to frontend SearchResult interface"
```

---

## Chunk 4: Integration testing and polish

### Task 10: Manual integration test

- [ ] **Step 1: Build and run dev mode**

Kill any running Omni instance first:
```bash
taskkill //IM omni.exe //F 2>/dev/null
```

Then:
```bash
cd "C:/Users/Brian/Code Projects/Omni" && npm run tauri dev
```

- [ ] **Step 2: Test checklist**

Verify each of these manually:

1. Type a search query that returns file results (e.g., "cargo")
2. Press Ctrl+T — window should widen, table panel appears on right with columns
3. Arrow keys navigate table rows, selected row is highlighted
4. Click column headers — results re-sort, arrow indicator updates
5. Ctrl+1/2/3/4 sorts by column while table is focused
6. Ctrl+Tab switches focus between result list and table (visual indicator changes)
7. Press Enter on a table row — file opens
8. Press Ctrl+Space on a table row — preview appears in right panel
9. Press Ctrl+Space again — preview closes, table returns
10. Press Escape — table closes, window restores to original size
11. Press Ctrl+T again — table reopens
12. Clear the search query — table auto-closes
13. Type a query with no file results (e.g., "2+2") — Ctrl+T does nothing
14. Shift+Up/Down in table for multi-select
15. Status bar updates based on which panel is active

- [ ] **Step 3: Fix any issues found**

Address any bugs discovered during manual testing.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "fix: polish table panel integration"
```
