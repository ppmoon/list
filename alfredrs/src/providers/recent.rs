//! Recently used documents (XDG recently-used.xbel).

use crate::config::Config;
use crate::model::{ItemKind, Query, ResultItem};
use crate::providers::files::file_actions;
use crate::providers::Provider;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct RecentProvider {
    cache: Mutex<Option<Vec<PathBuf>>>,
}

impl RecentProvider {
    pub fn load() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(data) = dirs::data_dir() {
            let xbel = data.join("recently-used.xbel");
            if let Ok(text) = std::fs::read_to_string(xbel) {
                paths.extend(parse_xbel(&text));
            }
        }
        paths.retain(|p| p.exists());
        paths
    }

    fn recent(&self) -> Vec<PathBuf> {
        let mut guard = self.cache.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Self::load());
        }
        guard.clone().unwrap_or_default()
    }
}

pub fn parse_xbel(text: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(idx) = line.find("href=\"file://") {
            let rest = &line[idx + "href=\"file://".len()..];
            if let Some(end) = rest.find('"') {
                let encoded = &rest[..end];
                let decoded = urlencoding::decode(encoded)
                    .unwrap_or_else(|_| encoded.into())
                    .to_string();
                out.push(PathBuf::from(decoded));
            }
        }
    }
    out
}

impl Provider for RecentProvider {
    fn name(&self) -> &'static str {
        "recent"
    }

    fn keywords(&self) -> &[&'static str] {
        &["recent"]
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let needle = match query.keyword.as_deref() {
            Some("recent") => query.argument.to_lowercase(),
            None if query.raw.trim().is_empty() => String::new(),
            _ => return Vec::new(),
        };

        // Only show on empty query or explicit `recent` keyword.
        if query.keyword.is_none() && !query.raw.trim().is_empty() {
            return Vec::new();
        }

        self.recent()
            .into_iter()
            .filter(|p| {
                needle.is_empty()
                    || p.display().to_string().to_lowercase().contains(&needle)
                    || p.file_name()
                        .map(|n| n.to_string_lossy().to_lowercase().contains(&needle))
                        .unwrap_or(false)
            })
            .take(20)
            .map(|path| {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string());
                ResultItem::new(
                    format!("recent:{}", path.display()),
                    name,
                    ItemKind::Recent,
                )
                .with_subtitle(path.display().to_string())
                .with_path(path.clone())
                .with_score(3_000)
                .with_actions(file_actions(&path))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_xbel_href() {
        let xml = r#"<bookmark href="file:///home/user/Doc%20uments/a.txt"/>"#;
        let paths = parse_xbel(xml);
        assert_eq!(paths[0], PathBuf::from("/home/user/Doc uments/a.txt"));
    }
}
