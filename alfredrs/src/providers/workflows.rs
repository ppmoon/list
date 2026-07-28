//! JSON workflow engine — Alfred Powerpack Workflows (simplified).

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use crate::paths::{data_dir, ensure_data_dir};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keyword: String,
    pub nodes: Vec<WorkflowNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowNode {
    Keyword { keyword: String },
    Script { command: String },
    OpenUrl { url: String },
    Copy { text: String },
    ListFilter { items: Vec<FilterItem> },
    ShowLargeType { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterItem {
    pub title: String,
    pub arg: String,
}

#[derive(Default)]
pub struct WorkflowProvider {
    cache: Mutex<Option<Vec<Workflow>>>,
}

impl WorkflowProvider {
    pub fn dir() -> PathBuf {
        data_dir().join("workflows")
    }

    pub fn ensure_examples() -> anyhow::Result<()> {
        ensure_data_dir()?;
        let dir = Self::dir();
        std::fs::create_dir_all(&dir)?;
        let example = dir.join("date-iso.json");
        if !example.exists() {
            let wf = Workflow {
                id: "date-iso".into(),
                name: "ISO Date".into(),
                description: "Copy today's date in ISO format".into(),
                keyword: "date".into(),
                nodes: vec![
                    WorkflowNode::Keyword {
                        keyword: "date".into(),
                    },
                    WorkflowNode::Script {
                        command: "date -I".into(),
                    },
                    WorkflowNode::Copy {
                        text: "{output}".into(),
                    },
                ],
            };
            std::fs::write(example, serde_json::to_string_pretty(&wf)?)?;
        }
        let greet = dir.join("hello.json");
        if !greet.exists() {
            let wf = Workflow {
                id: "hello".into(),
                name: "Hello".into(),
                description: "Greet someone in Large Type".into(),
                keyword: "hello".into(),
                nodes: vec![
                    WorkflowNode::Keyword {
                        keyword: "hello".into(),
                    },
                    WorkflowNode::ListFilter {
                        items: vec![
                            FilterItem {
                                title: "World".into(),
                                arg: "World".into(),
                            },
                            FilterItem {
                                title: "Alfred".into(),
                                arg: "Alfred".into(),
                            },
                        ],
                    },
                    WorkflowNode::ShowLargeType {
                        text: "Hello, {arg}!".into(),
                    },
                ],
            };
            std::fs::write(greet, serde_json::to_string_pretty(&wf)?)?;
        }
        Ok(())
    }

    pub fn load_all() -> Vec<Workflow> {
        let _ = Self::ensure_examples();
        let dir = Self::dir();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Ok(wf) = serde_json::from_str::<Workflow>(&text) {
                    out.push(wf);
                }
            }
        }
        out
    }

    /// Execute a workflow with an optional argument; returns final actionable outputs.
    pub fn run(workflow: &Workflow, arg: &str) -> anyhow::Result<Vec<Action>> {
        let mut output = String::new();
        let mut current_arg = arg.to_string();
        let mut actions = Vec::new();

        for node in &workflow.nodes {
            match node {
                WorkflowNode::Keyword { .. } => {}
                WorkflowNode::Script { command } => {
                    let cmd = command
                        .replace("{arg}", &current_arg)
                        .replace("{query}", &current_arg);
                    let result = Command::new("bash").args(["-lc", &cmd]).output()?;
                    output = String::from_utf8_lossy(&result.stdout).trim().to_string();
                    current_arg = output.clone();
                }
                WorkflowNode::OpenUrl { url } => {
                    let url = url
                        .replace("{arg}", &current_arg)
                        .replace("{query}", &current_arg)
                        .replace("{output}", &output);
                    actions.push(Action::OpenUrl(url));
                }
                WorkflowNode::Copy { text } => {
                    let text = text
                        .replace("{arg}", &current_arg)
                        .replace("{query}", &current_arg)
                        .replace("{output}", &output);
                    actions.push(Action::CopyText(text));
                }
                WorkflowNode::ListFilter { items } => {
                    if current_arg.is_empty() {
                        // Surface choices as no-op — engine will list them separately.
                        return Ok(items
                            .iter()
                            .map(|i| Action::RunWorkflow {
                                workflow_id: workflow.id.clone(),
                                arg: i.arg.clone(),
                            })
                            .collect());
                    }
                }
                WorkflowNode::ShowLargeType { text } => {
                    let text = text
                        .replace("{arg}", &current_arg)
                        .replace("{query}", &current_arg)
                        .replace("{output}", &output);
                    actions.push(Action::ShowLargeType(text));
                }
            }
        }
        Ok(actions)
    }

    fn workflows(&self) -> Vec<Workflow> {
        let mut guard = self.cache.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Self::load_all());
        }
        guard.clone().unwrap_or_default()
    }
}

impl Provider for WorkflowProvider {
    fn name(&self) -> &'static str {
        "workflows"
    }

    fn keywords(&self) -> &[&'static str] {
        &["workflow", "wf"]
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let (filter_kw, arg) = match query.keyword.as_deref() {
            Some("workflow") | Some("wf") => (None, query.argument.as_str()),
            Some(kw) => (Some(kw), query.argument.as_str()),
            None => return Vec::new(),
        };

        let mut items = Vec::new();
        for wf in self.workflows() {
            let matches_meta = filter_kw.is_none()
                && (arg.is_empty()
                    || wf.name.to_lowercase().contains(&arg.to_lowercase())
                    || wf.keyword.contains(arg));
            let matches_kw = filter_kw == Some(wf.keyword.as_str());
            if !(matches_meta || matches_kw) {
                continue;
            }

            // List-filter expansion when keyword matches and no arg yet.
            if matches_kw && arg.is_empty() {
                for node in &wf.nodes {
                    if let WorkflowNode::ListFilter { items: choices } = node {
                        for choice in choices {
                            items.push(
                                ResultItem::new(
                                    format!("wf:{}:{}", wf.id, choice.arg),
                                    choice.title.clone(),
                                    ItemKind::Workflow,
                                )
                                .with_subtitle(wf.name.clone())
                                .with_score(9_000)
                                .with_actions(vec![Action::RunWorkflow {
                                    workflow_id: wf.id.clone(),
                                    arg: choice.arg.clone(),
                                }]),
                            );
                        }
                    }
                }
            }

            items.push(
                ResultItem::new(format!("wf:{}", wf.id), wf.name.clone(), ItemKind::Workflow)
                    .with_subtitle(format!("{} — {}", wf.keyword, wf.description))
                    .with_score(8_500)
                    .with_actions(vec![Action::RunWorkflow {
                        workflow_id: wf.id.clone(),
                        arg: arg.to_string(),
                    }]),
            );
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_copy_workflow() {
        let wf = Workflow {
            id: "t".into(),
            name: "T".into(),
            description: "".into(),
            keyword: "t".into(),
            nodes: vec![
                WorkflowNode::Keyword {
                    keyword: "t".into(),
                },
                WorkflowNode::Copy {
                    text: "hi {arg}".into(),
                },
            ],
        };
        let actions = WorkflowProvider::run(&wf, "there").unwrap();
        assert_eq!(actions, vec![Action::CopyText("hi there".into())]);
    }
}
