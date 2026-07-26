//! System commands — sleep, logout, empty trash, etc.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;

pub struct SystemProvider;

#[derive(Clone)]
struct SysCmd {
    keywords: &'static [&'static str],
    title: &'static str,
    subtitle: &'static str,
    program: &'static str,
    args: &'static [&'static str],
}

fn commands() -> Vec<SysCmd> {
    vec![
        SysCmd {
            keywords: &["lock", "screensaver"],
            title: "Lock Screen",
            subtitle: "Lock the session",
            program: "loginctl",
            args: &["lock-session"],
        },
        SysCmd {
            keywords: &["sleep", "suspend"],
            title: "Sleep",
            subtitle: "Suspend the system",
            program: "systemctl",
            args: &["suspend"],
        },
        SysCmd {
            keywords: &["hibernate"],
            title: "Hibernate",
            subtitle: "Hibernate the system",
            program: "systemctl",
            args: &["hibernate"],
        },
        SysCmd {
            keywords: &["restart", "reboot"],
            title: "Restart",
            subtitle: "Reboot the computer",
            program: "systemctl",
            args: &["reboot"],
        },
        SysCmd {
            keywords: &["shutdown", "halt"],
            title: "Shut Down",
            subtitle: "Power off the computer",
            program: "systemctl",
            args: &["poweroff"],
        },
        SysCmd {
            keywords: &["logout", "log out"],
            title: "Log Out",
            subtitle: "End the current session",
            program: "loginctl",
            args: &["terminate-user", ""],
        },
        SysCmd {
            keywords: &["emptytrash", "trash"],
            title: "Empty Trash",
            subtitle: "Remove files from the trash",
            program: "rm",
            args: &["-rf"],
        },
        SysCmd {
            keywords: &["eject"],
            title: "Eject Removable Media",
            subtitle: "Eject all ejectable volumes",
            program: "eject",
            args: &["-a"],
        },
    ]
}

impl Provider for SystemProvider {
    fn name(&self) -> &'static str {
        "system"
    }

    fn keywords(&self) -> &[&'static str] {
        &["sys", "lock", "screensaver", "sleep", "suspend", "hibernate", "restart", "reboot", "shutdown", "halt", "logout", "emptytrash", "trash", "eject"]
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let needle = match query.keyword.as_deref() {
            Some("sys") => query.argument.to_lowercase(),
            Some(other) => {
                // Direct keyword match against system command names.
                let lower = other.to_lowercase();
                if commands()
                    .iter()
                    .any(|c| c.keywords.iter().any(|k| *k == lower))
                    && query.argument.is_empty()
                {
                    lower
                } else {
                    return Vec::new();
                }
            }
            None => {
                let raw = query.raw.trim().to_lowercase();
                if commands()
                    .iter()
                    .any(|c| c.keywords.iter().any(|k| *k == raw || raw.starts_with(k)))
                {
                    raw
                } else {
                    return Vec::new();
                }
            }
        };

        commands()
            .into_iter()
            .filter(|c| {
                c.keywords
                    .iter()
                    .any(|k| k.contains(&needle) || needle.contains(k))
                    || c.title.to_lowercase().contains(&needle)
            })
            .map(|c| {
                let mut args: Vec<String> = c.args.iter().map(|s| s.to_string()).collect();
                if c.title == "Log Out" {
                    let user = std::env::var("USER").unwrap_or_default();
                    args = vec!["terminate-user".into(), user];
                }
                if c.title == "Empty Trash" {
                    let trash = dirs::home_dir()
                        .unwrap_or_default()
                        .join(".local/share/Trash/files");
                    args = vec!["-rf".into(), trash.display().to_string()];
                }
                ResultItem::new(format!("sys:{}", c.title), c.title, ItemKind::System)
                    .with_subtitle(c.subtitle)
                    .with_score(6_000)
                    .with_actions(vec![Action::RunCommand {
                        program: c.program.into(),
                        args,
                    }])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Provider;

    #[test]
    fn finds_sleep() {
        let items = SystemProvider.search(&Query::parse("sleep"), &Config::default());
        assert!(items.iter().any(|i| i.title == "Sleep"));
    }
}
