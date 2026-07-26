//! Preferences, themes, and sync export/import.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::paths::{data_dir, ensure_data_dir};
use crate::ranking::UsageStats;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    pub accent: [u8; 3],
    pub selection: [u8; 3],
    pub font_size: f32,
    pub window_width: f32,
    pub window_height: f32,
    pub corner_radius: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Midnight Forge".into(),
            background: [24, 26, 32],
            foreground: [236, 239, 244],
            accent: [94, 196, 168],
            selection: [40, 48, 58],
            font_size: 18.0,
            window_width: 720.0,
            window_height: 420.0,
            corner_radius: 12.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearch {
    pub keyword: String,
    pub title: String,
    pub url: String,
}

impl WebSearch {
    pub fn render(&self, query: &str) -> String {
        self.url.replace("{query}", &urlencoding::encode(query))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkey: String,
    pub theme: Theme,
    pub web_searches: Vec<WebSearch>,
    pub fallback_searches: Vec<String>,
    pub file_search_roots: Vec<PathBuf>,
    pub file_search_max_results: usize,
    pub clipboard_max_items: usize,
    pub terminal: String,
    pub sync_path: Option<PathBuf>,
    pub contacts_path: Option<PathBuf>,
    pub enabled_providers: HashMap<String, bool>,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            hotkey: "Super+Space".into(),
            theme: Theme::default(),
            web_searches: default_web_searches(),
            fallback_searches: vec!["google".into(), "duckduckgo".into()],
            file_search_roots: vec![home.clone(), home.join("Documents"), home.join("Downloads")],
            file_search_max_results: 40,
            clipboard_max_items: 200,
            terminal: default_terminal(),
            sync_path: None,
            contacts_path: None,
            enabled_providers: default_enabled_providers(),
        }
    }
}

fn default_terminal() -> String {
    std::env::var("TERMINAL").unwrap_or_else(|_| "x-terminal-emulator".into())
}

fn default_web_searches() -> Vec<WebSearch> {
    vec![
        WebSearch {
            keyword: "google".into(),
            title: "Google".into(),
            url: "https://www.google.com/search?q={query}".into(),
        },
        WebSearch {
            keyword: "g".into(),
            title: "Google".into(),
            url: "https://www.google.com/search?q={query}".into(),
        },
        WebSearch {
            keyword: "duckduckgo".into(),
            title: "DuckDuckGo".into(),
            url: "https://duckduckgo.com/?q={query}".into(),
        },
        WebSearch {
            keyword: "ddg".into(),
            title: "DuckDuckGo".into(),
            url: "https://duckduckgo.com/?q={query}".into(),
        },
        WebSearch {
            keyword: "wiki".into(),
            title: "Wikipedia".into(),
            url: "https://en.wikipedia.org/wiki/Special:Search?search={query}".into(),
        },
        WebSearch {
            keyword: "gh".into(),
            title: "GitHub".into(),
            url: "https://github.com/search?q={query}".into(),
        },
        WebSearch {
            keyword: "yt".into(),
            title: "YouTube".into(),
            url: "https://www.youtube.com/results?search_query={query}".into(),
        },
        WebSearch {
            keyword: "maps".into(),
            title: "Google Maps".into(),
            url: "https://www.google.com/maps/search/{query}".into(),
        },
        WebSearch {
            keyword: "amazon".into(),
            title: "Amazon".into(),
            url: "https://www.amazon.com/s?k={query}".into(),
        },
    ]
}

fn default_enabled_providers() -> HashMap<String, bool> {
    [
        "apps",
        "files",
        "web",
        "calculator",
        "dictionary",
        "system",
        "shell",
        "clipboard",
        "snippets",
        "workflows",
        "bookmarks",
        "contacts",
        "music",
        "recent",
        "large_type",
        "buffer",
        "stats",
        "fallback",
    ]
    .into_iter()
    .map(|k| (k.to_string(), true))
    .collect()
}

impl Config {
    pub fn path() -> PathBuf {
        data_dir().join("config.toml")
    }

