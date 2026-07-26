//! Fuzzy matching + usage-learned ranking.

use crate::model::ResultItem;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    /// How many times each result id was selected.
    pub counts: HashMap<String, u64>,
    /// Total launches recorded.
    pub total_launches: u64,
}

impl UsageStats {
    pub fn record(&mut self, id: &str) {
        *self.counts.entry(id.to_string()).or_insert(0) += 1;
        self.total_launches += 1;
    }

    pub fn score_bonus(&self, id: &str) -> i64 {
        self.counts.get(id).copied().unwrap_or(0) as i64 * 25
    }
}

#[derive(Default)]
pub struct Ranker {
    matcher: SkimMatcherV2,
}

impl Ranker {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn score_text(&self, query: &str, text: &str) -> Option<i64> {
        if query.is_empty() {
            return Some(1);
        }
        self.matcher.fuzzy_match(text, query)
    }

    pub fn rank(
        &self,
        query: &str,
        mut items: Vec<ResultItem>,
        usage: &UsageStats,
    ) -> Vec<ResultItem> {
        for item in &mut items {
            let title_score = self.score_text(query, &item.title).unwrap_or(0);
            let sub_score = self.score_text(query, &item.subtitle).unwrap_or(0) / 2;
            let usage_bonus = usage.score_bonus(&item.id);
            item.score = item.score.max(title_score + sub_score) + usage_bonus;
        }
        items.retain(|i| query.is_empty() || i.score > 0);
        items.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.title.cmp(&b.title)));
        items
    }
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("alfredrs")
}

pub fn ensure_data_dir() -> anyhow::Result<PathBuf> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ItemKind;

    #[test]
    fn usage_boosts_familiar_items() {
        let ranker = Ranker::new();
        let mut usage = UsageStats::default();
        usage.record("app:vim");
        let items = vec![
            ResultItem::new("app:neovim", "Neovim", ItemKind::App),
            ResultItem::new("app:vim", "Vim", ItemKind::App),
        ];
        let ranked = ranker.rank("vim", items, &usage);
        assert_eq!(ranked[0].id, "app:vim");
        assert!(ranked.len() >= 2);
        assert!(ranked[0].score > ranked[1].score);
    }
}
