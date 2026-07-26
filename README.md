# list

Agent skills workspace. Primary product in this clone:

## alfredrs

Rust reimplementation of the [Alfred](https://www.alfredapp.com) interaction model for Linux.

```bash
cd alfredrs
cargo run --release          # launcher GUI
cargo run --release -- daemon  # Super+Space (configurable) summon
cargo test
cargo run -- features        # parity checklist
```

See [`alfredrs/README.md`](alfredrs/README.md) for the full feature map, keywords, and data layout.

Domain notes: [`CONTEXT.md`](CONTEXT.md). PRD: GitHub issue #2.
