# server — agent notes
Responsibility: the `rootmark` CLI, the MCP server (`rmcp`), the HTTP API, the file watcher, the policy recipe loader.
Dependency direction: core ← index ← server ← clients; `server` depends on `core` and `index` only (`scripts/check_deps.sh`).
Language and tools: Rust 1.92.0; `rmcp =3.6.0` (the official MCP Rust SDK, stdio), `axum =0.8.9`, `tokio =1.53.1`, `notify =8.2.0`, `schemars =1.2.1` (direct dependencies pinned to versions published at least 14 days earlier).
Frozen surfaces: the `rootmark` subcommands and their JSON outputs; `rootmark index …` / `rootmark search …` are `rootmarkindex::cli` (no second process). The behaviour of spec §7–§10 lives in the core and the index; the server only exposes it.
Test commands: `cargo test -p rootmark-server --release` (CLI tests `server/tests/cli_*.rs`; index tests need the model files, `make models`).
Not in this package: copies of core/index logic; any path other than `propose` that writes Markdown; identifiers or metadata in Markdown; loosening the default `promote` policy; binding to anything but 127.0.0.1; client code; new spec sections.