    pub fn load_or_default() -> anyhow::Result<Self> {
        ensure_data_dir()?;
        let path = Self::path();
        if path.exists() {
            let text = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&text)?)
        } else {
            let cfg = Self::default();
            cfg.save()?;
            Ok(cfg)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        ensure_data_dir()?;
        let text = toml::to_string_pretty(self)?;
        std::fs::write(Self::path(), text)?;
        Ok(())
    }

    pub fn provider_enabled(&self, name: &str) -> bool {
        self.enabled_providers.get(name).copied().unwrap_or(true)
    }

    /// Export preferences pack for sync/backup (Alfred Preferences Sync analogue).
    pub fn export_sync_pack(&self, usage: &UsageStats, dest: &Path) -> anyhow::Result<()> {
        use crate::providers::clipboard::ClipboardProvider;
        use crate::providers::contacts::ContactsProvider;
        use crate::providers::snippets::SnippetsProvider;
        use crate::providers::workflows::WorkflowProvider;

        let pack = SyncPack {
            config: self.clone(),
            usage: usage.clone(),
            snippets: Some(SnippetsProvider::load()),
            workflows: Some(WorkflowProvider::load_all()),
            contacts: Some(ContactsProvider::load(self.contacts_path.as_deref())),
            clipboard: Some(ClipboardProvider::load()),
            exported_at: chrono::Utc::now().to_rfc3339(),
        };
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(dest, serde_json::to_string_pretty(&pack)?)?;
        Ok(())
    }

    pub fn import_sync_pack(src: &Path) -> anyhow::Result<(Self, UsageStats)> {
        use crate::providers::clipboard::ClipboardProvider;
        use crate::providers::contacts::ContactsProvider;
        use crate::providers::snippets::SnippetsProvider;
        use crate::providers::workflows::WorkflowProvider;

        let text = std::fs::read_to_string(src)?;
        let pack: SyncPack = serde_json::from_str(&text)?;
        pack.config.save()?;
        if let Some(snippets) = &pack.snippets {
            SnippetsProvider::save(snippets)?;
        }
        if let Some(workflows) = &pack.workflows {
            let dir = WorkflowProvider::dir();
            std::fs::create_dir_all(&dir)?;
            for wf in workflows {
                let path = dir.join(format!("{}.json", wf.id));
                std::fs::write(path, serde_json::to_string_pretty(wf)?)?;
            }
        }
        if let Some(contacts) = &pack.contacts {
            let path = pack
                .config
                .contacts_path
                .clone()
                .unwrap_or_else(ContactsProvider::default_path);
            write_contacts_vcf(&path, contacts)?;
        }
        if let Some(clipboard) = &pack.clipboard {
            ClipboardProvider::save(clipboard)?;
        }
        Ok((pack.config, pack.usage))
    }
}

fn write_contacts_vcf(path: &Path, contacts: &[crate::providers::contacts::Contact]) -> anyhow::Result<()> {
    let mut out = String::new();
    for c in contacts {
        out.push_str("BEGIN:VCARD\nVERSION:3.0\n");
        out.push_str(&format!("FN:{}\n", c.name));
        if let Some(email) = &c.email {
            out.push_str(&format!("EMAIL:{email}\n"));
        }
        if let Some(phone) = &c.phone {
            out.push_str(&format!("TEL:{phone}\n"));
        }
        if let Some(org) = &c.organization {
            out.push_str(&format!("ORG:{org}\n"));
        }
        out.push_str("END:VCARD\n");
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, out)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPack {
    pub config: Config,
    pub usage: UsageStats,
    #[serde(default)]
    pub snippets: Option<Vec<crate::providers::snippets::Snippet>>,
    #[serde(default)]
    pub workflows: Option<Vec<crate::providers::workflows::Workflow>>,
    #[serde(default)]
    pub contacts: Option<Vec<crate::providers::contacts::Contact>>,
    #[serde(default)]
    pub clipboard: Option<Vec<crate::providers::clipboard::ClipItem>>,
    pub exported_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_search_encodes_query() {
        let search = WebSearch {
            keyword: "g".into(),
            title: "Google".into(),
            url: "https://www.google.com/search?q={query}".into(),
        };
        assert_eq!(
            search.render("hello world"),
            "https://www.google.com/search?q=hello%20world"
        );
    }

    #[test]
    fn theme_roundtrip_toml() {
        let theme = Theme::default();
        let cfg = Config {
            theme: theme.clone(),
            ..Config::default()
        };
        let text = toml::to_string(&cfg).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back.theme, theme);
    }
}
