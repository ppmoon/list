//! Spell check + define — lightweight built-in dictionary.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use std::collections::HashMap;

pub struct DictionaryProvider {
    words: Vec<&'static str>,
    definitions: HashMap<&'static str, &'static str>,
}

impl Default for DictionaryProvider {
    fn default() -> Self {
        let mut definitions = HashMap::new();
        definitions.insert("alfred", "A productivity application for macOS; also this project's inspiration.");
        definitions.insert("launcher", "A utility that starts programs or opens files quickly.");
        definitions.insert("workflow", "A chained set of triggers and actions that automate a task.");
        definitions.insert("snippet", "A short reusable piece of text expanded by abbreviation.");
        definitions.insert("clipboard", "The system buffer that holds cut or copied data.");
        definitions.insert("fuzzy", "Approximate matching that tolerates typos and partial input.");
        definitions.insert("hotkey", "A keyboard shortcut that triggers an action.");
        definitions.insert("theme", "A visual style defining colours, fonts, and sizes.");
        definitions.insert("rust", "A systems programming language focused on safety and performance.");
        definitions.insert("linux", "A family of open-source Unix-like operating systems.");

        let words = vec![
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "i", "it", "for", "not",
            "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from",
            "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would",
            "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which",
            "go", "me", "when", "make", "can", "like", "time", "no", "just", "him", "know",
            "take", "people", "into", "year", "your", "good", "some", "could", "them", "see",
            "other", "than", "then", "now", "look", "only", "come", "its", "over", "think",
            "also", "back", "after", "use", "two", "how", "our", "work", "first", "well",
            "way", "even", "new", "want", "because", "any", "these", "give", "day", "most",
            "us", "alfred", "launcher", "workflow", "snippet", "clipboard", "fuzzy", "hotkey",
            "theme", "rust", "linux", "search", "file", "application", "system", "terminal",
            "command", "buffer", "preview", "bookmark", "contact", "music", "calculator",
            "dictionary", "preferences", "sync", "action", "result", "query", "provider",
        ];
        Self { words, definitions }
    }
}

impl DictionaryProvider {
    pub fn suggestions(&self, word: &str, limit: usize) -> Vec<String> {
        let word = word.to_lowercase();
        if word.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(i64, &str)> = self
            .words
            .iter()
            .filter_map(|w| {
                let dist = edit_distance(&word, w);
                if dist <= 2 || w.starts_with(&word) {
                    Some((-(dist as i64), *w))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by_key(|(s, w)| (*s, *w));
        scored.dedup_by(|a, b| a.1 == b.1);
        scored.into_iter().take(limit).map(|(_, w)| w.to_string()).collect()
    }

    pub fn define(&self, word: &str) -> Option<&'static str> {
        self.definitions.get(word.to_lowercase().as_str()).copied()
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

impl Provider for DictionaryProvider {
    fn name(&self) -> &'static str {
        "dictionary"
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        let (mode, word) = match query.keyword.as_deref() {
            Some("define") => ("define", query.argument.as_str()),
            Some("spell") => ("spell", query.argument.as_str()),
            _ => return Vec::new(),
        };
        if word.is_empty() {
            return Vec::new();
        }

        if mode == "define" {
            if let Some(def) = self.define(word) {
                return vec![ResultItem::new(
                    format!("def:{word}"),
                    word.to_string(),
                    ItemKind::Dictionary,
                )
                .with_subtitle(def)
                .with_score(8_000)
                .with_actions(vec![
                    Action::CopyText(def.to_string()),
                    Action::ShowLargeType(format!("{word}: {def}")),
                ])];
            }
        }

        self.suggestions(word, 8)
            .into_iter()
            .map(|w| {
                let sub = self
                    .define(&w)
                    .unwrap_or("Spelling suggestion")
                    .to_string();
                ResultItem::new(format!("spell:{w}"), w.clone(), ItemKind::Dictionary)
                    .with_subtitle(sub.clone())
                    .with_score(4_000)
                    .with_actions(vec![Action::CopyText(w)])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_close_words() {
        let d = DictionaryProvider::default();
        let s = d.suggestions("clipbord", 5);
        assert!(s.iter().any(|w| w == "clipboard"));
    }

    #[test]
    fn defines_known_word() {
        let d = DictionaryProvider::default();
        assert!(d.define("rust").is_some());
    }
}
