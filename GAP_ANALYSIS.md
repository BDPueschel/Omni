# Omni — Gap Analysis vs Everything & Raycast

## Search & Indexing

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Instant file search | Yes | No (Spotlight) | Yes (es.exe) | Done |
| Fuzzy fragment matching | Yes | Yes | Yes (*foo*bar*) | Done |
| Path-scoped search | Yes (path:) | No | Yes (C:\path\) | Done |
| Wildcard patterns | Yes | No | Yes (*.rs) | Done |
| Regex search | Yes | No | No | Gap — es.exe supports `-r` flag |
| Boolean operators (OR, NOT) | Yes | No | Partial (| and !) | Needs testing |
| File content search | Yes (1.5a) | No | No | Gap — needs Everything content indexing |
| Sort by date/size/name | Yes | N/A | Partial (files by date-modified) | Gap — no user-selectable sort |
| File size filter | Yes (size:) | No | No | Gap — es.exe supports size filters |
| Date filters | Yes (dm:today) | No | No | Gap — es.exe supports date filters |
| Bookmarks/favorites | Yes | Yes | No | Gap |
| Search history | Yes | Yes | No | Gap |
| Preview pane | Yes | Yes (Quick Look) | No | Gap |

## App Launching

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Launch apps | No | Yes | Yes | Done |
| Fuzzy app matching | No | Yes | Yes (Everything + nucleo) | Done |
| Recent/frequent apps | No | Yes (learning) | No | Gap — no usage tracking |
| Run as admin | No | No | Yes (context menu) | Omni ahead |
| Uninstall shortcut | No | No | No | Nice to have |

## Calculator / Quick Actions

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Math evaluation | No | Yes | Yes | Done |
| Unit conversion | No | Yes | No | Gap |
| Currency conversion | No | Yes (live rates) | No | Gap |
| Timezone display | No | Yes | No | Nice to have |
| Color picker/preview | No | Yes | No | Gap |

## System & Productivity

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| System commands | No | Yes | Yes | Done |
| Clipboard history | No | Yes | No | Gap — major Raycast feature |
| Snippet expansion | No | Yes | No | Nice to have |
| Window management | No | Yes | No | Gap |
| Kill process | No | Yes | No | Gap |
| Emoji picker | No | Yes | No | Nice to have |

## Context Menu / Actions

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Open containing folder | Yes | Yes | Yes | Done |
| Copy path | Yes | Yes | Yes | Done |
| Open in terminal | No | Yes | Yes | Done |
| Open in editor | No | Yes | Yes (VS Code) | Done |
| Run as admin | No | No | Yes | Omni ahead |
| Move/Copy/Delete file | Yes | No | No | Gap |
| File properties/metadata | Yes | Yes (Quick Look) | No | Gap |
| Open with... (app picker) | Yes | Yes | No | Gap |

## UI/UX

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Global hotkey | No | Yes | Yes (Alt+Space) | Done |
| Dark theme | Yes | Yes | Yes (Acrylic) | Done |
| Dynamic window resize | No | Yes | Yes | Done |
| Keyboard-first navigation | Partial | Yes | Yes | Done |
| Category grouping | No (flat) | Yes | Yes | Done |
| Expandable categories | No | No | Yes (Ctrl+E) | Omni ahead |
| Context menu (inline) | Right-click | Yes | Yes (Shift+→) | Done |
| Help overlay | No | Yes | Yes (Ctrl+H) | Done |
| App icons (real thumbnails) | Yes | Yes | No (text badges) | Gap |
| File thumbnails/preview | Yes | Yes | No | Gap |
| Plugin/extension system | No | Yes (store) | No | Gap |
| Themes/customization | Limited | Yes | Partial (opacity) | Gap |

## Where Omni is Already Ahead

- **Everything-powered search** — faster than Raycast's Spotlight
- **Expandable categories** (Ctrl+E) — neither app has this
- **Path context scoping** — type a path to filter
- **Run as admin** from context menu
- **Directories as a separate category**
- **Fuzzy fragment search** with Everything's full index
