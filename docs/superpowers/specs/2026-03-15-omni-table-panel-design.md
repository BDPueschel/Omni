# Omni Table Panel — Design Spec

**Date:** 2026-03-15
**Branch:** `feature/table-panel`
**Status:** Draft

## Overview

Add an Everything-style sortable column view to Omni that opens as a side panel to the right of the existing result list. The launcher stays compact by default; when the user wants to browse and sort file results, they expand into a wider table view. This preserves Omni's fast, keyboard-driven launcher identity while adding data-rich file browsing when needed.

## Trigger & Window Behavior

- **Hotkey:** Ctrl+T toggles the table panel open/closed.
- **Activation guard:** Ctrl+T only activates when the current results contain Files or Directories. If neither category has results, the hotkey is a no-op.
- **Window resize:** When the table opens, the window widens from 600px to ~1200px (capped at 85% of `screen.availWidth`). The window anchors to its left edge and grows rightward.
- **Screen edge handling:** Before widening, check `window.screen.availWidth` and the window's current X position. If expanding right would overflow, shift the window left first, then widen. On close, restore the original position and width (600px).
- **Layout:** Left side remains exactly as today — full result list with all categories at ~600px. Right side is the new table panel showing file/directory results with sortable columns. A subtle vertical divider separates them.

## Focus Model

- When the table opens, keyboard focus moves to the table panel.
- **Tab** cycles focus between the result list and the table panel.
- **Escape** closes the table and returns focus to the result list.
- **Ctrl+T** also closes the table (toggle behavior).
- **Visual indicator:** The active panel has a thin accent-colored border and slightly brighter background. The inactive panel dims slightly. This makes it always clear where keystrokes are going.

## Table Panel Content

### Columns

| Column | Description | Width | Default Sort |
|--------|-------------|-------|-------------|
| Name | Filename with extension-based icon (reuses existing icon set) | ~35% | Alphabetical |
| Path | Parent directory (not full path — Name covers the filename) | ~35% | Alphabetical |
| Size | Human-readable (KB, MB, GB), formatted on frontend from raw bytes | ~12% | Numeric |
| Date Modified | Relative when recent ("2h ago", "yesterday"), absolute when older ("2026-01-15") | ~18% | Newest first |

### Sorting

- Click a column header to sort by that column. Click again to reverse direction.
- A small arrow indicator (up/down chevron) shows the current sort column and direction.
- **Keyboard shortcut:** Ctrl+1/2/3/4 sorts by column number while the table panel is focused.
- **Default sort:** Date Modified, newest first (matches Everything's default and the current HTTP API sort).

### Row Interaction

- Arrow Up/Down navigates rows.
- **Enter** opens the selected file/directory.
- **Shift+Right** opens the context menu (same actions as the existing context menu).
- **Ctrl+Space** opens the file preview, replacing the table content in the same panel space. Ctrl+Space again closes the preview and returns to the table.
- **Shift+Up/Down** multi-selects rows. Batch context menu works the same as the current result list.

## Backend Changes

### SearchResult Struct

Add optional metadata fields. Existing non-file results pass `None`, so this is backward-compatible:

```rust
pub struct SearchResult {
    pub category: String,
    pub title: String,
    pub subtitle: String,
    pub action: ResultAction,
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_modified: Option<String>,
}
```

### Everything HTTP API

- Add `size_column=1` and `date_modified_column=1` to the query string in `query_http()`.
- Update `EverythingHttpResult` to deserialize `size` and `date_modified` from the JSON response.
- Populate the new fields when building file/directory `SearchResult`s.
- Non-Everything providers (apps, calculator, system, etc.) pass `None` for both fields.

### New Tauri Command: `search_table`

- Similar to `search` but requests more results (up to 100) and always includes size/date metadata.
- Keeps the normal `search` path lean and fast.
- The table panel calls `search_table` when it opens, using the current query.

### No new dependencies

Everything needed is already available in the Everything HTTP API response — we're requesting additional columns.

## Frontend Architecture

### New Component: `TablePanel.tsx`

- Receives an array of file/directory results with metadata (size, date_modified).
- Renders a column header row (clickable for sorting) with sort direction indicators.
- Renders scrollable rows with the 4 columns.
- Manages its own sort state (column, direction) internally.
- Emits callbacks for: execute (Enter), context menu (Shift+Right), preview (Ctrl+Space).

### App.tsx State Additions

- `tableOpen: boolean` — whether the table panel is visible.
- `activePanel: "results" | "table"` — which panel currently has keyboard focus.
- `tableSelectedIndex: number` — independent row selection for the table.
- `tableResults: SearchResult[]` — results with metadata from `search_table`.

### App.tsx Key Handler Changes

- Ctrl+T: toggles `tableOpen`, invokes `search_table` if opening, flips `activePanel` to `"table"`.
- Tab (when table is open): cycles `activePanel` between `"results"` and `"table"`.
- Key routing: `handleKeyDown` checks `activePanel` to decide whether keys go to the result list or the table.

### Window Resize Logic

The existing `useEffect` that calls `LogicalSize` gets a new branch: if `tableOpen`, set width to ~1200px (capped at 85% of screen width). On close, restore to 600px.

### Styling (styles.css)

- `.panel-active` — thin accent-colored left/top border, slightly brighter background.
- `.panel-inactive` — slightly dimmed, no accent border.
- `.table-panel` — right-side panel with column layout.
- `.table-header` — sticky header row with clickable column labels.
- `.table-row` — individual row styling, selected state, multi-selected state.
- `.table-divider` — subtle vertical separator between panels.

## Live Query Updates

When the user types while the table is open, results update live:

- The table re-fetches via `search_table` with the new query.
- The table stays open and updates in place — no need to close and reopen.
- If the new query produces zero file/directory results, the table shows an empty state ("No file results") but remains open.

## Interaction with Existing Features

- **Ctrl+E:** If the table is open, Ctrl+E is a no-op for Files/Directories (table already shows the expanded set). Ctrl+E still works for other categories in the left panel.
- **Preview from table:** Ctrl+Space opens preview replacing table content. Ctrl+Space again returns to the table.
- **Context menu from table:** Shift+Right opens the same context menu, positioned in the table panel area.

## Out of Scope (v1)

- Column resizing (fixed proportional widths)
- Column reordering / hiding
- Filtering within the table (search bar handles this)
- Synced selection between left list and table
- Inline file renaming
