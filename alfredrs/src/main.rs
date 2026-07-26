//! alfredrs CLI / GUI entrypoint.

use alfredrs::config::Config;
use alfredrs::engine::Engine;
use alfredrs::model::Query;
use alfredrs::providers::buffer::FileBuffer;
use alfredrs::providers::clipboard::ClipboardProvider;
use alfredrs::providers::stats::StatsProvider;
use alfredrs::providers::workflows::WorkflowProvider;
use alfredrs::providers::ProviderSet;
use alfredrs::ranking::Ranker;
use alfredrs::ui::run_launcher;
use anyhow::Context;
use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        run_launcher().map_err(|e| anyhow::anyhow!("launch GUI: {e}"))?;
        return Ok(());
    }

    let cmd = args.remove(0);
    match cmd.as_str() {
        "gui" => {
            run_launcher().map_err(|e| anyhow::anyhow!("launch GUI: {e}"))?;
        }
        "daemon" => {
            alfredrs::hotkey::run_daemon()?;
        }
        "search" => {
            let q = args.join(" ");
            let config = Config::load_or_default()?;
            let usage = StatsProvider::load();
            let providers = ProviderSet::builtin();
            let ranker = Ranker::new();
            let results = providers.search(&Query::parse(q), &config, &usage, &ranker);
            for (i, item) in results.iter().take(20).enumerate() {
                println!(
                    "{:>2}. [{}] {} — {}",
                    i + 1,
                    format!("{:?}", item.kind),
                    item.title,
                    item.subtitle
                );
            }
        }
        "run" => {
            let q = args.join(" ");
            let mut engine = Engine::new()?;
            engine.set_query(q);
            engine.activate()?;
            if let Some(text) = engine.large_type {
                println!("{text}");
            }
        }
        "clip" | "clipboard" => match args.first().map(String::as_str) {
            Some("capture") => {
                let cfg = Config::load_or_default()?;
                if let Some(text) = ClipboardProvider::capture_current(cfg.clipboard_max_items)? {
                    println!("Captured ({} bytes)", text.len());
                } else {
                    println!("Clipboard empty or unavailable");
                }
            }
            Some("list") | None => {
                for item in ClipboardProvider::load().into_iter().take(20) {
                    let preview: String = item.text.chars().take(80).collect();
                    println!("- ({}) {}", item.kind, preview);
                }
            }
            other => anyhow::bail!("unknown clipboard subcommand: {other:?}"),
        },
        "buffer" => match args.first().map(String::as_str) {
            Some("clear") => {
                let mut buf = FileBuffer::load();
                buf.clear();
                buf.save()?;
                println!("Buffer cleared");
            }
            Some("list") | None => {
                let buf = FileBuffer::load();
                for p in buf.paths {
                    println!("{}", p.display());
                }
            }
            Some("add") => {
                let path = PathBuf::from(args.get(1).context("path required")?);
                alfredrs::providers::buffer::BufferProvider::add_path(path)?;
                println!("Added");
            }
            other => anyhow::bail!("unknown buffer subcommand: {other:?}"),
        },
        "workflow" => match args.first().map(String::as_str) {
            Some("list") | None => {
                for wf in WorkflowProvider::load_all() {
                    println!("{} [{}] — {}", wf.name, wf.keyword, wf.description);
                }
            }
            Some("run") => {
                let id = args.get(1).context("workflow id required")?;
                let arg = args.get(2).cloned().unwrap_or_default();
                let workflows = WorkflowProvider::load_all();
                let wf = workflows
                    .iter()
                    .find(|w| w.id == *id || w.keyword == *id)
                    .context("workflow not found")?;
                let mut engine = Engine::new()?;
                for action in WorkflowProvider::run(wf, &arg)? {
                    engine.execute(action)?;
                }
                if let Some(text) = engine.large_type {
                    println!("{text}");
                }
            }
            other => anyhow::bail!("unknown workflow subcommand: {other:?}"),
        },
        "sync" => match args.first().map(String::as_str) {
            Some("export") => {
                let dest = PathBuf::from(args.get(1).context("destination path required")?);
                let engine = Engine::new()?;
                engine.export_sync(&dest)?;
                println!("Exported to {}", dest.display());
            }
            Some("import") => {
                let src = PathBuf::from(args.get(1).context("source path required")?);
                let (cfg, usage) = Config::import_sync_pack(&src)?;
                StatsProvider::save(&usage)?;
                println!("Imported theme {} and {} usage keys", cfg.theme.name, usage.counts.len());
            }
            other => anyhow::bail!("unknown sync subcommand: {other:?}"),
        },
        "stats" => {
            let stats = StatsProvider::load();
            println!("Total launches: {}", stats.total_launches);
            let mut top: Vec<_> = stats.counts.iter().collect();
            top.sort_by(|a, b| b.1.cmp(a.1));
            for (id, n) in top.into_iter().take(20) {
                println!("{n:>5}  {id}");
            }
        }
        "features" => {
            print_features();
        }
        "help" | "-h" | "--help" => print_help(),
        other => anyhow::bail!("unknown command: {other}. Try `alfredrs help`."),
    }
    Ok(())
}

fn print_help() {
    println!(
        "\
alfredrs — Alfred-inspired launcher for Linux

USAGE:
  alfredrs                 Open the launcher GUI
  alfredrs gui             Open the launcher GUI (same as no args)
  alfredrs daemon          Listen for configured hotkey and summon GUI
  alfredrs search <query>  Print ranked results
  alfredrs run <query>     Activate the top result
  alfredrs clip capture    Snapshot current clipboard into history
  alfredrs clip list       Show clipboard history
  alfredrs buffer list|add|clear
  alfredrs workflow list|run <id> [arg]
  alfredrs sync export|import <path>
  alfredrs stats
  alfredrs features        Feature parity checklist

KEYWORDS (in the launcher):
  find / open / in   File search
  g / wiki / gh …    Web search
  = expr             Calculator
  > cmd              Shell / terminal
  clip / cb          Clipboard history
  snip / sp          Snippets
  wf / workflow      Workflows
  bm / bookmark      Browser bookmarks
  contact            Contacts (vCard)
  music              Media control (playerctl)
  recent             Recent documents
  large <text>       Large Type
  buf                File buffer
  define / spell     Dictionary
  sys / sleep / …    System commands
  stats              Usage stats
"
    );
}

fn print_features() {
    let features = [
        ("App launcher (.desktop)", true),
        ("Usage-based ranking", true),
        ("File search", true),
        ("Web search (default + custom)", true),
        ("Calculator", true),
        ("Dictionary / spell", true),
        ("System commands", true),
        ("Shell / terminal", true),
        ("Large Type", true),
        ("File preview (text subtitle)", true),
        ("Hotkey daemon (configurable, default Super+Space)", true),
        ("Navigation & file actions", true),
        ("Recent documents", true),
        ("Fallback searches", true),
        ("Usage stats", true),
        ("Clipboard history", true),
        ("Snippets + auto-expand", true),
        ("Workflows (JSON)", true),
        ("Universal actions", true),
        ("File buffer", true),
        ("Themes", true),
        ("Preferences sync export/import", true),
        ("Bookmarks (Chrome/Firefox HTML)", true),
        ("Contacts (vCard)", true),
        ("Music control (playerctl)", true),
        ("Alfred Remote iOS", false),
        ("Native 1Password 1Click", false),
        ("Apple Music.app / Contacts.app", false),
    ];
    for (name, done) in features {
        println!("{} {}", if done { "[x]" } else { "[ ]" }, name);
    }
}
