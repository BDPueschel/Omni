# Column Drag-and-Reorder with Persistence — Design Spec

**Date:** 2026-03-15
**Branch:** `feature/table-panel`
**Status:** Draft

## Overview

Allow users to reorder table panel columns via mouse drag-and-drop or keyboard shortcuts, with the order persisted across reboots via the existing OmniConfig system.

## Drag-and-Drop (Mouse)

- Column headers have `draggable="true"`.
- User grabs a column header, drags left/right. A translucent ghost of the header follows the cursor.
- As the ghost crosses column boundaries, columns visually swap to preview the new order.
- Drop to confirm the new order.
- Uses native HTML5 drag events (`dragstart`, `dragover`, `drop`) — no external libraries.

## Keyboard Reorder

- **Ctrl+Shift+Left/Right** moves the currently sorted column one position in that direction.
- The sort indicator identifies which column is "active" — that's the one that moves.
- If the active column is already at the edge, the shortcut is a no-op.

## Persistence

- Column order stored as an array of column key strings, e.g. `["type", "name", "path", "size", "date_modified"]`.
- New field on `OmniConfig` struct (`src-tauri/src/config.rs`): `table_column_order: Option<Vec<String>>`.
- `Option` so existing configs without this field deserialize cleanly (defaults to `None`).
- On app load, if config has a saved order, use it. Otherwise use the default: `["type", "name", "path", "size", "date_modified"]`.
- Saved whenever the user reorders (drag or keyboard) via the existing `save_config` Tauri command.

## State Flow

- `App.tsx` owns `columnOrder: SortColumn[]` state, initialized from config on mount.
- `TablePanel` receives `columnOrder` as a prop and renders headers + row cells in that order.
- When reorder happens (drag or keyboard), `App.tsx` updates state and calls `save_config` to persist.
- **Ctrl+1-5 sort shortcuts** map to visual position — whatever column is in position 1 gets Ctrl+1. This stays intuitive after reorder.

## Frontend Changes

### TablePanel.tsx

- Accept new prop: `columnOrder: SortColumn[]`.
- Accept new callback: `onColumnReorder: (newOrder: SortColumn[]) => void`.
- Render headers and row cells by iterating `columnOrder` instead of hardcoded JSX.
- Each header cell: `draggable="true"`, with `onDragStart`, `onDragOver`, `onDrop` handlers.
- Drag state: track `draggedCol` and `dragOverCol` in local state for visual feedback.
- CSS class `.col-dragging` on the dragged header (opacity reduction), `.col-drag-over` on the target (left-border highlight).

### App.tsx

- New state: `columnOrder: SortColumn[]`, initialized from config.
- Load column order from config on mount (in the `window-shown` listener or a dedicated effect).
- `onColumnReorder` callback: updates state + calls `save_config`.
- Ctrl+Shift+Left/Right handler in the table key block: swaps `tableSortColumn` position in `columnOrder`.
- Update Ctrl+1-5 to use `columnOrder[n-1]` instead of a hardcoded array.

### Help Overlay

- Add: `Ctrl+Shift+←→` — "Reorder table columns"
- Adjust help overlay height if clipping occurs.

## Backend Changes

### OmniConfig (config.rs)

- Add field: `pub table_column_order: Option<Vec<String>>`.
- With `#[serde(default)]` so existing config files without this field load cleanly.
- No validation needed — frontend treats unknown column names as a reset to defaults.

## Out of Scope

- Column resizing (widths stay fixed proportional)
- Column hiding/showing
- Persisting sort column/direction across sessions
