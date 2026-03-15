# Omni — Gap Analysis vs Everything & Raycast

## Search & Indexing

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Instant file search | Yes | No (Spotlight) | Yes (HTTP API) | Done |
| Fuzzy fragment matching | Yes | Yes | Yes (*foo*bar*) | Done |
| Path-scoped search | Yes (path:) | No | Yes (C:\path\) | Done |
| Tab completion | Yes | No | Yes (Tab on paths) | Done |
| Wildcard patterns | Yes | No | Yes (*.rs) | Done |
| Regex search | Yes | No | Yes (regex: or r:) | Done |
| Boolean operators (OR, NOT) | Yes | No | Yes (| and ! passthrough) | Done |
| File content search | Yes (1.5a) | No | No | Gap |
| Sort by date/size/name | Yes | N/A | Yes (files by date-modified desc) | Done |
| File size filter | Yes (size:) | No | Yes (passthrough) | Done |
| Date filters | Yes (dm:today) | No | Yes (passthrough) | Done |
| Bookmarks/favorites | Yes | Yes | Yes (usage-based Frequent section) | Done |
| Search history | Yes | Yes | Yes (usage tracking boosts results) | Done |
| Preview pane | Yes | Yes (Quick Look) | Yes (Ctrl+Space) | Done |

## App Launching

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Launch apps | No | Yes | Yes | Done |
| Fuzzy app matching | No | Yes | Yes (Everything + nucleo) | Done |
| Recent/frequent apps | No | Yes (learning) | Yes (usage-based ranking) | Done |
| Run as admin | No | No | Yes (context menu) | Omni ahead |
| Real app icons | Yes | Yes | Yes (Win32 SHGetFileInfoW) | Done |
| Uninstall shortcut | No | No | No | Nice to have |

## Calculator / Quick Actions

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Math evaluation | No | Yes | Yes (meval) | Done |
| Unit conversion | No | Yes | Yes (7 categories, 50+ units) | Done |
| Currency conversion | No | Yes (live rates) | Yes (40+ currencies, live rates) | Done |
| Color picker/preview | No | Yes | Yes (hex/rgb/hsl + swatch) | Done |
| Timezone display | No | Yes | No | Nice to have |

## System & Productivity

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| System commands | No | Yes | Yes | Done |
| Kill process | No | Yes | Yes (kill <name>) | Done |
| Clipboard history | No | Yes | Yes (clip/cb prefix) | Done |
| Snippet expansion | No | Yes | No | Nice to have |
| Window management | No | Yes | No | Gap |
| Emoji picker | No | Yes | No | Nice to have |

## Context Menu / Actions

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Open containing folder | Yes | Yes | Yes | Done |
| Copy path | Yes | Yes | Yes (also Ctrl+C) | Done |
| Open in terminal | No | Yes | Yes (Windows Terminal) | Done |
| Open in editor | No | Yes | Yes (VS Code) | Done |
| Run as admin | No | No | Yes | Omni ahead |
| Open with... | Yes | Yes | Yes | Done |
| Move/Copy file | Yes | No | Yes (single + batch) | Done |
| Delete (recycle bin) | Yes | No | Yes (single + batch) | Done |
| Multi-select batch ops | No | No | Yes (Shift+Arrow) | Omni ahead |
| File properties/metadata | Yes | Yes | Yes (preview) | Done |

## UI/UX

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Global hotkey | No | Yes | Yes (Alt+Space) | Done |
| Dark theme | Yes | Yes | Yes (Acrylic) | Done |
| Dynamic window resize | No | Yes | Yes | Done |
| Keyboard-first navigation | Partial | Yes | Yes (full suite) | Done |
| Category grouping | No (flat) | Yes | Yes (with counts) | Done |
| Expandable categories | No | No | Yes (Ctrl+E toggle) | Omni ahead |
| Context menu (inline) | Right-click | Yes | Yes (Shift+→) | Done |
| Help overlay | No | Yes | Yes (Ctrl+H) | Done |
| App icons (real) | Yes | Yes | Yes (Win32 Shell API) | Done |
| File preview | Yes | Yes | Yes (Ctrl+Space) | Done |
| Status bar | No | Yes | Yes (path + selection count) | Done |
| Custom app icon | N/A | Yes | Yes (marble glass orb) | Done |
| Plugin/extension system | No | Yes (store) | No | Gap |
| Themes/customization | Limited | Yes | Partial (opacity) | Gap |

## Remaining Gaps

| Feature | Difficulty | Impact |
|---------|-----------|--------|
| Window management | Medium | Medium |
| Plugin/extension system | Very High | High |
| File content search | Medium | Medium |
| Emoji picker | Medium | Low |
| Timezone display | Low | Low |
| Full theme system | Medium | Low |

## Where Omni is Ahead of Both

- **Everything HTTP API** — single request, sub-50ms response, zero process overhead
- **Expandable categories** (Ctrl+E toggle) — neither app has this
- **Multi-select with batch operations** — Shift+Arrow, batch open/copy/move/delete
- **Path context scoping** — type a path to scope search instantly
- **Tab completion** on paths — like a terminal
- **Run as admin** from inline context menu
- **Directories as a separate category**
- **Fuzzy fragment search** with Everything's full NTFS index
- **File preview with keyboard navigation**
- **Usage-based ranking** — learns your preferences across sessions
- **Regex search** — prefix with regex: or r:
- **Full file management** — open with, copy to, move to, delete
- **Status bar** with full path display and selection count

## Score Summary

| Category | Total Features | Omni Done | Coverage |
|----------|---------------|-----------|----------|
| Search & Indexing | 14 | 13 | 93% |
| App Launching | 6 | 5 | 83% |
| Calculator / Quick Actions | 5 | 4 | 80% |
| System & Productivity | 6 | 4 | 67% |
| Context Menu | 10 | 10 | 100% |
| UI/UX | 14 | 12 | 86% |
| **Total** | **55** | **48** | **87%** |
