//! End-to-end feature matrix — definition of done for Alfred parity.

use alfredrs::config::{Config, Theme};
use alfredrs::model::{Action, Query};
use alfredrs::providers::apps::{clean_exec, parse_desktop_file};
use alfredrs::providers::bookmarks::parse_netscape_bookmarks;
use alfredrs::providers::buffer::FileBuffer;
use alfredrs::providers::calculator::{evaluate, looks_like_math};
use alfredrs::providers::contacts::parse_vcf;
use alfredrs::providers::dictionary::DictionaryProvider;
use alfredrs::providers::files::{preview_text, FilesProvider};
use alfredrs::providers::snippets::SnippetsProvider;
use alfredrs::providers::workflows::{Workflow, WorkflowNode, WorkflowProvider};
use alfredrs::providers::ProviderSet;
use alfredrs::ranking::{Ranker, UsageStats};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn search(q: &str) -> Vec<alfredrs::ResultItem> {
    let config = Config::default();
    let usage = UsageStats::default();
    let providers = ProviderSet::builtin();
    let ranker = Ranker::new();
    providers.search(&Query::parse(q), &config, &usage, &ranker)
}

#[test]
fn calculator_feature() {
    assert_eq!(evaluate("6*7"), Some(42.0));
    assert!(looks_like_math("1+2"));
    let results = search("= 6*7");
    assert_eq!(results[0].title, "42");
}

#[test]
fn web_search_feature() {
    let results = search("g alfredapp");
    assert!(results.iter().any(|r| r.subtitle.contains("google.com")));
}

#[test]
fn shell_feature() {
    let results = search("> echo hi");
    assert!(results.iter().any(|r| r.title.contains("echo hi")));
}

#[test]
fn system_feature() {
    let results = search("sleep");
    assert!(results.iter().any(|r| r.title == "Sleep"));
}

#[test]
fn dictionary_feature() {
    let d = DictionaryProvider::default();
    assert!(d.define("workflow").is_some());
    let results = search("spell clipbord");
    assert!(results.iter().any(|r| r.title == "clipboard"));
}

#[test]
fn large_type_feature() {
    let results = search("large 123-456");
    assert!(matches!(
        results[0].primary_action(),
        Some(Action::ShowLargeType(_))
    ));
}

#[test]
fn snippets_feature() {
    let snips = SnippetsProvider::load();
    let hit = SnippetsProvider::auto_expand(";lorem", &snips).unwrap();
    assert!(hit.1.contains("Lorem"));
    let results = search("snip lorem");
    assert!(!results.is_empty());
}

#[test]
fn workflow_feature() {
    let wf = Workflow {
        id: "echo".into(),
        name: "Echo".into(),
        description: "echo".into(),
        keyword: "echo".into(),
        nodes: vec![
            WorkflowNode::Keyword {
                keyword: "echo".into(),
            },
            WorkflowNode::Copy {
                text: "ping {arg}".into(),
            },
        ],
    };
    let actions = WorkflowProvider::run(&wf, "pong").unwrap();
    assert_eq!(actions, vec![Action::CopyText("ping pong".into())]);
}

#[test]
fn file_search_feature() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("alfredrs-feature-probe.txt");
    fs::write(&file, "hello preview").unwrap();
    let hits =
        FilesProvider::search_files(&[dir.path().to_path_buf()], "alfredrs-feature-probe", 5);
    assert_eq!(hits.len(), 1);
    assert!(preview_text(&file, 32).unwrap().contains("hello"));
}

#[test]
fn apps_desktop_parse_feature() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("forge.desktop");
    let mut f = fs::File::create(&path).unwrap();
    writeln!(
        f,
        "[Desktop Entry]\nName=Forge\nExec=forge --flag %U\nType=Application\n"
    )
    .unwrap();
    let app = parse_desktop_file(&path).unwrap();
    assert_eq!(app.name, "Forge");
    assert_eq!(clean_exec(&app.exec).0, "forge");
}

#[test]
fn bookmarks_feature() {
    let html = r#"<DT><A HREF="https://www.alfredapp.com">Alfred</A>"#;
    let bms = parse_netscape_bookmarks(html);
    assert_eq!(bms[0].url, "https://www.alfredapp.com");
}

#[test]
fn contacts_feature() {
    let contacts = parse_vcf("BEGIN:VCARD\nFN:Test\nEMAIL:a@b.c\nEND:VCARD\n");
    assert_eq!(contacts[0].email.as_deref(), Some("a@b.c"));
}

#[test]
fn buffer_feature() {
    let mut buf = FileBuffer::default();
    buf.add(PathBuf::from("/tmp/one"));
    buf.add(PathBuf::from("/tmp/two"));
    assert_eq!(buf.paths.len(), 2);
}

#[test]
fn theme_and_sync_feature() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("pack.json");
    let cfg = Config {
        theme: Theme {
            name: "Test Theme".into(),
            ..Theme::default()
        },
        ..Config::default()
    };
    let usage = UsageStats::default();
    cfg.export_sync_pack(&usage, &dest).unwrap();
    assert!(dest.exists());
    let text = fs::read_to_string(dest).unwrap();
    assert!(text.contains("Test Theme"));
}

#[test]
fn fallback_feature() {
    let results = search("unlikely-unique-query-xyz");
    assert!(results.iter().any(|r| {
        matches!(r.kind, alfredrs::ItemKind::Fallback) || r.subtitle.contains("google.com")
    }));
}

#[test]
fn ranking_learns_usage() {
    let ranker = Ranker::new();
    let mut usage = UsageStats::default();
    usage.record("app:fav");
    let items = vec![
        alfredrs::ResultItem::new("app:other", "Other", alfredrs::ItemKind::App),
        alfredrs::ResultItem::new("app:fav", "Favourite", alfredrs::ItemKind::App),
    ];
    // Both match "f" loosely; usage should tip favourite.
    let ranked = ranker.rank("f", items, &usage);
    assert_eq!(ranked[0].id, "app:fav");
}
