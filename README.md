# rootmark

**rootmark** is named for what it keeps: the *root* of every block of a Markdown knowledge base, meaning where it came from, who wrote it and what it was derived from, while the Markdown (*mark*) files stay the one source of truth. It is a client-independent kernel that gives a Markdown corpus written by people and agents together block-level lineage, provenance and a deterministic derived index.

[![Conformance](https://img.shields.io/endpoint?url=https%3A%2F%2Farknsensei.github.io%2Frootmark%2Fspec%2Fbadge.json)](https://arknsensei.github.io/rootmark/spec/conformance.json)
[![Spec 2.0.3](https://img.shields.io/badge/spec-2.0.3-blue)](https://arknsensei.github.io/rootmark/spec/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

**[Running rootmark — the guide](docs/GUIDE.md)**: install, prepare a vault, run the server, connect an agent, install the Obsidian plugin, daily use, collecting data for an evaluation, upgrading, headless use.

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

Release 0.3.0 targets macOS on Apple silicon (`aarch64-apple-darwin`) and Linux x86_64 (`x86_64-unknown-linux-gnu`, glibc 2.38+); a Windows x86_64 archive is built and checked in CI for information. Step by step, with checks and outputs: [docs/GUIDE.md §1](docs/GUIDE.md#1-install).

```sh
VER=0.3.0
curl -fsSLO "https://github.com/arknsensei/rootmark/releases/download/v$VER/install.sh"
curl -fsSLO "https://github.com/arknsensei/rootmark/releases/download/v$VER/SHA256SUMS"
grep ' install.sh$' SHA256SUMS | shasum -a 256 -c      # Linux: sha256sum -c
bash install.sh --version "$VER"                       # needs curl, tar, shasum/sha256sum, jq
export PATH="$HOME/.local/bin:$PATH" ROOTMARK_MODELS="$HOME/.local/share/rootmark/models"
rootmark --version            # rootmark 0.3.0
rootmark --spec-version       # 2.0.3
rootmark conformance >/dev/null   # rootmark conformance: level 3, 39/39 (spec 2.0.3)
```

Other ways: the release archives (`rootmark-0.3.0-<target>.tar.gz`, checked against `SHA256SUMS`; provenance with `gh attestation verify <file> -R arknsensei/rootmark`); `cargo binstall rootmark-server` once the crate is published; a Homebrew formula draft in [`packaging/homebrew/rootmark.rb.in`](packaging/homebrew/rootmark.rb.in) (no tap yet); from source with Rust 1.92.0: `cargo build --release -p rootmark-server --bin rootmark`, then `make models`.

**Turkish and mixed-language vaults:** `[index] tokenizer = "uni_fold_tr"` and `embedder = "multilingual-minilm-l12-v2"` in the vault's `.rootmark/policy.toml`, with the multilingual model fetched as in [GUIDE §1.2](docs/GUIDE.md#12-turkish-and-mixed-language-vaults-the-multilingual-model) (on P2-L's Turkish set: hybrid nDCG@10 0.8801 against 0.8029 with the defaults).

**Upgrading:** from 0.2.0, `rootmark migrate <vault>` rebuilds the index once (the embedder's arithmetic no longer depends on the processor; until then the index is `migration_required`); from 0.1.0, the same command adds `[attestation] unattested = "canonical"` to the recipe (a recipe without the table is read as `unverified`; the HTTP/MCP `status` says `attestation.implicit: true` and `rootmark serve` warns). `--dry-run` shows the plan. A vault from before the project's rename opens after `rootmark migrate --rename-store <vault>`. Details: [GUIDE §8](docs/GUIDE.md#8-upgrading-and-migrating), [CHANGELOG.md](CHANGELOG.md).

### Obsidian

The Obsidian plugin (status bar, blame gutter, proposals with Promote, search, checkpoints, traces; it attests your saves) is not in the community store. Install it from the release's `rootmark-0.3.0.zip` into `<vault>/.obsidian/plugins/rootmark/` and enable it under *Settings → Community plugins*, or with [BRAT](https://github.com/TfTHacker/obsidian42-brat) (`arknsensei/rootmark`): [GUIDE §5](docs/GUIDE.md#5-the-obsidian-plugin). It connects to a running `rootmark serve` on the recipe's `[server] http_port` as `[identity] human_id`; the status bar says `rootmark 0.3.0 · human:<you>`.

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
