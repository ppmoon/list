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

pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn search(&self, query: &Query, config: &Config) -> Vec<ResultItem>;
}

pub struct ProviderSet {
    providers: Vec<Box<dyn Provider>>,
}

impl ProviderSet {
    pub fn builtin() -> Self {
        Self {
            providers: vec![
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
            ],
        }
    }

    pub fn search(
        &self,
        query: &Query,
        config: &Config,
        usage: &UsageStats,
        ranker: &crate::ranking::Ranker,
    ) -> Vec<ResultItem> {
        let mut items = Vec::new();
        for provider in &self.providers {
            if !config.provider_enabled(provider.name()) {
                continue;
            }
            items.extend(provider.search(query, config));
        }
        // Prefer argument for ranking when a keyword is present.
        let rank_query = if query.keyword.is_some() {
            &query.argument
        } else {
            query.raw.trim()
        };
        ranker.rank(rank_query, items, usage)
    }
}
