# RootMark

**Block-level lineage, provenance and a deterministic derived index for Markdown written by people and agents together.**

[![Conformance](https://img.shields.io/endpoint?url=https%3A%2F%2Fdocs.rootmark.ai%2Fspec%2Fbadge.json)](https://docs.rootmark.ai/spec/conformance.json)
[![Spec 2.0.3](https://img.shields.io/badge/spec-2.0.3-blue)](https://docs.rootmark.ai/spec/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

> When an agent writes into a knowledge base, every block records who wrote it, when, and which block it came from. Nothing unapproved counts as knowledge. Every answer says which blocks it rests on. You can go back to yesterday's state block by block — and the files stay plain Markdown.

That is the goal. What has been measured, and under which conditions, is listed under [Evidence](#evidence) below; read the [condition on agents that write files directly](#the-measured-condition) before you rely on the second sentence.

## What it is

RootMark is one binary, `rootmark`, that runs beside a folder of Markdown files (a *vault*) and keeps, for every block of every note:

- **lineage** — which block it was derived from, across edits, splits, merges and moves [[E1]](#evidence);
- **provenance** — who wrote it (a person, an agent by name, a policy rule a person wrote) and through which channel [[E2]](#evidence);
- **a layer** — `canonical` (approved knowledge), `proposal` (an agent's suggestion), `unverified` (an unattested file change, if you choose so) [[E2]](#evidence), [[E6]](#evidence);
- **a deterministic index** — lexical (BM25) and vector search over approved blocks; the same files give the same index and the same event-log hash on every machine [[E3]](#evidence).

Agents talk to it over **MCP** (stdio); people and their editors over a **local HTTP API** (127.0.0.1 only) and the **CLI**. An **Obsidian plugin** shows blame, proposals with *Promote*, search, checkpoints and traces, and attests your saves.

## Scope

- **Your Markdown is never changed by RootMark.** No identifiers, no front matter, no hidden comments: everything derived lives in `<vault>/.rootmark/` and can be rebuilt from the files.
- **Approval is a human act** (or a rule a human wrote in advance): agents propose, people promote [[E2]](#evidence), [[E7]](#evidence). An agent writing through MCP cannot change approved content [[E4]](#evidence).
- **Every answer is traceable:** `search` and `read_block` record which blocks an answer rested on; `trace` lists them [[E2]](#evidence).
- **Checkpoints:** name a state and search or blame it later, block by block [[E2]](#evidence).
- **A published specification** with a conformance suite (Levels 1–3, 39 tests) that the binary runs on itself: `rootmark conformance`.
- Scale: tens of thousands to a few hundred thousand notes.

## What it is not

Not a vector database, not an agent-memory API, not a note-taking app, not a team wiki, not an LLM framework. It does not call any model API: the embedder is a small local ONNX model. It does not sync between machines. It does not replace git — see [how it differs from git](https://docs.rootmark.ai/git.html). It does not decide what is true; it makes it possible to decide, and records who did.

### The measured condition

An agent that can write the vault's files directly (for example Claude Code with file tools on a local vault) bypasses MCP. With the **default** policy such a write is counted as `canonical`, like a person's edit. It is kept out of `canonical` only when the recipe says `[attestation] unattested = "unverified"` **and** the people use an attesting client (the Obsidian plugin or `rootmark attest`); renames can then leave some human content `unverified` (on the safe side) [[E6]](#evidence). A process that can read the token, drive the human's client or write the plugin's files is on the human channel ([threat model](docs/THREAT_MODEL.md)).

## Quick start

### Binary (macOS on Apple silicon, Linux x86_64)

```sh
VER=0.4.0
curl -fsSLO "https://github.com/rootmark-ai/rootmark/releases/download/v$VER/install.sh"
curl -fsSLO "https://github.com/rootmark-ai/rootmark/releases/download/v$VER/SHA256SUMS"
grep ' install.sh$' SHA256SUMS | shasum -a 256 -c      # Linux: sha256sum -c
gh attestation verify install.sh -R rootmark-ai/rootmark   # optional: provenance
bash install.sh --version "$VER"                       # needs curl, tar, shasum/sha256sum, jq
export PATH="$HOME/.local/bin:$PATH" ROOTMARK_MODELS="$HOME/.local/share/rootmark/models"
rootmark conformance >/dev/null                        # level 3, 39/39
```

### Homebrew (macOS on Apple silicon, Linux x86_64)

```sh
brew install rootmark-ai/tap/rootmark
"$(brew --prefix rootmark)/share/rootmark/models/fetch_model.sh" "$(brew --prefix rootmark)/share/rootmark"
export ROOTMARK_MODELS="$(brew --prefix rootmark)/share/rootmark/models"
```

### Docker (linux/amd64; the default model is in the image)

```sh
IMG=ghcr.io/rootmark-ai/rootmark:0.4.0          # also on Docker Hub: rootmark/rootmark:0.4.0
docker run --rm "$IMG" conformance >/dev/null
docker run --rm --user "$(id -u):$(id -g)" -v "$HOME/notes:/vault" "$IMG" ingest-dir /vault /vault
docker run --rm --user "$(id -u):$(id -g)" -v "$HOME/notes:/vault" "$IMG" index build /vault
```

The image's default command is the MCP server over stdio with the watcher (`serve --vault /vault --mcp stdio --watch`): an agent starts it with `docker run -i --rm …`. The HTTP API listens on 127.0.0.1 inside the container only — see [the guide](https://docs.rootmark.ai/guide.html).

### Your first vault

```sh
export VAULT="$HOME/notes"          # any folder of Markdown files; use a copy the first time
rootmark ingest-dir "$VAULT" "$VAULT"
rootmark index build "$VAULT"
rootmark serve --vault "$VAULT" --http 127.0.0.1:7357 --watch   # the HTTP API for people and editors
```

Keep `--watch` on: changes made while no server watches the vault are taken in when it starts again, but only by a server started with `--watch`. Then connect an agent (Claude Code or any MCP client) and install the Obsidian plugin: **[the guide](https://docs.rootmark.ai/guide.html)**.

## The scene, in eleven steps

`scripts/scene.sh` runs it on a clean copy of the specification's mini vault; the agent talks MCP over stdio, the human talks HTTP with the token in `.rootmark/server.token` [[E5]](#evidence):

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

## Documentation

- **Guide** — install, prepare a vault, run the server, connect an agent, the Obsidian plugin, daily use, upgrading, headless use: <https://docs.rootmark.ai/guide.html> (source: [docs/GUIDE.md](docs/GUIDE.md))
- **How RootMark differs from git**: <https://docs.rootmark.ai/git.html>
- **Client contract** — every tool, field and error code: [docs/CLIENT_CONTRACT.md](docs/CLIENT_CONTRACT.md)
- **Specification** and conformance suite: <https://docs.rootmark.ai/spec/> (source in [`spec/`](spec/))
- **Threat model**: [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md) · **Releasing**: [RELEASING.md](RELEASING.md)

## Evidence

Each claim above rests on a measurement with criteria written before the work and, for the listed rounds, an independent verification. The records live in the development repository at the tag of this release:

| | Claim | Record | Condition |
|---|---|---|---|
| E1 | Block lineage across edits, splits, merges and moves | [P1-A final report](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-A/FINAL.md); the prototype rounds, [NORTH_STAR.md](https://github.com/arknsensei/rootmark/blob/v0.4.0/NORTH_STAR.md) (Turkish) | the conformance fixtures (C1 17/17) and the prototype's synthetic and real (56k-block) histories |
| E2 | Layers, promotion by people, checkpoints, consumption trace | [P1-B](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-B/FINAL.md), [P1-B.1](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-B.1/FINAL.md), [P1-D.1](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-D.1/FINAL.md) | spec §7, §8, §11; conformance C2 10/10 |
| E3 | Deterministic index; the same bytes on every machine | [P1-C](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-C/FINAL.md), [P2-V.1](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P2-V.1/FINAL.md) | embedder vectors byte-equal on x86_64 and aarch64 Linux and macOS arm64 |
| E4 | An agent cannot change approved content through MCP | [P1-D](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-D/FINAL.md) | the default `append_only` edit policy; the MCP channel |
| E5 | The eleven-step scene, end to end | [P1-E](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-E/FINAL.md), [P1-E.1](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P1-E.1/FINAL.md) | 11/11; the human side also in the real Obsidian interface |
| E6 | Unattested direct file writes are kept out of `canonical` | [P2-0.3](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P2-0.3/FINAL.md) | **only** with `unattested = "unverified"` and an attesting client; the default is `canonical`; renames: some human content may stay `unverified` (151/2228 = 0.0678 on the official seed) |
| E7 | Rules a person writes in advance can promote an agent's proposals | [P2-P](https://github.com/arknsensei/rootmark/blob/v0.4.0/reports/rounds/P2-P/FINAL.md) | the agent's name is a declaration, not an authenticated identity |

## Repository

| Path | Contents |
|---|---|
| `core/`, `index/`, `server/` | Rust crates `rootmark-core`, `rootmark-index`, `rootmark-server` (the `rootmark` binary) |
| `bindings/python/` | the `rootmark_core` Python module (distribution `rootmark-bindings`; not on PyPI) |
| `clients/obsidian/`, `clients/vscode/` | the Obsidian plugin; a VS Code client skeleton |
| `spec/` | the specification, the conformance suite and its fixtures |
| `packaging/` | install scripts and the Homebrew formula template |
| `testdata/` | reference data of the tests (old vaults for migration, tokenizer and embedder references) |
| `site/` | the documentation site (mdBook) |

## How this repository is made

This repository is **generated, one commit per release,** from the development repository [arknsensei/rootmark](https://github.com/arknsensei/rootmark), where the work happens in rounds with criteria written before the work and an independent verification after it. Each commit names its source (`Lab-Commit`, `Lab-Tag`, `Transfer-Tree`), and `TRANSFER.json` lists the source of every file; the `transfer-verify` workflow rebuilds the tree from the lab tag and checks that it is identical. Comments in the code that cite rounds or decisions (for example `K-P2B.1`) resolve in the development repository, whose process documents are in Turkish.

**Contributing:** issues and pull requests are welcome here; accepted changes are carried into the development repository with your authorship and come back with the next release ([CONTRIBUTING.md](CONTRIBUTING.md)). **Security:** [SECURITY.md](SECURITY.md) (security@rootmark.ai).

## License

Apache License 2.0 for the code, the clients and the specification ([LICENSE](LICENSE), [NOTICE](NOTICE), [spec/LICENSE](spec/LICENSE)). The embedder models (all-MiniLM-L6-v2, paraphrase-multilingual-MiniLM-L12-v2) are Apache-2.0; they are fetched at pinned revisions, and the default one ships in the container image with its licence. Third-party licences of the binary: `THIRD_PARTY_LICENSES` in every release archive and in the image.
