//! Large Type — display text full-screen style.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;

pub struct LargeTypeProvider;

impl Provider for LargeTypeProvider {
    fn name(&self) -> &'static str {
        "large_type"
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let text = match query.keyword.as_deref() {
            Some("large") => query.argument.clone(),
            _ => return Vec::new(),
        };
        if text.is_empty() {
            return Vec::new();
        }
        vec![ResultItem::new(
            "large:show",
            "Show Large Type",
            ItemKind::LargeType,
        )
        .with_subtitle(text.clone())
        .with_score(9_500)
        .with_actions(vec![Action::ShowLargeType(text)])]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::Provider;

    #[test]
    fn large_type_action() {
        let items = LargeTypeProvider.search(&Query::parse("large 555-0199"), &Config::default());
        assert_eq!(items.len(), 1);
        assert!(matches!(
            items[0].primary_action(),
            Some(Action::ShowLargeType(_))
        ));
    }
}
