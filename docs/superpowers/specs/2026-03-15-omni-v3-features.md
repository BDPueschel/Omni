# Omni v3 Features — Design Spec

Seven features focused on daily-driver usability and performance.

---

## 1. Clipboard History

**The #1 missing feature for power users.**

**Trigger:** Type `clip` or `cb` prefix, or press `Alt+V` dedicated hotkey.

**Architecture:**
- Background thread started on app launch using Win32 `AddClipboardFormatListener` API
- Monitors clipboard changes system-wide, stores entries in SQLite (`%AppData%\Omni\usage.db`, new `clipboard` table)
- Stores last 100 entries with timestamp, content type (text/image/file), and preview
- Text entries: store full content (cap at 10KB per entry)
- Image entries: store as base64 PNG thumbnail (resize to 200px wide for storage)
- File copy entries: store the file path list

**Table schema:**
```sql
CREATE TABLE clipboard (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,  -- "text", "image", "files"
    content TEXT NOT NULL,        -- text content, base64 image, or JSON file list
    preview TEXT NOT NULL,        -- first 100 chars for display
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    pinned INTEGER NOT NULL DEFAULT 0
);
```

**Frontend:**
- New "Clipboard" category in results when query starts with `clip` or `cb`
- Each entry shows: preview text (truncated), timestamp ("2m ago", "1h ago"), content type badge
- Enter pastes the selected entry (write to clipboard + simulate Ctrl+V? or just copy to clipboard)
- Actually: Enter copies the entry back to the active clipboard, then hides Omni. User can paste normally.
- Context menu: "Pin" (stays at top), "Delete", "Copy"

**Rust implementation:**
- `src-tauri/src/clipboard.rs` — clipboard monitoring thread + SQLite storage
- Uses `windows` crate: `AddClipboardFormatListener`, `GetClipboardData`, `OpenClipboard`, `CloseClipboard`
- Thread runs in a hidden message-only window (`HWND_MESSAGE`) to receive `WM_CLIPBOARDUPDATE`
- Tauri commands: `get_clipboard_history(query, limit)`, `paste_clipboard_entry(id)`, `delete_clipboard_entry(id)`, `pin_clipboard_entry(id)`, `clear_clipboard_history()`

**Hotkey:** Register `Alt+V` as a second global shortcut that opens Omni pre-filled with `cb ` prefix.

---

## 2. Switch Everything from es.exe to HTTP API

**Biggest performance win — eliminate process spawn overhead.**

**Current:** 3-4 `es.exe` process spawns per search query (files, dirs, apps).

**New:** Single HTTP request to Everything's built-in HTTP server. JSON response, no process overhead.

**Setup:** Everything 1.5a has a built-in HTTP server. Enable in:
- Tools > Options > HTTP Server > Enable HTTP Server
- Port: 8080 (or configurable in Omni settings)
- No authentication needed for localhost

**API endpoint:**
```
GET http://localhost:8080/?s=<query>&c=<count>&o=<offset>&j=1&sort=date_modified&ascending=0
```

Parameters: `s`=search, `c`=count, `o`=offset, `j=1` for JSON, `sort`=field, `ascending`=0/1, `path_column`=1 for full paths.

**Response format (j=1):**
```json
{
  "totalResults": 1234,
  "results": [
    { "type": "file", "name": "foo.rs", "path": "C:\\Projects", "size": 1234, "date_modified": 132456789 }
  ]
}
```

**Implementation:**
- Replace `run_es()` in `everything.rs` with `query_http()` using `reqwest` or `ureq` (lightweight HTTP client)
- Actually, to avoid adding deps, use `std::net::TcpStream` for raw HTTP GET — the API is simple enough
- Keep `es.exe` as a fallback if HTTP server isn't running
- Single request for all results, split files/dirs by `type` field in response
- App search: add `folder:` or extension filters to the query parameter

**Config:** Add `everything_http_port` to `OmniConfig` (default: 8080).

**Fallback chain:** HTTP API → es.exe → manual scan

---

## 3. Tab Completion for Paths

