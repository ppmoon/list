//! Query engine: dispatch providers, execute actions, maintain state.

use crate::config::Config;
use crate::model::{Action, Query, ResultItem};
use crate::providers::actions::action_items_for_selection;
use crate::providers::buffer::BufferProvider;
use crate::providers::clipboard::ClipboardProvider;
use crate::providers::stats::StatsProvider;
use crate::providers::workflows::WorkflowProvider;
use crate::providers::ProviderSet;
use crate::ranking::{Ranker, UsageStats};
use anyhow::Context;
use std::process::Command;

pub struct Engine {
    pub config: Config,
    pub usage: UsageStats,
    pub providers: ProviderSet,
    pub ranker: Ranker,
    pub results: Vec<ResultItem>,
    pub selected: usize,
    pub query: String,
    pub large_type: Option<String>,
    pub actions_mode: bool,
    pub action_source: Option<ResultItem>,
}

impl Engine {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load_or_default()?;
        let usage = StatsProvider::load();
        let _ = WorkflowProvider::ensure_examples();
        Ok(Self {
            config,
            usage,
            providers: ProviderSet::builtin(),
            ranker: Ranker::new(),
            results: Vec::new(),
            selected: 0,
            query: String::new(),
            large_type: None,
            actions_mode: false,
            action_source: None,
        })
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.actions_mode = false;
        self.action_source = None;
        self.refresh();
    }

    pub fn refresh(&mut self) {
        let parsed = Query::parse(&self.query);
        self.results = self
            .providers
            .search(&parsed, &self.config, &self.usage, &self.ranker);
        if self.selected >= self.results.len() {
            self.selected = 0;
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.results.is_empty() {
            return;
        }
        let len = self.results.len() as isize;
        let next = (self.selected as isize + delta).rem_euclid(len);
        self.selected = next as usize;
    }

    pub fn current(&self) -> Option<&ResultItem> {
        self.results.get(self.selected)
    }

    pub fn enter_actions_mode(&mut self) {
        let Some(item) = self.current().cloned() else {
            return;
        };
        self.action_source = Some(item.clone());
        self.results = action_items_for_selection(&item);
        self.selected = 0;
        self.actions_mode = true;
    }

    pub fn exit_actions_mode(&mut self) {
        if self.actions_mode {
            self.actions_mode = false;
            self.action_source = None;
            self.refresh();
        }
    }

    pub fn activate(&mut self) -> anyhow::Result<()> {
        let Some(item) = self.current().cloned() else {
            return Ok(());
        };
        self.usage.record(&item.id);
        let _ = StatsProvider::save(&self.usage);
        if let Some(action) = item.primary_action().cloned() {
            self.execute(action)?;
        }
        Ok(())
    }

    pub fn execute(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::OpenPath(path) => {
                open::that(&path).with_context(|| format!("open {}", path.display()))?;
            }
            Action::OpenUrl(url) => {
                open::that(&url).with_context(|| format!("open url {url}"))?;
            }
            Action::RunCommand { program, args } => {
                Command::new(&program)
                    .args(&args)
                    .spawn()
                    .with_context(|| format!("run {program}"))?;
            }
            Action::CopyText(text) | Action::PasteText(text) | Action::ExpandSnippet(text) => {
                let mut clipboard = arboard::Clipboard::new()?;
                clipboard.set_text(&text)?;
                let _ = ClipboardProvider::push_text(&text, self.config.clipboard_max_items);
            }
            Action::Reveal(path) => {
                // xdg-open parent directory
                let parent = path.parent().unwrap_or(&path);
                open::that(parent)?;
            }
            Action::ShowLargeType(text) => {
                self.large_type = Some(text);
            }
            Action::AddToBuffer(path) => {
                BufferProvider::add_path(path)?;
            }
            Action::RunWorkflow { workflow_id, arg } => {
                let workflows = WorkflowProvider::load_all();
                let wf = workflows
                    .iter()
                    .find(|w| w.id == workflow_id)
                    .with_context(|| format!("workflow {workflow_id}"))?;
                let actions = WorkflowProvider::run(wf, &arg)?;
                for a in actions {
                    self.execute(a)?;
                }
            }
            Action::Noop => {}
        }
        Ok(())
    }

    pub fn capture_clipboard(&self) -> anyhow::Result<()> {
        let _ = ClipboardProvider::capture_current(self.config.clipboard_max_items)?;
        Ok(())
    }

    pub fn export_sync(&self, dest: &std::path::Path) -> anyhow::Result<()> {
        self.config.export_sync_pack(&self.usage, dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculator_query_ranks_first() {
        let mut engine = Engine {
            config: Config::default(),
            usage: UsageStats::default(),
            providers: ProviderSet::builtin(),
            ranker: Ranker::new(),
            results: Vec::new(),
            selected: 0,
            query: String::new(),
            large_type: None,
            actions_mode: false,
            action_source: None,
        };
        engine.set_query("= 21*2");
        assert!(!engine.results.is_empty());
        assert_eq!(engine.results[0].title, "42");
    }
}
