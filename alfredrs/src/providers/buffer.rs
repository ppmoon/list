//! File buffer — accumulate files then act on all at once.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use crate::ranking::{data_dir, ensure_data_dir};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileBuffer {
    pub paths: Vec<PathBuf>,
}

impl FileBuffer {
    pub fn path() -> PathBuf {
        data_dir().join("buffer.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        ensure_data_dir()?;
        std::fs::write(Self::path(), serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn add(&mut self, path: PathBuf) {
        if !self.paths.contains(&path) {
            self.paths.push(path);
        }
    }

    pub fn clear(&mut self) {
        self.paths.clear();
    }
}

#[derive(Default)]
pub struct BufferProvider {
    cache: Mutex<FileBuffer>,
}

impl BufferProvider {
    pub fn add_path(path: PathBuf) -> anyhow::Result<()> {
        let mut buf = FileBuffer::load();
        buf.add(path);
        buf.save()
    }
}

impl Provider for BufferProvider {
    fn name(&self) -> &'static str {
        "buffer"
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        if query.keyword.as_deref() != Some("buf") {
            return Vec::new();
        }
        let buf = FileBuffer::load();
        *self.cache.lock().unwrap() = buf.clone();

        let mut items = Vec::new();
        items.push(
            ResultItem::new(
                "buf:summary",
                format!("{} file(s) in buffer", buf.paths.len()),
                ItemKind::Buffer,
            )
            .with_subtitle("Use actions to copy paths or clear")
            .with_score(9_000)
            .with_actions(vec![
                Action::CopyText(
                    buf.paths
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
                Action::RunCommand {
                    program: "alfredrs".into(),
                    args: vec!["buffer".into(), "clear".into()],
                },
            ]),
        );

        let arg = query.argument.to_lowercase();
        for path in buf.paths {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            if !arg.is_empty() && !name.to_lowercase().contains(&arg) {
                continue;
            }
            items.push(
                ResultItem::new(format!("buf:{}", path.display()), name, ItemKind::Buffer)
                    .with_subtitle(path.display().to_string())
                    .with_path(path.clone())
                    .with_score(8_000)
                    .with_actions(vec![
                        Action::OpenPath(path.clone()),
                        Action::CopyText(path.display().to_string()),
                    ]),
            );
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_dedupes() {
        let mut b = FileBuffer::default();
        b.add(PathBuf::from("/tmp/a"));
        b.add(PathBuf::from("/tmp/a"));
        assert_eq!(b.paths.len(), 1);
    }
}
