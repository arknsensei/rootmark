# rootmark

**rootmark** is named for what it keeps: the *root* of every block of a Markdown knowledge base, meaning where it came from, who wrote it and what it was derived from, while the Markdown (*mark*) files stay the one source of truth. It is a client-independent kernel that gives a Markdown corpus written by people and agents together block-level lineage, provenance and a deterministic derived index.

[![Conformance](https://img.shields.io/endpoint?url=https%3A%2F%2Farknsensei.github.io%2Frootmark%2Fspec%2Fbadge.json)](https://arknsensei.github.io/rootmark/spec/conformance.json)
[![Spec 2.0.0](https://img.shields.io/badge/spec-2.0.0-blue)](https://arknsensei.github.io/rootmark/spec/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

> When an agent writes into a knowledge base, every block records who wrote it, when, and which block it came from. Nothing unapproved counts as knowledge. Every answer says which blocks it rests on. You can go back to yesterday's state block by block, and the files stay plain Markdown.

rootmark is not a vector database, an agent memory API, a note-taking app, a team wiki or an LLM framework. It is one binary that runs beside a Markdown folder and answers over MCP (for agents), a local HTTP API (for people and their editors) and a CLI. It never writes identifiers or metadata into your Markdown files; everything it derives lives in `<vault>/.rootmark/` and can be rebuilt from the files.

## The scene, in eleven steps

The product scene of phase 1 (`scripts/scene.sh` runs it on a clean copy of `spec/fixtures/mini_vault`; the real-Obsidian version is `scripts/plugin_e2e_real.sh`). The agent talks MCP over stdio and the human talks HTTP with the token in `.rootmark/server.token`:

```sh
rootmark ingest-dir vault vault                 # blocks, lineages, event log
rootmark serve --vault vault --mcp stdio --http 127.0.0.1:0 --watch
```

| # | Who | Call | What you see |
|---|---|---|---|
| 1 | human | `GET /v1/status` | spec version, policies (`human_only` promotion, `append_only` edits), the index state |
| 2 | human | `POST /v1/checkpoint {"name":"before"}` | a named checkpoint of the state before the agent |
| 3 | agent | MCP `propose {doc_path:"agent/scene.md", …}` | a new plain Markdown file; its blocks are `proposal` |
| 4 | human | `POST /v1/blame {"doc_path":"agent/scene.md"}` | every block: `proposal`, author `Agent`, source = the agent |
| 5 | human | `POST /v1/search {"query":…}` | the proposal is not found: unapproved is not knowledge |
| 6 | human | `POST /v1/promote {"lineage_id":…, "reason":…}` | `Promoted` by `human:<you>`; without a reason → 400, nothing written |
| 7 | human | `POST /v1/search` | found, `canonical` |
| 8 | human | `POST /v1/trace` | the answer of step 7 and the blocks it rested on |
| 9 | human | `POST /v1/search {"at":"before"}`, `POST /v1/blame {"at":"before"}` | yesterday's state, block by block |
| 10 | agent | MCP `propose` `replace_section` of a human section | `denied`; a heading-less append → `policy_violation` with a hint; nothing written |
| 11 | human | `POST /v1/blame`, `POST /v1/search` | unchanged |

The full contract (every tool, field and error code) is [docs/CLIENT_CONTRACT.md](docs/CLIENT_CONTRACT.md).

## Install

Release targets of 0.2.0: macOS on Apple silicon (`aarch64-apple-darwin`) and Linux x86_64 (`x86_64-unknown-linux-gnu`). Windows x86_64 is built, installed and checked in CI for information (K-E1.1, K-P2A.6).

- **Installer:** download `install.sh` and `SHA256SUMS` from the [release](https://github.com/arknsensei/rootmark/releases), check them (`gh attestation verify install.sh -R arknsensei/rootmark`, `grep ' install.sh$' SHA256SUMS | sha256sum -c`), then `bash install.sh --version 0.2.0` (needs `tar`, `sha256sum` or `shasum`, `curl` and `jq`); Windows: `install.ps1`. Details: [docs/RELEASE.md](docs/RELEASE.md) "Kurulum".
- **Release archive:** download `rootmark-0.2.0-<target>.tar.gz` from the [releases](https://github.com/arknsensei/rootmark/releases), check it against `SHA256SUMS`, put `rootmark` on your `PATH`. The embedder model is fetched and checked by `models/fetch_model.sh` in the archive (pinned revision, sha256 and blake3).
- **cargo-binstall:** `cargo binstall rootmark-server` (installs the `rootmark` binary from the release archive; works once the crate is published).
- **Homebrew:** a formula draft is in [`packaging/homebrew/rootmark.rb.in`](packaging/homebrew/rootmark.rb.in) (filled by `scripts/release.sh`); there is no tap yet.
- **From source:** Rust 1.92.0, `cargo build --release -p rootmark-server --bin rootmark`, then `make models`.

```sh
rootmark --version            # rootmark 0.2.0
rootmark --spec-version       # 2.0.2
rootmark conformance          # Levels 1–3 on the embedded fixtures: 39/39 (spec 2.0.2)
```

A vault written by a build from before the project's rename is opened after `rootmark migrate --rename-store <vault>`, which moves its metadata directory to `.rootmark/` and changes no byte inside it (same event log, same `events_hash`, same index).

### Obsidian (BRAT)

The Obsidian client is not in the community store yet. With [BRAT](https://github.com/TfTHacker/obsidian42-brat):

1. Install and enable BRAT in Obsidian.
2. BRAT → *Add beta plugin* → `arknsensei/rootmark` (the release assets carry `main.js`, `manifest.json`, `styles.css`).
3. Enable **rootmark**. In your vault set `[server] http_port` and `[identity] human_id` in `.rootmark/policy.toml` and run `rootmark serve --vault <vault> --http 127.0.0.1:<port>`.
4. The status bar says `rootmark 0.2.0 · human:<you>`. A stale or damaged index is shown there with the way to rebuild it.

## Specification

The specification is published separately from this implementation: [spec/SPEC.md](spec/SPEC.md) (rendered at https://arknsensei.github.io/rootmark/spec/), with the conformance suite in [spec/CONFORMANCE.md](spec/CONFORMANCE.md) and its fixtures in `spec/fixtures/`. §1–§6 are frozen. `rootmark conformance` runs Levels 1–3 on the fixtures embedded in the binary.

## Repository

| Path | Contents |
|---|---|
| `core/`, `index/`, `server/` | Rust crates `rootmark-core`, `rootmark-index`, `rootmark-server` (the `rootmark` binary) |
| `bindings/python/` | the `rootmark_core` Python module (used by `research/` only) |
| `clients/obsidian/` | the Obsidian client (TypeScript) |
| `spec/` | the specification, the conformance suite and fixtures |
| `research/` | measurement code and research arms (Python, never shipped) |
| `docs/` | north star, architecture, decisions, development process, round directives (Turkish) |
| `reports/` | every round's measured results and independent verifications |

Development runs in rounds with criteria written before the work and an independent verification after it; see [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/DEV_PROCESS.md](docs/DEV_PROCESS.md). Security reports: [SECURITY.md](SECURITY.md).

## License

Apache License 2.0 for the code, the Obsidian client and the specification ([LICENSE](LICENSE), [NOTICE](NOTICE), [spec/LICENSE](spec/LICENSE)).
