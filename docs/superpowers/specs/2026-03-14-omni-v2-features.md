# Omni v2 Features — Design Spec

Seven new features to close the gap with Raycast while keeping Everything's speed advantage.

## 1. Unit Conversion

**Trigger:** Input matches pattern like `5km in miles`, `100f to c`, `2.5lb in kg`

**Supported categories:**
- Length: mm, cm, m, km, in, ft, yd, mi
- Weight: mg, g, kg, lb, oz, st
- Temperature: c/celsius, f/fahrenheit, k/kelvin
- Volume: ml, l, gal, fl oz, cup, pt, qt
- Speed: mph, km/h, m/s, knots
- Data: b, kb, mb, gb, tb, pb
- Time: ms, s, min, hr, day, week, year

**Detection heuristic:** `<number><unit> (in|to) <unit>` — regex match before other providers run.

**Implementation:** Pure Rust conversion logic (no external API). A `UnitProvider` in `providers/units.rs` with a conversion table. Returns a Math-category result with the converted value.

**Priority:** High — simple to implement, high daily utility.

---

## 2. Currency Conversion

**Trigger:** Input matches `100 usd in eur`, `50 gbp to jpy`, `$100 to €`

**API:** Free exchange rate API — `https://open.er-api.com/v6/latest/USD` (no key required, updates daily). Cache rates in memory on first request, refresh every 6 hours.

**Supported:** All ISO 4217 currency codes + common symbols ($, €, £, ¥).

**Implementation:** `CurrencyProvider` in `providers/currency.rs`. On first query, fetch rates via `reqwest` (or shell out to curl to avoid adding a dep). Cache in `OnceLock<HashMap<String, f64>>`. Parse input with regex for `<amount> <from> (in|to) <from>`.

**Offline behavior:** If no cached rates and network unavailable, show "Currency rates unavailable — check connection."

**Priority:** High — very common use case.

---

## 3. Color Picker / Preview

**Trigger:** Input matches hex color `#ff5733`, RGB `rgb(255,87,51)`, or HSL `hsl(11,100%,60%)`.

**Display:** Show the color as a filled square swatch next to the result. Show conversions to other formats (hex → rgb → hsl). Enter copies the hex value.

**Implementation:**
- Rust: `ColorProvider` in `providers/color.rs` — parse hex/rgb/hsl, convert between formats, return result with a special `color_hex` field.
- Frontend: `ResultItem` checks for `color_hex` field and renders a CSS `background-color` swatch in the icon area instead of the text badge.

**Priority:** Medium — useful for developers, elegant to implement.

---

## 4. Real App Icons

**Approach:** Extract icons from `.exe` and `.lnk` files using Windows Shell API, serve as base64 data URIs.

**Implementation:**
- New Tauri command `get_icon(path: &str) -> String` that:
  1. Uses `windows` crate to call `SHGetFileInfoW` with `SHGFI_ICON | SHGFI_LARGEICON`
  2. Converts HICON to bitmap via `GetIconInfo` + `GetDIBits`
  3. Encodes as base64 PNG
  4. Returns the data URI string
- Frontend: `ResultItem` calls `get_icon` lazily for each app result, caches in a `Map<string, string>`, renders as `<img src={dataUri}>`.
- Cache: Icons are cached in a Rust-side `HashMap<String, String>` so extraction only happens once per path.

**Performance:** Icon extraction runs async, results render immediately with text badge fallback, icons pop in as they load.

**Dependency:** Add `windows` crate with features: `Win32_UI_Shell`, `Win32_Graphics_Gdi`, `Win32_UI_WindowsAndMessaging`.

**Priority:** High — biggest visual improvement.

---

## 5. File Preview (Quick Look)

**Trigger:** Press `Space` on any file result to open a preview panel.

**Supported file types:**
- Text files (.txt, .md, .rs, .py, .js, .json, .toml, .yaml, .csv, .log) — show first ~50 lines with syntax highlighting
- Images (.png, .jpg, .gif, .svg, .bmp, .webp) — show thumbnail
- PDF — show first page (via pdf.js or native rendering)
- Everything else — show file metadata (size, dates, path)

**Implementation:**
- New Tauri command `preview_file(path: &str) -> FilePreview` returning:
  ```rust
  struct FilePreview {
      file_type: String,      // "text", "image", "pdf", "binary"
      content: String,        // text content or base64 image data
      size: u64,
      modified: String,
      created: String,
  }
  ```
- Frontend: New `PreviewPanel` component that replaces the results view (like context menu does). Escape goes back.
- Text preview: use a `<pre>` with basic syntax highlighting (keyword coloring via regex, not a full parser).
- Image preview: `<img>` with the base64 data, max-width constrained.

**Priority:** High — a defining Raycast feature, big UX improvement.

---

## 6. Kill Process

**Trigger:** Input starts with `kill ` followed by a process name, or type a process name and use context menu.

**Implementation:**
- New `ProcessProvider` in `providers/process.rs`:
  - `search_processes(query, max)` — runs `tasklist /FO CSV /NH` and fuzzy-matches against process names
  - Returns results with category "Processes", action `KillProcess { pid, name }`
- New Tauri command `kill_process(pid: u32)` — runs `taskkill /PID <pid> /F`
- Confirmation dialog before killing (same pattern as destructive system commands)
- Show process memory usage in subtitle (from tasklist output)

**Detection:** Only activate when query starts with `kill ` prefix, or when user explicitly searches in a "processes" mode. Don't pollute normal search with process names.

**Priority:** Medium — power user feature, useful for devs.

---

## 7. Usage-Based Ranking

**Concept:** Track which results the user selects and boost frequently-chosen results to the top.

**Implementation:**
- New `UsageTracker` in `src-tauri/src/usage.rs`:
  - SQLite database at `%AppData%\Omni\usage.db`
  - Table: `usage(query TEXT, result_path TEXT, count INTEGER, last_used TEXT)`
  - On every `execute_action`, record the query + selected result path
  - On search, query the usage table for the current query and boost matching results
- Boosting logic:
  - After all providers return results, look up usage counts for the current query
  - Results with higher usage counts get moved to the top of their category
  - Results used more than 5 times for a given query get promoted to a "Frequent" category at the top
- Privacy: All data local, never transmitted. User can clear via Settings.

**Dependency:** Add `rusqlite` crate with `bundled` feature (bundles SQLite, no system dep).

**Priority:** High — makes the app feel intelligent and personalized over time.

---

## Implementation Order

Recommended sequence based on dependencies and impact:

| Phase | Features | Rationale |
|-------|----------|-----------|
| 1 | Unit Conversion + Currency | Quick wins, pure logic, high daily utility |
| 2 | Real App Icons | Biggest visual upgrade, independent of other features |
| 3 | Kill Process | Simple provider, follows existing pattern |
| 4 | Color Picker | Small feature, nice developer tool |
| 5 | Usage-Based Ranking | Needs SQLite, affects all search results |
| 6 | File Preview | Most complex, needs multiple file type handlers |

Each phase is independently shippable — no cross-dependencies between phases.
