//! Usage stats — Alfred-style addiction graph data.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use crate::paths::data_dir;
use crate::ranking::UsageStats;

pub struct StatsProvider;

impl StatsProvider {
    pub fn load() -> UsageStats {
        let path = data_dir().join("usage.json");
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(stats: &UsageStats) -> anyhow::Result<()> {
        crate::paths::ensure_data_dir()?;
        std::fs::write(
            data_dir().join("usage.json"),
            serde_json::to_string_pretty(stats)?,
        )?;
        Ok(())
    }
}

impl Provider for StatsProvider {
    fn name(&self) -> &'static str {
        "stats"
    }

    fn keywords(&self) -> &[&'static str] {
        &["stats"]
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        if query.keyword.as_deref() != Some("stats") {
            return Vec::new();
        }
        let stats = Self::load();
        let mut top: Vec<_> = stats.counts.iter().collect();
        top.sort_by(|a, b| b.1.cmp(a.1));
        let mut items = vec![ResultItem::new(
            "stats:total",
            format!("{} total launches", stats.total_launches),
            ItemKind::Stats,
        )
        .with_subtitle("Usage statistics")
        .with_score(9_000)
        .with_actions(vec![Action::ShowLargeType(format!(
            "{} launches",
            stats.total_launches
        ))])];

        for (id, count) in top.into_iter().take(15) {
            items.push(
                ResultItem::new(format!("stats:{id}"), id.clone(), ItemKind::Stats)
                    .with_subtitle(format!("{count} times"))
                    .with_score(8_000 + *count as i64)
                    .with_actions(vec![Action::CopyText(format!("{id}: {count}"))]),
            );
        }
        items
    }
}
