//! Music control via playerctl (Linux stand-in for Music.app Mini Player).

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use std::process::Command;

pub struct MusicProvider;

fn playerctl(args: &[&str]) -> Option<String> {
    let output = Command::new("playerctl").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

impl Provider for MusicProvider {
    fn name(&self) -> &'static str {
        "music"
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let needle = match query.keyword.as_deref() {
            Some("music") => query.argument.to_lowercase(),
            Some("play") | Some("pause") | Some("next") | Some("previous")
                if query.argument.is_empty() =>
            {
                query.keyword.clone().unwrap().to_lowercase()
            }
            _ => return Vec::new(),
        };

        let status = playerctl(&["status"]).unwrap_or_else(|| "No player".into());
        let meta = playerctl(&["metadata", "--format", "{{artist}} — {{title}}"])
            .unwrap_or_else(|| "Not playing".into());

        let commands = [
            ("play", "Play", "playerctl", vec!["play".into()]),
            ("pause", "Pause", "playerctl", vec!["pause".into()]),
            (
                "playpause",
                "Play/Pause",
                "playerctl",
                vec!["play-pause".into()],
            ),
            ("next", "Next Track", "playerctl", vec!["next".into()]),
            (
                "previous",
                "Previous Track",
                "playerctl",
                vec!["previous".into()],
            ),
            ("stop", "Stop", "playerctl", vec!["stop".into()]),
        ];

        let mut items = vec![ResultItem::new(
            "music:now",
            meta,
            ItemKind::Music,
        )
        .with_subtitle(format!("Status: {status}"))
        .with_score(7_200)
        .with_actions(vec![Action::RunCommand {
            program: "playerctl".into(),
            args: vec!["play-pause".into()],
        }])];

        for (key, title, program, args) in commands {
            if needle.is_empty() || key.contains(&needle) || title.to_lowercase().contains(&needle)
            {
                items.push(
                    ResultItem::new(format!("music:{key}"), title, ItemKind::Music)
                        .with_subtitle("Control media player")
                        .with_score(7_000)
                        .with_actions(vec![Action::RunCommand {
                            program: program.into(),
                            args,
                        }]),
                );
            }
        }
        items
    }
}