**Type `C:\Us` then press Tab to auto-complete to `C:\Users\`.**

**Trigger:** Tab key when the query looks like a path (starts with drive letter or `\\`).

**Implementation:**
- In the Tab key handler, check if the query looks like a partial path
- If so, query Everything for directories matching the partial path
- Auto-complete to the first matching directory, appending `\`
- Subsequent Tab presses cycle through matches (like terminal completion)

**Rust:** New Tauri command `complete_path(partial: &str) -> Vec<String>`:
- Use es.exe or HTTP API: search for `<partial>*` with `-ad` (directories only)
- Return top 5 matching directory paths

**Frontend:**
- Override Tab behavior when query matches path pattern
- Store completion candidates in state
- Tab cycles through candidates, Shift+Tab goes backwards
- When a completion is selected, replace the query text with it

---

## 4. Multi-Select

**Hold Shift to select multiple results, then batch-operate.**

**Implementation:**
- New state: `selectedIndices: Set<number>` (multiple selection)
- `Shift+ArrowDown/Up` adds items to the selection set
- `Shift+Click` adds/removes from selection
- `Ctrl+A` selects all visible results
- When multiple items are selected, Enter/context menu operates on all of them

**Batch operations (context menu when multi-selected):**
- "Open all" — opens each file/app
- "Copy all paths" — joins paths with newline, copies to clipboard
- "Copy all to..." — folder picker, copies all files
- "Move all to..." — folder picker, moves all files
- "Delete all (recycle bin)" — with confirmation showing count

**Visual:** Selected items get a persistent highlight (different from the cursor highlight). Show selection count in a footer: "3 items selected".

**Rust:** New Tauri commands:
- `batch_open(paths: Vec<String>)`
- `batch_copy_to(paths: Vec<String>)`
- `batch_move_to(paths: Vec<String>)`
- `batch_delete(paths: Vec<String>)`

---

## 5. Result Count in Category Headers

**Show "FILES (7)" instead of just "FILES".**

**Implementation:** Pure frontend change in `ResultGroup.tsx`:
```tsx
<div class="result-group-header">
    <span>{category} ({results.length})</span>
    {hint && <span class="group-hint">{hint}</span>}
</div>
```

If the category is expanded (Ctrl+E), show the total: "FILES (47)".

---

## 6. Ctrl+C to Copy Path

**When a result is highlighted, Ctrl+C copies its path without opening context menu.**

**Implementation:** In the keyDown handler:
```tsx
case "c":
case "C":
    if (e.ctrlKey && flatResults.length > 0) {
        e.preventDefault();
        const result = flatResults[selectedIndex];
        await navigator.clipboard.writeText(result.subtitle);
        setCopiedFlash(true);
        setTimeout(() => setCopiedFlash(false), 1000);
    }
    break;
```

Update help overlay to document this.

---

## 7. Status Footer Bar

**Show the selected item's full path in a subtle footer, like VS Code's status bar.**

**Implementation:**
- New `<div class="status-bar">` at the bottom of `.omni-container`
- Shows the full path/subtitle of the currently highlighted result
- Truncated with ellipsis from the LEFT (show the end of the path, which is most relevant)
- Also shows: selection count (if multi-select), Everything connection status (green dot / red dot)

**CSS:**
```css
.status-bar {
    padding: 4px 12px;
    font-size: 10px;
    color: rgba(255, 255, 255, 0.25);
    border-top: 1px solid rgba(255, 255, 255, 0.04);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl; /* truncate from left */
    text-align: left;
    flex-shrink: 0;
}
```

**Everything status dot:**
- Check Everything HTTP server on startup (or es.exe availability)
- Green dot: Everything connected
- Red dot: Everything not running
- Shown in the status bar: `● Connected` or `● Disconnected`

---

## Implementation Order

| Phase | Feature | Effort | Dependencies |
|-------|---------|--------|--------------|
| 1 | Result counts in headers (#5) | 5 min | None |
| 2 | Ctrl+C copy path (#6) | 5 min | None |
| 3 | Status footer bar (#7) | 15 min | None |
| 4 | Tab completion (#3) | 30 min | Everything API |
| 5 | Everything HTTP API (#2) | 1-2 hr | Everything HTTP server enabled |
| 6 | Multi-select (#4) | 1-2 hr | None |
| 7 | Clipboard history (#1) | 2-3 hr | Win32 clipboard APIs |

Quick wins first (5, 6, 7), then the meatier features.
