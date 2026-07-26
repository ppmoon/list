# alfredrs

Alfred-inspired keyboard launcher for Linux, written in Rust.

Inspired by [Alfred for macOS](https://www.alfredapp.com). **Not affiliated with Running with Crayons Ltd.** “Alfred” is a trademark of Running with Crayons Ltd.

## Features

| Alfred feature | alfredrs | Notes |
|---|---|---|
| App launcher | ✅ | FreeDesktop `.desktop` discovery |
| Usage-based ranking | ✅ | Persisted launch counts |
| File search | ✅ | `find` / `open` / `in` keywords |
| Web search | ✅ | Defaults + custom URL templates |
| Calculator | ✅ | `=` prefix or inline maths |
| Spell & define | ✅ | Built-in dictionary |
| System commands | ✅ | lock / sleep / reboot / trash… |
| Shell / terminal | ✅ | `>` prefix |
| Large Type | ✅ | `large <text>` |
| File preview | ✅ | Text snippet in subtitle |
| Hotkeys | ✅ | `alfredrs daemon` (default `Super+Space`) |
| Navigation & actions | ✅ | Alt+Enter universal actions |
| Recent documents | ✅ | XDG `recently-used.xbel` |
| Fallback searches | ✅ | Google / DuckDuckGo by default |
| Usage stats | ✅ | `stats` keyword + CLI |
| Clipboard history | ✅ | `clip` / `cb`; daemon polls clipboard |
| Snippets | ✅ | `snip` / `;keyword` auto-expands in launcher |
| Workflows | ✅ | JSON graphs in data dir |
| File buffer | ✅ | `buf` |
| Themes | ✅ | Colours / fonts / sizes |
| Preferences sync | ✅ | export/import config, usage, snippets, workflows, contacts, clipboard |
| Bookmarks | ✅ | Chrome JSON + Netscape HTML |
| Contacts | ✅ | Local vCard |
| Music control | ✅ | `playerctl` (Linux) |
| Alfred Remote iOS | ❌ | macOS/iOS exclusive |
| 1Password 1Click | ❌ | Optional later via workflow |
| Music.app / Apple Contacts | ❌ | Platform-specific |

## Install / run

```bash
cd alfredrs
cargo run --release
```

CLI:

```bash
cargo run -- search "g rust"
cargo run -- run "= 2+2"
cargo run -- features
cargo run -- help
```

## Keywords

- `find notes` — file search  
- `g query` / `wiki query` / `gh query` — web search  
- `= 2*21` — calculator  
- `> ls -la` — shell  
- `clip paste` — clipboard history  
- `snip sig` — snippets  
- `wf` / `date` / `hello` — workflows  
- `bm alfred` — bookmarks  
- `contact ada` — contacts  
- `music` — media controls  
- `large 555-0199` — Large Type  
- `define rust` / `spell clipbord` — dictionary  
- `sleep` / `sys lock` — system  
- `stats` — usage  

## Data layout

Under `~/.local/share/alfredrs/`:

- `config.toml` — preferences & theme  
- `usage.json` — launch ranking  
- `clipboard.json` — clipboard history  
- `snippets.json` — text snippets  
- `workflows/*.json` — workflow definitions  
- `contacts.vcf` — address book  
- `buffer.json` — file buffer  

## Architecture

- **Providers** implement Alfred feature surfaces behind a shared `Provider` trait  
- **Engine** ranks results (fuzzy + usage) and executes actions  
- **UI** is an egui floating launcher (`alfredrs` with no args)  
- Domain logic is unit/integration tested without a display  

## License

MIT (see repository root). Alfred itself is proprietary software; this project is an independent reimplementation of the interaction model for Linux.
