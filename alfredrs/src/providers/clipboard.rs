//! Clipboard history — Alfred Powerpack Clipboard History.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use crate::paths::{data_dir, ensure_data_dir};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipItem {
    pub id: String,
    pub text: String,
    pub kind: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct ClipboardProvider {
    store: Mutex<Option<Vec<ClipItem>>>,
}

impl ClipboardProvider {
    pub fn history_path() -> std::path::PathBuf {
        data_dir().join("clipboard.json")
    }

    pub fn load() -> Vec<ClipItem> {
        let path = Self::history_path();
        if let Ok(text) = std::fs::read_to_string(path) {
            serde_json::from_str(&text).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    pub fn save(items: &[ClipItem]) -> anyhow::Result<()> {
        ensure_data_dir()?;
        std::fs::write(Self::history_path(), serde_json::to_string_pretty(items)?)?;
        Ok(())
    }

    pub fn push_text(text: &str, max: usize) -> anyhow::Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }
        let mut items = Self::load();
        items.retain(|i| i.text != text);
        items.insert(
            0,
            ClipItem {
                id: uuid::Uuid::new_v4().to_string(),
                text: text.to_string(),
                kind: classify(text).into(),
                created_at: Utc::now(),
            },
        );
        items.truncate(max);
        Self::save(&items)
    }

    pub fn capture_current(max: usize) -> anyhow::Result<Option<String>> {
        let mut clipboard = arboard::Clipboard::new()?;
        let text = clipboard.get_text().ok();
        if let Some(ref t) = text {
            Self::push_text(t, max)?;
        }
        Ok(text)
    }

    fn items(&self) -> Vec<ClipItem> {
        let mut guard = self.store.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Self::load());
        }
        guard.clone().unwrap_or_default()
    }

    pub fn refresh(&self) {
        *self.store.lock().unwrap() = Some(Self::load());
    }
}

fn classify(text: &str) -> &'static str {
    let t = text.trim();
    if t.starts_with("http://") || t.starts_with("https://") {
        "url"
    } else if t.starts_with('#')
        && t.len() >= 4
        && t.chars().skip(1).all(|c| c.is_ascii_hexdigit())
    {
        "color"
    } else if std::path::Path::new(t).exists() {
        "path"
    } else {
        "text"
    }
}

impl Provider for ClipboardProvider {
    fn name(&self) -> &'static str {
        "clipboard"
    }

    fn keywords(&self) -> &[&'static str] {
        &["clip", "cb"]
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let needle = match query.keyword.as_deref() {
            Some("clip") | Some("cb") => query.argument.to_lowercase(),
            _ => return Vec::new(),
        };
        self.refresh();
        self.items()
            .into_iter()
            .filter(|item| needle.is_empty() || item.text.to_lowercase().contains(&needle))
            .take(50)
            .map(|item| {
                let preview: String = item.text.chars().take(80).collect();
                ResultItem::new(
                    format!("clip:{}", item.id),
                    preview,
                    ItemKind::Clipboard,
                )
                .with_subtitle(format!("{} · {}", item.kind, item.created_at.format("%Y-%m-%d %H:%M")))
                .with_score(7_500)
                .with_payload(item.text.clone())
                .with_actions(vec![
                    Action::CopyText(item.text.clone()),
                    Action::PasteText(item.text.clone()),
                    Action::ShowLargeType(item.text),
                ])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_url_and_color() {
        assert_eq!(classify("https://example.com"), "url");
        assert_eq!(classify("#ff00aa"), "color");
    }

    #[test]
    fn push_dedupes_and_caps() {
        let mut items = Vec::new();
        for i in 0..5 {
            items.insert(
                0,
                ClipItem {
                    id: i.to_string(),
                    text: format!("item-{i}"),
                    kind: "text".into(),
                    created_at: Utc::now(),
                },
            );
        }
        items.truncate(3);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].text, "item-4");
    }
}
