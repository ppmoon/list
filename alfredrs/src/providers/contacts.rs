//! Local contacts via vCard files.

use crate::config::Config;
use crate::model::{Action, ItemKind, Query, ResultItem};
use crate::providers::Provider;
use crate::ranking::{data_dir, ensure_data_dir};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
}

#[derive(Default)]
pub struct ContactsProvider {
    cache: Mutex<Option<Vec<Contact>>>,
}

impl ContactsProvider {
    pub fn default_path() -> std::path::PathBuf {
        data_dir().join("contacts.vcf")
    }

    pub fn ensure_sample() -> anyhow::Result<()> {
        ensure_data_dir()?;
        let path = Self::default_path();
        if !path.exists() {
            std::fs::write(
                path,
                "BEGIN:VCARD\nVERSION:3.0\nFN:Ada Lovelace\nEMAIL:ada@example.com\nTEL:+1-555-0100\nORG:Analytical Engine\nEND:VCARD\nBEGIN:VCARD\nVERSION:3.0\nFN:Grace Hopper\nEMAIL:grace@example.com\nTEL:+1-555-0101\nORG:US Navy\nEND:VCARD\n",
            )?;
        }
        Ok(())
    }

    pub fn load(path: Option<&std::path::Path>) -> Vec<Contact> {
        let _ = Self::ensure_sample();
        let path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Self::default_path);
        let Ok(text) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        parse_vcf(&text)
    }

    fn contacts(&self, config: &Config) -> Vec<Contact> {
        let mut guard = self.cache.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Self::load(config.contacts_path.as_deref()));
        }
        guard.clone().unwrap_or_default()
    }
}

pub fn parse_vcf(text: &str) -> Vec<Contact> {
    let mut contacts = Vec::new();
    let mut current: Option<Contact> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.eq_ignore_ascii_case("BEGIN:VCARD") {
            current = Some(Contact {
                name: String::new(),
                email: None,
                phone: None,
                organization: None,
            });
            continue;
        }
        if line.eq_ignore_ascii_case("END:VCARD") {
            if let Some(c) = current.take() {
                if !c.name.is_empty() {
                    contacts.push(c);
                }
            }
            continue;
        }
        let Some(c) = current.as_mut() else {
            continue;
        };
        if let Some(rest) = line.strip_prefix("FN:") {
            c.name = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("EMAIL") {
            let val = rest.split(':').last().unwrap_or("").to_string();
            if !val.is_empty() {
                c.email = Some(val);
            }
        } else if let Some(rest) = line.strip_prefix("TEL") {
            let val = rest.split(':').last().unwrap_or("").to_string();
            if !val.is_empty() {
                c.phone = Some(val);
            }
        } else if let Some(rest) = line.strip_prefix("ORG:") {
            c.organization = Some(rest.to_string());
        }
    }
    contacts
}

impl Provider for ContactsProvider {
    fn name(&self) -> &'static str {
        "contacts"
    }

    fn search(&self, query: &Query, config: &Config) -> Vec<ResultItem> {
        let needle = match query.keyword.as_deref() {
            Some("contact") => query.argument.to_lowercase(),
            _ => return Vec::new(),
        };
        self.contacts(config)
            .into_iter()
            .filter(|c| {
                needle.is_empty()
                    || c.name.to_lowercase().contains(&needle)
                    || c.email
                        .as_ref()
                        .map(|e| e.to_lowercase().contains(&needle))
                        .unwrap_or(false)
                    || c.organization
                        .as_ref()
                        .map(|o| o.to_lowercase().contains(&needle))
                        .unwrap_or(false)
            })
            .map(|c| {
                let mut actions = Vec::new();
                if let Some(email) = &c.email {
                    actions.push(Action::CopyText(email.clone()));
                    actions.push(Action::OpenUrl(format!("mailto:{email}")));
                }
                if let Some(phone) = &c.phone {
                    actions.push(Action::CopyText(phone.clone()));
                    actions.push(Action::ShowLargeType(phone.clone()));
                }
                let subtitle = [
                    c.email.clone(),
                    c.phone.clone(),
                    c.organization.clone(),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · ");
                ResultItem::new(format!("contact:{}", c.name), c.name, ItemKind::Contact)
                    .with_subtitle(subtitle)
                    .with_score(7_000)
                    .with_actions(actions)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vcf() {
        let contacts = parse_vcf(
            "BEGIN:VCARD\nFN:Test User\nEMAIL:t@example.com\nTEL:+123\nEND:VCARD\n",
        );
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].email.as_deref(), Some("t@example.com"));
    }
}
