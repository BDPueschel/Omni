# Omni Theming Approach

## How accent colors work

All accent colors in the CSS use CSS custom properties defined on `:root`. Each property is a complete `rgba()` value at a specific opacity level:

```css
:root {
  --accent-03: rgba(130, 180, 255, 0.03);
  --accent-08: rgba(130, 180, 255, 0.08);
  --accent-12: rgba(130, 180, 255, 0.12);
  --accent-15: rgba(130, 180, 255, 0.15);
  --accent-20: rgba(130, 180, 255, 0.2);
  --accent-25: rgba(130, 180, 255, 0.25);
  --accent-50: rgba(130, 180, 255, 0.5);
  --accent-70: rgba(130, 180, 255, 0.7);
}
```

Used throughout the CSS like:
```css
.active-group { border-color: var(--accent-25); }
.selected     { background: var(--accent-08); }
.panel-active { border-top: 2px solid var(--accent-50); }
```

## Why pre-computed values instead of channel variables

Chromium/WebView2 does **not** support injecting individual RGB channels via CSS vars into `rgba()`:

```css
/* DOES NOT WORK in Chromium */
--r: 130; --g: 180; --b: 255;
background: rgba(var(--r), var(--g), var(--b), 0.5);

/* ALSO DOES NOT WORK */
--accent: 130, 180, 255;
background: rgba(var(--accent), 0.5);
```

The only reliable approach is pre-computed complete color values as CSS vars, overridden at runtime via JS.

## Reading Windows system accent color (Rust)

The accent color lives in the registry at `HKCU\SOFTWARE\Microsoft\Windows\DWM\AccentColor` as a `DWORD` in **ABGR** format (not ARGB).

```rust
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD,
};

let subkey: Vec<u16> = "SOFTWARE\\Microsoft\\Windows\\DWM\0"
    .encode_utf16().collect();
let value: Vec<u16> = "AccentColor\0"
    .encode_utf16().collect();
let mut data: u32 = 0;
let mut size: u32 = 4;

let result = unsafe {
    RegGetValueW(
        HKEY_CURRENT_USER,
        PCWSTR(subkey.as_ptr()),
        PCWSTR(value.as_ptr()),
        RRF_RT_REG_DWORD,
        None,
        Some(&mut data as *mut u32 as *mut _),
        Some(&mut size),
    )
};

if result.is_ok() {
    // ABGR format — extract RGB
    let r = (data & 0xFF) as u8;
    let g = ((data >> 8) & 0xFF) as u8;
    let b = ((data >> 16) & 0xFF) as u8;
    // Use (r, g, b) for your theme
}
```

Requires the `Win32_System_Registry` feature in `Cargo.toml`:
```toml
windows = { version = "0.61", features = ["Win32_System_Registry"] }
```

## Applying from JS (Tauri frontend)

Call the Rust command to get RGB, then override all accent vars:

```js
const [r, g, b] = await invoke("get_system_accent");
const el = document.documentElement;
const opacities = [0.03, 0.08, 0.12, 0.15, 0.2, 0.25, 0.5, 0.7];
for (const a of opacities) {
    const name = `--accent-${String(a * 100).padStart(2, '0')}`;
    el.style.setProperty(name, `rgba(${r}, ${g}, ${b}, ${a})`);
}
```

To reset back to defaults, call `el.style.removeProperty(name)` for each — the CSS `:root` values take over again.

## Config persistence

Stored as a boolean `use_system_accent` in the app config JSON. Re-read on every window-shown event so settings changes take effect without a full restart.
