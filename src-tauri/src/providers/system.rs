use super::{ResultAction, SearchResult};

struct SystemCommand {
    name: &'static str,
    aliases: &'static [&'static str],
    command: &'static str,
    icon: &'static str,
}

const COMMANDS: &[SystemCommand] = &[
    SystemCommand { name: "Lock", aliases: &["lock", "lock screen", "lock pc"], command: "lock", icon: "lock" },
    SystemCommand { name: "Sleep", aliases: &["sleep", "suspend"], command: "sleep", icon: "moon" },
    SystemCommand { name: "Shutdown", aliases: &["shutdown", "shut down", "power off"], command: "shutdown", icon: "power" },
    SystemCommand { name: "Restart", aliases: &["restart", "reboot"], command: "restart", icon: "refresh" },
    SystemCommand { name: "Empty Recycle Bin", aliases: &["recycle", "recycle bin", "empty recycle", "trash"], command: "empty_recycle_bin", icon: "trash" },
    SystemCommand { name: "Sign Out", aliases: &["sign out", "log out", "logout", "logoff"], command: "sign_out", icon: "log-out" },
];

pub struct SystemProvider;

impl SystemProvider {
    pub fn evaluate(input: &str) -> Vec<SearchResult> {
        let query = input.trim().to_lowercase();
        if query.is_empty() {
            return vec![];
        }
        COMMANDS.iter()
            .filter(|cmd| cmd.aliases.iter().any(|alias| alias.starts_with(&query) || alias.contains(&query)))
            .map(|cmd| SearchResult {
                category: "System".to_string(),
                title: cmd.name.to_string(),
                subtitle: "System command".to_string(),
                action: ResultAction::SystemCommand { command: cmd.command.to_string() },
                icon: cmd.icon.to_string(),
                size: None,
                date_modified: None,
            })
            .collect()
    }
}
