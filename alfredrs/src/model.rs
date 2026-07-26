//! Core domain types shared across providers and the UI.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A single searchable / actionable result shown in the launcher.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub kind: ItemKind,
    pub score: i64,
    pub actions: Vec<Action>,
    pub icon: Option<String>,
    pub path: Option<PathBuf>,
    pub payload: Option<String>,
}

impl ResultItem {
    pub fn new(id: impl Into<String>, title: impl Into<String>, kind: ItemKind) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: String::new(),
            kind,
            score: 0,
            actions: Vec::new(),
            icon: None,
            path: None,
            payload: None,
        }
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_score(mut self, score: i64) -> Self {
        self.score = score;
        self
    }

    pub fn with_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = actions;
        self
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_payload(mut self, payload: impl Into<String>) -> Self {
        self.payload = Some(payload.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn primary_action(&self) -> Option<&Action> {
        self.actions.first()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ItemKind {
    App,
    File,
    Web,
    Calculator,
    Dictionary,
    System,
    Shell,
    Clipboard,
    Snippet,
    Workflow,
    Bookmark,
    Contact,
    Music,
    Recent,
    LargeType,
    Action,
    Buffer,
    Stats,
    Fallback,
    Preview,
}

/// Something the user can do with a result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    OpenPath(PathBuf),
    OpenUrl(String),
    RunCommand { program: String, args: Vec<String> },
    CopyText(String),
    PasteText(String),
    Reveal(PathBuf),
    ShowLargeType(String),
    AddToBuffer(PathBuf),
    RunWorkflow { workflow_id: String, arg: String },
    ExpandSnippet(String),
    Noop,
}

#[derive(Debug, Clone, Default)]
pub struct Query {
    pub raw: String,
    pub keyword: Option<String>,
    pub argument: String,
}

impl Query {
    pub fn parse(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::default();
        }

        // Shell shortcut: "> ls -la"
        if let Some(rest) = trimmed.strip_prefix('>') {
            return Self {
                raw: trimmed.to_string(),
                keyword: Some(">".into()),
                argument: rest.trim().to_string(),
            };
        }

        // Calculator shortcut: "= 1+2" or bare math-looking
        if let Some(rest) = trimmed.strip_prefix('=') {
            return Self {
                raw: trimmed.to_string(),
                keyword: Some("=".into()),
                argument: rest.trim().to_string(),
            };
        }

        // Trailing space after a single token → keyword mode with empty arg
        // (Alfred-style: type `clip ` / `snip ` to open that feature).
        let trailing_space = raw.ends_with(|c: char| c.is_whitespace());
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let first = parts.next().unwrap_or("").to_string();
        let rest = parts.next().unwrap_or("").to_string();
        if rest.is_empty() {
            if trailing_space {
                Self {
                    raw: trimmed.to_string(),
                    keyword: Some(first),
                    argument: String::new(),
                }
            } else {
                Self {
                    raw: trimmed.to_string(),
                    keyword: None,
                    argument: first,
                }
            }
        } else {
            Self {
                raw: trimmed.to_string(),
                keyword: Some(first),
                argument: rest,
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.raw.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_keyword_and_argument() {
        let q = Query::parse("find notes.md");
        assert_eq!(q.keyword.as_deref(), Some("find"));
        assert_eq!(q.argument, "notes.md");
    }

    #[test]
    fn parses_shell_prefix() {
        let q = Query::parse("> echo hi");
        assert_eq!(q.keyword.as_deref(), Some(">"));
        assert_eq!(q.argument, "echo hi");
    }

    #[test]
    fn trailing_space_enters_keyword_mode() {
        let q = Query::parse("snip ");
        assert_eq!(q.keyword.as_deref(), Some("snip"));
        assert_eq!(q.argument, "");
    }

    #[test]
    fn parses_calculator_prefix() {
        let q = Query::parse("= 2*21");
        assert_eq!(q.keyword.as_deref(), Some("="));
        assert_eq!(q.argument, "2*21");
    }
}
