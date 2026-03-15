# Omni — Gap Analysis vs Everything & Raycast

## Search & Indexing

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Instant file search | Yes | No (Spotlight) | Yes (es.exe) | Done |
| Fuzzy fragment matching | Yes | Yes | Yes (*foo*bar*) | Done |
| Path-scoped search | Yes (path:) | No | Yes (C:\path\) | Done |
| Wildcard patterns | Yes | No | Yes (*.rs) | Done |
| Regex search | Yes | No | Yes (regex: or r:) | Done |
| Boolean operators (OR, NOT) | Yes | No | Yes (| and ! passthrough) | Done |
| File content search | Yes (1.5a) | No | No | Gap |
| Sort by date/size/name | Yes | N/A | Yes (files by date-modified desc) | Done |
| File size filter | Yes (size:) | No | Yes (passthrough to es.exe) | Done |
| Date filters | Yes (dm:today) | No | Yes (passthrough to es.exe) | Done |
| Bookmarks/favorites | Yes | Yes | Yes (usage-based Frequent section) | Done |
| Search history | Yes | Yes | Yes (usage tracking boosts results) | Done |
| Preview pane | Yes | Yes (Quick Look) | Yes (Ctrl+Space, text/image/binary) | Done |

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
| Kill process | No | Yes | Yes (kill <name>, confirmation) | Done |
| Clipboard history | No | Yes | No | Gap — major Raycast feature |
| Snippet expansion | No | Yes | No | Nice to have |
| Window management | No | Yes | No | Gap |
| Emoji picker | No | Yes | No | Nice to have |

## Context Menu / Actions

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Open containing folder | Yes | Yes | Yes | Done |
| Copy path | Yes | Yes | Yes | Done |
| Open in terminal | No | Yes | Yes (Windows Terminal) | Done |
| Open in editor | No | Yes | Yes (VS Code) | Done |
| Run as admin | No | No | Yes | Omni ahead |
| Open with... | Yes | Yes | Yes (Windows Open As dialog) | Done |
| Move/Copy file | Yes | No | Yes (folder picker dialogs) | Done |
| Delete (recycle bin) | Yes | No | Yes (with confirmation) | Done |
| File properties/metadata | Yes | Yes (Quick Look) | Yes (preview shows size/date) | Done |

## UI/UX

| Feature | Everything | Raycast | Omni | Status |
|---------|-----------|---------|------|--------|
| Global hotkey | No | Yes | Yes (Alt+Space) | Done |
| Dark theme | Yes | Yes | Yes (Acrylic) | Done |
| Dynamic window resize | No | Yes | Yes (compact → expands) | Done |
| Keyboard-first navigation | Partial | Yes | Yes (full suite) | Done |
| Category grouping | No (flat) | Yes | Yes | Done |
| Expandable categories | No | No | Yes (Ctrl+E toggle) | Omni ahead |
| Context menu (inline) | Right-click | Yes | Yes (Shift+→) | Done |
| Help overlay | No | Yes | Yes (Ctrl+H) | Done |
| App icons (real thumbnails) | Yes | Yes | Yes (Win32 Shell API) | Done |
| File preview | Yes | Yes (Quick Look) | Yes (Ctrl+Space) | Done |
| Custom app icon | N/A | Yes | Yes (marble glass orb) | Done |
| Plugin/extension system | No | Yes (store) | No | Gap |
| Themes/customization | Limited | Yes | Partial (opacity) | Gap |

## Remaining Gaps

| Feature | Difficulty | Impact | Notes |
|---------|-----------|--------|-------|
| Clipboard history | High | High | Raycast's most-used feature, needs background service |
| Window management | Medium | Medium | Move/resize windows by command |
| Plugin/extension system | Very High | High | Raycast's ecosystem — major undertaking |
| File content search | Medium | Medium | Everything 1.5a supports this natively |
| Emoji picker | Medium | Low | |
| Timezone display | Low | Low | |
| Full theme system | Medium | Low | Beyond opacity slider |

## Where Omni is Ahead of Both

- **Everything-powered search** — faster than Raycast's Spotlight, more integrated than Everything's UI
- **Expandable categories** (Ctrl+E toggle) — neither app has this
- **Path context scoping** — type a path to scope search instantly
- **Run as admin** from inline context menu
- **Directories as a separate category**
- **Fuzzy fragment search** with Everything's full NTFS index
- **File preview with keyboard navigation** — scroll, page, jump in preview mode
- **Usage-based ranking** — learns your preferences across sessions
- **Regex search** — prefix with regex: or r:
- **Full file management** — open with, copy to, move to, delete from context menu

## Score Summary

| Category | Total Features | Omni Done | Coverage |
|----------|---------------|-----------|----------|
| Search & Indexing | 13 | 12 | 92% |
| App Launching | 6 | 5 | 83% |
| Calculator / Quick Actions | 5 | 4 | 80% |
| System & Productivity | 6 | 3 | 50% |
| Context Menu | 9 | 9 | 100% |
| UI/UX | 13 | 11 | 85% |
| **Total** | **52** | **44** | **85%** |
