//! Terminal / shell integration — Alfred `>` prefix.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;

pub struct ShellProvider;

impl Provider for ShellProvider {
    fn name(&self) -> &'static str {
        "shell"
    }

    fn search(&self, query: &Query, config: &Config) -> Vec<ResultItem> {
        if query.keyword.as_deref() != Some(">") {
            return Vec::new();
        }
        let cmd = query.argument.trim();
        if cmd.is_empty() {
            return vec![ResultItem::new("shell:open", "Open Terminal", ItemKind::Shell)
                .with_subtitle(config.terminal.clone())
                .with_score(7_000)
                .with_actions(vec![Action::RunCommand {
                    program: config.terminal.clone(),
                    args: vec![],
                }])];
        }

        vec![
            ResultItem::new("shell:run", format!("Run in Terminal: {cmd}"), ItemKind::Shell)
                .with_subtitle(format!("{} -e {cmd}", config.terminal))
                .with_score(9_000)
                .with_actions(vec![Action::RunCommand {
                    program: config.terminal.clone(),
                    args: vec!["-e".into(), "bash".into(), "-lc".into(), cmd.to_string()],
                }]),
            ResultItem::new(
                "shell:bg",
                format!("Run silently: {cmd}"),
                ItemKind::Shell,
            )
            .with_subtitle("Execute via bash without opening a terminal window")
            .with_score(8_500)
            .with_actions(vec![Action::RunCommand {
                program: "bash".into(),
                args: vec!["-lc".into(), cmd.to_string()],
            }]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Provider;

    #[test]
    fn shell_prefix_offers_run() {
        let items = ShellProvider.search(&Query::parse("> ls -la"), &Config::default());
        assert!(items.iter().any(|i| i.title.contains("ls -la")));
    }
}
