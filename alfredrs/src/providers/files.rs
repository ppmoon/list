//! File search — Alfred "find" / "open" / "in" style.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct FilesProvider;

impl FilesProvider {
    pub fn search_files(roots: &[PathBuf], needle: &str, limit: usize) -> Vec<PathBuf> {
        if needle.is_empty() {
            return Vec::new();
        }
        let needle_l = needle.to_lowercase();
        let mut hits = Vec::new();
        for root in roots {
            if !root.exists() {
                continue;
            }
            for entry in WalkDir::new(root)
                .follow_links(false)
                .max_depth(6)
                .into_iter()
                .filter_entry(|e| !is_ignored(e.path()))
                .flatten()
            {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                if name.contains(&needle_l) {
                    hits.push(path.to_path_buf());
                    if hits.len() >= limit {
                        return hits;
                    }
                }
            }
        }
        hits
    }
}

fn is_ignored(path: &Path) -> bool {
    path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some(
                ".git"
                    | "node_modules"
                    | "target"
                    | ".cache"
                    | ".local"
                    | "__pycache__"
                    | ".Trash"
            )
        )
    })
}

pub fn file_actions(path: &Path) -> Vec<Action> {
    vec![
        Action::OpenPath(path.to_path_buf()),
        Action::Reveal(path.to_path_buf()),
        Action::CopyText(path.display().to_string()),
        Action::AddToBuffer(path.to_path_buf()),
        Action::ShowLargeType(path.display().to_string()),
    ]
}

pub fn preview_text(path: &Path, max_bytes: usize) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > 512_000 {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes
        .iter()
        .take(512)
        .any(|&b| b == 0 || (b < 9 && b != b'\n' && b != b'\t' && b != b'\r'))
    {
        return None;
    }
    let text = String::from_utf8_lossy(&bytes);
    Some(text.chars().take(max_bytes).collect())
}

impl Provider for FilesProvider {
    fn name(&self) -> &'static str {
        "files"
    }

    fn search(&self, query: &Query, config: &Config) -> Vec<ResultItem> {
        let (needle, force) = match query.keyword.as_deref() {
            Some("find") | Some("open") | Some("in") => (query.argument.as_str(), true),
            Some(_) => return Vec::new(),
            None => (query.raw.trim(), false),
        };
        if needle.is_empty() {
            return Vec::new();
        }
        // Free-text: only contribute file hits when query looks like a filename
        // (has a dot or slash) unless forced by keyword.
        if !force && !(needle.contains('.') || needle.contains('/') || needle.len() >= 3) {
            return Vec::new();
        }

        Self::search_files(&config.file_search_roots, needle, config.file_search_max_results)
            .into_iter()
            .map(|path| {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string());
                let preview = preview_text(&path, 120)
                    .map(|p| p.replace('\n', " "))
                    .unwrap_or_else(|| path.display().to_string());
                ResultItem::new(format!("file:{}", path.display()), name, ItemKind::File)
                    .with_subtitle(preview)
                    .with_path(path.clone())
                    .with_actions(file_actions(&path))
                    .with_score(if force { 40 } else { 10 })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("alfred-notes.txt");
        fs::write(&file, "hello").unwrap();
        let hits = FilesProvider::search_files(&[dir.path().to_path_buf()], "alfred-notes", 10);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn preview_skips_binary() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("bin.dat");
        fs::write(&file, [0u8, 1, 2, 3]).unwrap();
        assert!(preview_text(&file, 100).is_none());
    }
}
