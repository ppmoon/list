//! Application launcher via FreeDesktop `.desktop` files.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct DesktopApp {
    pub id: String,
    pub name: String,
    pub comment: String,
    pub exec: String,
    pub icon: String,
    pub path: PathBuf,
}

#[derive(Default)]
pub struct AppsProvider {
    cache: Mutex<Option<Vec<DesktopApp>>>,
}

impl AppsProvider {
    pub fn discover() -> Vec<DesktopApp> {
        let mut apps = Vec::new();
        let mut dirs = Vec::new();
        if let Some(data) = dirs::data_dir() {
            dirs.push(data.join("applications"));
        }
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".local/share/applications"));
        }
        dirs.push(PathBuf::from("/usr/share/applications"));
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

        for dir in dirs {
            if !dir.is_dir() {
                continue;
            }
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }
                if let Some(app) = parse_desktop_file(&path) {
                    apps.push(app);
                }
            }
        }
        apps.sort_by(|a, b| a.name.cmp(&b.name));
        apps.dedup_by(|a, b| a.id == b.id);
        apps
    }

    fn apps(&self) -> Vec<DesktopApp> {
        let mut guard = self.cache.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Self::discover());
        }
        guard.clone().unwrap_or_default()
    }
}

pub fn parse_desktop_file(path: &Path) -> Option<DesktopApp> {
    let text = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut comment = String::new();
    let mut exec = None;
    let mut icon = String::new();
    let mut no_display = false;
    let mut hidden = false;
    let mut in_desktop_entry = false;

    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            match k {
                "Name" if name.is_none() => name = Some(v.to_string()),
                "Comment" if comment.is_empty() => comment = v.to_string(),
                "Exec" if exec.is_none() => exec = Some(v.to_string()),
                "Icon" if icon.is_empty() => icon = v.to_string(),
                "NoDisplay" => no_display = v.eq_ignore_ascii_case("true"),
                "Hidden" => hidden = v.eq_ignore_ascii_case("true"),
                _ => {}
            }
        }
    }

    if no_display || hidden {
        return None;
    }
    let name = name?;
    let exec = exec?;
    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.clone());

    Some(DesktopApp {
        id: format!("app:{id}"),
        name,
        comment,
        exec,
        icon,
        path: path.to_path_buf(),
    })
}

/// Strip FreeDesktop field codes (`%f`, `%U`, …) from Exec lines.
pub fn clean_exec(exec: &str) -> Option<(String, Vec<String>)> {
    let cleaned = exec
        .replace("%f", "")
        .replace("%F", "")
        .replace("%u", "")
        .replace("%U", "")
        .replace("%i", "")
        .replace("%c", "")
        .replace("%k", "")
        .trim()
        .to_string();
    let mut parts = shell_words(&cleaned);
    if parts.is_empty() {
        return None;
    }
    let program = parts.remove(0);
    Some((program, parts))
}

fn shell_words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for ch in s.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

impl Provider for AppsProvider {
    fn name(&self) -> &'static str {
        "apps"
    }

    fn search(&self, query: &Query, _config: &Config) -> Vec<ResultItem> {
        // Keyword ownership is enforced by ProviderSet; here we always search free text.
        let q = query.raw.trim();
        if q.is_empty() {
            return Vec::new();
        }

        self.apps()
            .into_iter()
            .filter_map(|app| {
                let (program, args) = clean_exec(&app.exec)?;
                Some(
                    ResultItem::new(app.id.clone(), app.name.clone(), ItemKind::App)
                        .with_subtitle(if app.comment.is_empty() {
                            app.path.display().to_string()
                        } else {
                            app.comment.clone()
                        })
                        .with_icon(app.icon)
                        .with_path(app.path)
                        .with_actions(vec![Action::RunCommand { program, args }]),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_desktop_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("demo.desktop");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(
            f,
            "[Desktop Entry]\nName=Demo App\nComment=A demo\nExec=demo %U\nIcon=demo\nType=Application"
        )
        .unwrap();
        let app = parse_desktop_file(&path).unwrap();
        assert_eq!(app.name, "Demo App");
        assert_eq!(clean_exec(&app.exec).unwrap().0, "demo");
    }

    #[test]
    fn hides_nodisplay() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hidden.desktop");
        fs::write(
            &path,
            "[Desktop Entry]\nName=Hidden\nExec=true\nNoDisplay=true\n",
        )
        .unwrap();
        assert!(parse_desktop_file(&path).is_none());
    }
}
