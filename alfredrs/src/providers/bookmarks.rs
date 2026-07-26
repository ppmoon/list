//! Browser bookmark search (Chrome / Firefox).

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Default)]
pub struct BookmarksProvider {
    cache: Mutex<Option<Vec<Bookmark>>>,
}

impl BookmarksProvider {
    pub fn load() -> Vec<Bookmark> {
        let mut out = Vec::new();
        out.extend(load_chrome());
        out.extend(load_firefox());
        out
    }

    fn bookmarks(&self) -> Vec<Bookmark> {
        let mut guard = self.cache.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Self::load());
        }
        guard.clone().unwrap_or_default()
    }
}

fn load_chrome() -> Vec<Bookmark> {
    let mut paths = Vec::new();
    if let Some(config) = dirs::config_dir() {
        paths.push(config.join("google-chrome/Default/Bookmarks"));
        paths.push(config.join("chromium/Default/Bookmarks"));
        paths.push(config.join("BraveSoftware/Brave-Browser/Default/Bookmarks"));
    }
    let mut out = Vec::new();
    for path in paths {
        if let Ok(text) = std::fs::read_to_string(path) {
            if let Ok(root) = serde_json::from_str::<ChromeBookmarks>(&text) {
                collect_chrome(&root.roots.bookmark_bar, &mut out);
                collect_chrome(&root.roots.other, &mut out);
            }
        }
    }
    out
}

#[derive(Deserialize)]
struct ChromeBookmarks {
    roots: ChromeRoots,
}

#[derive(Deserialize)]
struct ChromeRoots {
    bookmark_bar: ChromeNode,
    other: ChromeNode,
}

#[derive(Deserialize)]
struct ChromeNode {
    #[serde(default)]
    children: Vec<ChromeNode>,
    #[serde(default)]
    name: String,
    #[serde(default)]
    url: String,
    #[serde(rename = "type")]
    #[serde(default)]
    node_type: String,
}

fn collect_chrome(node: &ChromeNode, out: &mut Vec<Bookmark>) {
    if node.node_type == "url" || (!node.url.is_empty() && node.children.is_empty()) {
        out.push(Bookmark {
            title: node.name.clone(),
            url: node.url.clone(),
        });
    }
    for child in &node.children {
        collect_chrome(child, out);
    }
}

fn load_firefox() -> Vec<Bookmark> {
    // Firefox stores bookmarks in places.sqlite — without sqlite dep, scan
    // exported HTML bookmarks if present.
    let mut out = Vec::new();
    let Some(home) = dirs::home_dir() else {
        return out;
    };
    let html = home.join(".mozilla/firefox/bookmarks.html");
    if let Ok(text) = std::fs::read_to_string(html) {
        out.extend(parse_netscape_bookmarks(&text));
    }
    // Also accept a user-exported file in alfredrs data dir.
    let exported = crate::ranking::data_dir().join("bookmarks.html");
    if let Ok(text) = std::fs::read_to_string(exported) {
        out.extend(parse_netscape_bookmarks(&text));
    }
    let _ = PathBuf::new();
    out
}

pub fn parse_netscape_bookmarks(html: &str) -> Vec<Bookmark> {
    let mut out = Vec::new();
    for line in html.lines() {
        let lower = line.to_lowercase();
        let Some(anchor_at) = lower.find("<a ") else {
            continue;
        };
        let lower_from = &lower[anchor_at..];
        let line_from = &line[anchor_at..];
        let Some(href_rel) = lower_from.find("href=\"") else {
            continue;
        };
        let after_href = &line_from[href_rel + 6..];
        let Some(url_end) = after_href.find('"') else {
            continue;
        };
        let url = after_href[..url_end].to_string();
        let Some(title_rel) = after_href.find('>') else {
            continue;
        };
        let after_gt = &after_href[title_rel + 1..];
        let close = after_gt.to_lowercase().find("</a>").unwrap_or(after_gt.len());
        let title = after_gt[..close].to_string();
        if !url.is_empty() {
            out.push(Bookmark { title, url });
        }
    }
    out
}

impl Provider for BookmarksProvider {
    fn name(&self) -> &'static str {
        "bookmarks"
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let needle = match query.keyword.as_deref() {
            Some("bm") | Some("bookmark") => query.argument.to_lowercase(),
            _ => return Vec::new(),
        };
        self.bookmarks()
            .into_iter()
            .filter(|b| {
                needle.is_empty()
                    || b.title.to_lowercase().contains(&needle)
                    || b.url.to_lowercase().contains(&needle)
            })
            .take(40)
            .map(|b| {
                ResultItem::new(
                    format!("bm:{}", b.url),
                    b.title,
                    ItemKind::Bookmark,
                )
                .with_subtitle(b.url.clone())
                .with_score(6_500)
                .with_actions(vec![
                    Action::OpenUrl(b.url.clone()),
                    Action::CopyText(b.url),
                ])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_netscape_html() {
        let html = r#"<DT><A HREF="https://example.com">Example</A>"#;
        let bms = parse_netscape_bookmarks(html);
        assert_eq!(bms.len(), 1);
        assert_eq!(bms[0].title, "Example");
    }
}
