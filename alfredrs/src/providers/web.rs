//! Default + custom web searches.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;

pub struct WebProvider;

impl Provider for WebProvider {
    fn name(&self) -> &'static str {
        "web"
    }

    fn search(&self, query: &Query, config: &Config) -> Vec<ResultItem> {
        let Some(keyword) = query.keyword.as_deref() else {
            return Vec::new();
        };
        if query.argument.is_empty() {
            return Vec::new();
        }
        config
            .web_searches
            .iter()
            .filter(|s| s.keyword.eq_ignore_ascii_case(keyword))
            .map(|s| {
                let url = s.render(&query.argument);
                ResultItem::new(
                    format!("web:{}:{}", s.keyword, query.argument),
                    format!("Search {}: {}", s.title, query.argument),
                    ItemKind::Web,
                )
                .with_subtitle(url.clone())
                .with_score(5_000)
                .with_actions(vec![Action::OpenUrl(url)])
            })
            .collect()
    }
}

/// Fallback searches when nothing else matched well — Alfred default results fallback.
pub fn fallback_results(query: &Query, config: &Config) -> Vec<ResultItem> {
    if query.is_empty() || query.keyword.is_some() {
        return Vec::new();
    }
    let q = query.raw.trim();
    config
        .fallback_searches
        .iter()
        .filter_map(|kw| {
            config
                .web_searches
                .iter()
                .find(|s| s.keyword == *kw)
                .map(|s| {
                    let url = s.render(q);
                    ResultItem::new(
                        format!("fallback:{}:{q}", s.keyword),
                        format!("{} \"{q}\"", s.title),
                        ItemKind::Fallback,
                    )
                    .with_subtitle(url.clone())
                    .with_score(1)
                    .with_actions(vec![Action::OpenUrl(url)])
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Provider;

    #[test]
    fn google_keyword() {
        let cfg = Config::default();
        let q = Query::parse("g rust lang");
        let items = WebProvider.search(&q, &cfg);
        assert_eq!(items.len(), 1);
        assert!(items[0].subtitle.contains("google.com"));
    }
}
