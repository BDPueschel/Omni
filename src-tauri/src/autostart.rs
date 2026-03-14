use std::process::Command;

const REG_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "Omni";

pub fn enable_autostart(exe_path: &str) -> Result<(), String> {
    Command::new("reg")
        .args(["add", REG_KEY, "/v", APP_NAME, "/t", "REG_SZ", "/d", exe_path, "/f"])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn disable_autostart() -> Result<(), String> {
    Command::new("reg")
        .args(["delete", REG_KEY, "/v", APP_NAME, "/f"])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn is_autostart_enabled() -> bool {
    Command::new("reg")
        .args(["query", REG_KEY, "/v", APP_NAME])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
