//! Global hotkey daemon — Alfred-style summon key.

use crate::config::Config;
use anyhow::{bail, Context};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// Parse strings like `Super+Space`, `Ctrl+Alt+Space`, `Alt+Space`.
pub fn parse_hotkey(spec: &str) -> anyhow::Result<HotKey> {
    let mut mods = Modifiers::empty();
    let mut code: Option<Code> = None;
    for part in spec.split('+') {
        let p = part.trim();
        match p.to_ascii_lowercase().as_str() {
            "super" | "meta" | "cmd" | "win" => mods |= Modifiers::SUPER,
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" | "option" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "space" => code = Some(Code::Space),
            "tab" => code = Some(Code::Tab),
            "enter" | "return" => code = Some(Code::Enter),
            "escape" | "esc" => code = Some(Code::Escape),
            other if other.len() == 1 => {
                let ch = other.chars().next().unwrap();
                code = Some(letter_code(ch).with_context(|| format!("unsupported key: {other}"))?);
            }
            other => bail!("unsupported hotkey token: {other}"),
        }
    }
    let code = code.context("hotkey missing a key (e.g. Space)")?;
    Ok(HotKey::new(Some(mods), code))
}

fn letter_code(ch: char) -> Option<Code> {
    Some(match ch.to_ascii_uppercase() {
        'A' => Code::KeyA,
        'B' => Code::KeyB,
        'C' => Code::KeyC,
        'D' => Code::KeyD,
        'E' => Code::KeyE,
        'F' => Code::KeyF,
        'G' => Code::KeyG,
        'H' => Code::KeyH,
        'I' => Code::KeyI,
        'J' => Code::KeyJ,
        'K' => Code::KeyK,
        'L' => Code::KeyL,
        'M' => Code::KeyM,
        'N' => Code::KeyN,
        'O' => Code::KeyO,
        'P' => Code::KeyP,
        'Q' => Code::KeyQ,
        'R' => Code::KeyR,
        'S' => Code::KeyS,
        'T' => Code::KeyT,
        'U' => Code::KeyU,
        'V' => Code::KeyV,
        'W' => Code::KeyW,
        'X' => Code::KeyX,
        'Y' => Code::KeyY,
        'Z' => Code::KeyZ,
        _ => return None,
    })
}

/// Block forever, launching the GUI whenever the configured hotkey is pressed.
/// Also polls the system clipboard into alfredrs history (Clipboard History watch).
pub fn run_daemon() -> anyhow::Result<()> {
    use crate::providers::clipboard::ClipboardProvider;

    let config = Config::load_or_default()?;
    let hotkey = parse_hotkey(&config.hotkey)?;
    let manager = GlobalHotKeyManager::new().context("create hotkey manager")?;
    manager.register(hotkey).context("register hotkey")?;
    eprintln!(
        "alfredrs daemon listening for {} (clipboard watch on)",
        config.hotkey
    );

    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("alfredrs"));
    let receiver = GlobalHotKeyEvent::receiver();
    let mut last_clip: Option<String> = None;
    let mut ticks: u64 = 0;
    loop {
        if let Ok(event) = receiver.try_recv() {
            if event.state == HotKeyState::Pressed {
                let _ = Command::new(&exe).arg("gui").spawn();
            }
        }
        ticks += 1;
        // Poll clipboard ~ every 500ms.
        if ticks % 10 == 0 {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    if last_clip.as_ref() != Some(&text) {
                        let _ = ClipboardProvider::push_text(&text, config.clipboard_max_items);
                        last_clip = Some(text);
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_super_space() {
        assert!(parse_hotkey("Super+Space").is_ok());
    }

    #[test]
    fn parses_ctrl_alt_letter() {
        assert!(parse_hotkey("Ctrl+Alt+A").is_ok());
    }
}
