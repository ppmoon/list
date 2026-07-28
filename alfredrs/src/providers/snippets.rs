//! Snippets & keyword expansion.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use crate::paths::{data_dir, ensure_data_dir};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub keyword: String,
    pub content: String,
    pub auto_expand: bool,
}

#[derive(Default)]
pub struct SnippetsProvider {
    cache: Mutex<Option<Vec<Snippet>>>,
}

impl SnippetsProvider {
    pub fn path() -> std::path::PathBuf {
        data_dir().join("snippets.json")
    }

    pub fn load() -> Vec<Snippet> {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(path)
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or_else(default_snippets)
        } else {
            let snips = default_snippets();
            let _ = Self::save(&snips);
            snips
        }
    }

    pub fn save(snippets: &[Snippet]) -> anyhow::Result<()> {
        ensure_data_dir()?;
        std::fs::write(Self::path(), serde_json::to_string_pretty(snippets)?)?;
        Ok(())
    }

    /// If `typed` ends with a snippet keyword (word boundary), return expansion.
    pub fn auto_expand(typed: &str, snippets: &[Snippet]) -> Option<(String, String)> {
        let trimmed = typed.trim_end();
        for snip in snippets.iter().filter(|s| s.auto_expand) {
            if trimmed == snip.keyword
                || trimmed.ends_with(&format!(" {}", snip.keyword))
                || trimmed.ends_with(&format!("\n{}", snip.keyword))
            {
                return Some((snip.keyword.clone(), snip.content.clone()));
            }
        }
        None
    }

    fn snippets(&self) -> Vec<Snippet> {
        let mut guard = self.cache.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Self::load());
        }
        guard.clone().unwrap_or_default()
    }
}

fn default_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            id: "sig".into(),
            name: "Email signature".into(),
            keyword: ";sig".into(),
            content: "Best regards,\nSent via alfredrs".into(),
            auto_expand: true,
        },
        Snippet {
            id: "addr".into(),
            name: "Example address".into(),
            keyword: ";addr".into(),
            content: "221B Baker Street, London".into(),
            auto_expand: true,
        },
        Snippet {
            id: "lorem".into(),
            name: "Lorem ipsum".into(),
            keyword: ";lorem".into(),
            content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit.".into(),
            auto_expand: true,
        },
    ]
}

impl Provider for SnippetsProvider {
    fn name(&self) -> &'static str {
        "snippets"
    }

    fn keywords(&self) -> &[&'static str] {
        &["snip", "sp"]
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let needle = match query.keyword.as_deref() {
            Some("snip") | Some("sp") => query.argument.to_lowercase(),
            None => {
                // Also match when typing a snippet keyword directly.
                let raw = query.raw.trim();
                if raw.starts_with(';') {
                    raw.to_lowercase()
                } else {
                    return Vec::new();
                }
            }
            _ => return Vec::new(),
        };

        self.snippets()
            .into_iter()
            .filter(|s| {
                needle.is_empty()
                    || s.keyword.to_lowercase().contains(&needle)
                    || s.name.to_lowercase().contains(&needle)
                    || s.content.to_lowercase().contains(&needle)
                    || needle == s.keyword.to_lowercase()
            })
            .map(|s| {
                ResultItem::new(format!("snip:{}", s.id), s.name.clone(), ItemKind::Snippet)
                    .with_subtitle(format!("{} → {}", s.keyword, s.content.chars().take(60).collect::<String>()))
                    .with_score(8_000)
                    .with_actions(vec![
                        Action::ExpandSnippet(s.content.clone()),
                        Action::CopyText(s.content),
                    ])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_expand_keyword() {
        let snips = default_snippets();
        let hit = SnippetsProvider::auto_expand(";sig", &snips).unwrap();
        assert_eq!(hit.0, ";sig");
        assert!(hit.1.contains("Best regards"));
    }
}
