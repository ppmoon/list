//! Result providers — each mirrors an Alfred feature surface.

pub mod actions;
pub mod apps;
pub mod bookmarks;
pub mod buffer;
pub mod calculator;
pub mod clipboard;
pub mod contacts;
pub mod dictionary;
pub mod files;
pub mod large_type;
pub mod music;
pub mod recent;
pub mod shell;
pub mod snippets;
pub mod stats;
pub mod system;
pub mod web;
pub mod workflows;

use crate::config::Config;
use crate::model::{Query, ResultItem};
use crate::ranking::UsageStats;
use std::collections::HashSet;

pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;

    /// Keywords this provider exclusively owns (e.g. `clip`, `find`).
    /// When the query keyword is in this set, free-text providers like apps
    /// should stand down.
    fn keywords(&self) -> &[&'static str] {
        &[]
    }

    fn search(&self, query: &Query, config: &Config) -> Vec<ResultItem>;
}

pub struct ProviderSet {
    providers: Vec<Box<dyn Provider>>,
    reserved_keywords: HashSet<String>,
}

impl ProviderSet {
    pub fn builtin() -> Self {
        let providers: Vec<Box<dyn Provider>> = vec![
            Box::new(calculator::CalculatorProvider),
            Box::new(shell::ShellProvider),
            Box::new(system::SystemProvider),
            Box::new(web::WebProvider),
            Box::new(dictionary::DictionaryProvider::default()),
            Box::new(large_type::LargeTypeProvider),
            Box::new(clipboard::ClipboardProvider::default()),
            Box::new(snippets::SnippetsProvider::default()),
            Box::new(workflows::WorkflowProvider::default()),
            Box::new(bookmarks::BookmarksProvider::default()),
            Box::new(contacts::ContactsProvider::default()),
            Box::new(music::MusicProvider),
            Box::new(recent::RecentProvider::default()),
            Box::new(buffer::BufferProvider::default()),
            Box::new(stats::StatsProvider),
            Box::new(files::FilesProvider),
            Box::new(apps::AppsProvider::default()),
            Box::new(actions::FallbackProvider),
        ];
        let mut reserved_keywords = HashSet::new();
        for provider in &providers {
            for kw in provider.keywords() {
                reserved_keywords.insert((*kw).to_string());
            }
        }
        // Web search keywords come from config at query time; seed common defaults.
        for kw in [
            "google", "g", "duckduckgo", "ddg", "wiki", "gh", "yt", "maps", "amazon",
        ] {
            reserved_keywords.insert(kw.into());
        }
        Self {
            providers,
            reserved_keywords,
        }
    }

    pub fn is_reserved_keyword(&self, keyword: &str) -> bool {
        self.reserved_keywords.contains(keyword)
    }

    pub fn search(
        &self,
        query: &Query,
        config: &Config,
        usage: &UsageStats,
        ranker: &crate::ranking::Ranker,
    ) -> Vec<ResultItem> {
        // Dynamic web keywords from config.
        let mut reserved = self.reserved_keywords.clone();
        for search in &config.web_searches {
            reserved.insert(search.keyword.clone());
        }

        let mut items = Vec::new();
        for provider in &self.providers {
            if !config.provider_enabled(provider.name()) {
                continue;
            }
            // Free-text providers skip when another feature owns the keyword.
            if matches!(provider.name(), "apps" | "fallback") {
                if let Some(kw) = query.keyword.as_deref() {
                    if reserved.contains(kw) && !provider.keywords().contains(&kw) {
                        continue;
                    }
                }
            }
            items.extend(provider.search(query, config));
        }
        let rank_query = if query.keyword.is_some() {
            &query.argument
        } else {
            query.raw.trim()
        };
        ranker.rank(rank_query, items, usage)
    }
}
