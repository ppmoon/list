//! Universal actions + fallback search provider.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::web::fallback_results;
use crate::providers::Provider;
use std::path::Path;

/// Actions that can be applied to files, URLs, or free text (Alfred Universal Actions).
pub fn universal_actions_for(text: &str) -> Vec<Action> {
    let mut actions = vec![
        Action::CopyText(text.to_string()),
        Action::ShowLargeType(text.to_string()),
    ];
    if text.starts_with("http://") || text.starts_with("https://") {
        actions.insert(0, Action::OpenUrl(text.to_string()));
    }
    let path = Path::new(text);
    if path.exists() {
        actions.insert(0, Action::OpenPath(path.to_path_buf()));
        actions.push(Action::Reveal(path.to_path_buf()));
        actions.push(Action::AddToBuffer(path.to_path_buf()));
    }
    actions
}

pub fn action_items_for_selection(item: &ResultItem) -> Vec<ResultItem> {
    item.actions
        .iter()
        .enumerate()
        .map(|(idx, action)| {
            let title = action_label(action);
            ResultItem::new(
                format!("action:{}:{idx}", item.id),
                title,
                ItemKind::Action,
            )
            .with_subtitle(item.title.clone())
            .with_score(10_000 - idx as i64)
            .with_actions(vec![action.clone()])
        })
        .collect()
}

pub fn action_label(action: &Action) -> String {
    match action {
        Action::OpenPath(_) => "Open".into(),
        Action::OpenUrl(_) => "Open URL".into(),
        Action::RunCommand { program, .. } => format!("Run {program}"),
        Action::CopyText(_) => "Copy to Clipboard".into(),
        Action::PasteText(_) => "Paste".into(),
        Action::Reveal(_) => "Reveal in File Manager".into(),
        Action::ShowLargeType(_) => "Large Type".into(),
        Action::AddToBuffer(_) => "Add to File Buffer".into(),
        Action::RunWorkflow { workflow_id, .. } => format!("Run Workflow {workflow_id}"),
        Action::ExpandSnippet(_) => "Expand Snippet".into(),
        Action::Noop => "Do Nothing".into(),
    }
}

pub struct FallbackProvider;

impl Provider for FallbackProvider {
    fn name(&self) -> &'static str {
        "fallback"
    }

    fn search(&self, query: &Query, config: &Config) -> Vec<ResultItem> {
        fallback_results(query, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn universal_actions_for_url() {
        let actions = universal_actions_for("https://example.com");
        assert!(matches!(actions[0], Action::OpenUrl(_)));
    }
}
