<!--
D-0/R6 — skeleton of the product repository's README (English). In D-1 it
becomes product/overlay/README.md (TRANSFER_PLAN §1.3, type Y). Placeholders:
<ORG> = the GitHub organisation (proposal: rootmark-ai, decision D0-K3),
<LAB> = the lab repository (proposal: arknsensei/rootmark-lab, D0-K4),
<DOCS> = the docs site (proposal: https://docs.rootmark.ai, D0-K9),
X.Y.Z = the release. Sections marked [D-1] are filled when the channel exists.
This file is not the lab's README.md, which stays as it is in D-0.
-->

# RootMark

**Block-level lineage, provenance and a deterministic derived index for Markdown written by people and agents together.**

[![Conformance](https://img.shields.io/endpoint?url=<DOCS>/spec/badge.json)](<DOCS>/spec/conformance.json)
[![Spec 2.0.3](https://img.shields.io/badge/spec-2.0.3-blue)](<DOCS>/spec/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

> When an agent writes into a knowledge base, every block records who wrote it, when, and which block it came from. Nothing unapproved counts as knowledge. Every answer says which blocks it rests on. You can go back to yesterday's state block by block — and the files stay plain Markdown.

## What it is

RootMark is one binary, `rootmark`, that runs beside a folder of Markdown files (a *vault*) and keeps, for every block of every note:

- **lineage** — which block it was derived from, across edits, splits, merges and moves;
- **provenance** — who wrote it (a person, an agent by name, a policy rule) and through which channel;
- **a layer** — `canonical` (approved knowledge), `proposal` (an agent's suggestion), `unverified` (an unattested file change, if you choose so);
- **a deterministic index** — lexical (BM25) and vector search over approved blocks only; the same files give the same index and the same event log hash on every machine.

Agents talk to it over **MCP** (stdio); people and their editors over a **local HTTP API** (127.0.0.1 only) and the **CLI**. An **Obsidian plugin** shows blame, proposals with *Promote*, search, checkpoints and traces, and attests your saves.

## Scope

- **Your Markdown is never changed by RootMark.** No identifiers, no front-matter, no hidden comments: everything derived lives in `<vault>/.rootmark/` and can be rebuilt from the files.
- **Approval is a human act** (or a rule a human wrote in advance): agents propose; people promote.
- **Every answer is traceable:** `search` and `read_block` record which blocks an answer rested on; `trace` lists them.
- **Checkpoints:** name a state and search or blame it later, block by block.
- **A published specification** with a conformance suite (Levels 1–3, 39 tests) that the binary runs on itself: `rootmark conformance`.
- Scale: tens of thousands to a few hundred thousand notes.

## What it is not

Not a vector database, not an agent-memory API, not a note-taking app, not a team wiki, not an LLM framework. It does not call any model API: the embedder is a small local ONNX model. It does not sync between machines (yet). It does not decide what is true — it makes it possible to decide, and records who did.

## Quick start

### Binary (macOS on Apple silicon, Linux x86_64)

```sh
VER=X.Y.Z
curl -fsSLO "https://github.com/<ORG>/rootmark/releases/download/v$VER/install.sh"
curl -fsSLO "https://github.com/<ORG>/rootmark/releases/download/v$VER/SHA256SUMS"
grep ' install.sh$' SHA256SUMS | shasum -a 256 -c      # Linux: sha256sum -c
gh attestation verify install.sh -R <ORG>/rootmark      # optional: provenance
bash install.sh --version "$VER"                       # needs curl, tar, shasum/sha256sum, jq
export PATH="$HOME/.local/bin:$PATH" ROOTMARK_MODELS="$HOME/.local/share/rootmark/models"
rootmark conformance >/dev/null                        # level 3, 39/39
```

### Homebrew [D-1]

```sh
brew install <ORG>/tap/rootmark
```

### Docker [D-1]

```sh
IMG=ghcr.io/<ORG>/rootmark:X.Y.Z      # also on Docker Hub: rootmark/rootmark:X.Y.Z
docker run --rm "$IMG" conformance >/dev/null
docker run --rm --user "$(id -u):$(id -g)" -v "$HOME/notes:/vault" "$IMG" ingest-dir /vault /vault
```

The HTTP API listens on 127.0.0.1 inside the container; use the image as an MCP server over stdio (`docker run -i … serve --vault /vault --mcp stdio --watch`) — see the guide.

### Your first vault

```sh
export VAULT="$HOME/notes"          # any folder of Markdown files; use a copy the first time
rootmark ingest-dir "$VAULT" "$VAULT"
rootmark index build "$VAULT"
rootmark serve --vault "$VAULT" --http 127.0.0.1 --watch
```

Then connect an agent (Claude Code or any MCP client) and install the Obsidian plugin: **[the guide](<DOCS>/guide/)**.

## The scene, in eleven steps

An agent proposes a note → you see it as plain Markdown in any editor → `blame` shows its origin block by block → unapproved blocks do not appear in search → `promote` approves them → `trace` lists the blocks an answer rested on → a checkpoint brings back yesterday's state. `scripts/scene.sh` runs it end to end on the specification's mini vault. [table of the eleven steps, from the lab README]

## Documentation

- **Guide** — install, prepare a vault, run the server, connect an agent, the Obsidian plugin, daily use, upgrading, headless use: <DOCS>/guide/
- **Client contract** — every tool, field and error code: [docs/CLIENT_CONTRACT.md](docs/CLIENT_CONTRACT.md)
- **Specification** and conformance suite: <DOCS>/spec/ (source in [`spec/`](spec/))
- **Threat model**: [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md) · **Known issues**: guide, "Known issues"

## Repository

| Path | Contents |
|---|---|
| `core/`, `index/`, `server/` | Rust crates `rootmark-core`, `rootmark-index`, `rootmark-server` (the `rootmark` binary) |
| `bindings/python/` | the `rootmark_core` Python module (distribution `rootmark-bindings`; not on PyPI yet) |
| `clients/obsidian/`, `clients/vscode/` | the Obsidian plugin; a VS Code client skeleton |
| `spec/` | the specification, the conformance suite and its fixtures |
| `packaging/` | install scripts and the Homebrew formula template |
| `testdata/` | reference data of the tests (old vaults for migration, embedder reference vectors) |

## How this repository is made

This repository is **generated, one commit per release,** from the development repository [<LAB>](https://github.com/<LAB>), where the work happens in rounds with criteria written before the work and an independent verification after it. Each commit names its source (`Lab-Commit`, `Lab-Tag`), and `TRANSFER.json` lists the source of every file; the `transfer-verify` workflow rebuilds the tree from the lab tag and checks that it is identical. Comments in the code that cite rounds or decisions (e.g. `K-P2B.1`) resolve in the lab repository.

**Contributing:** issues and pull requests are welcome here; accepted changes are carried into the lab repository with your authorship and come back with the next release ([CONTRIBUTING.md](CONTRIBUTING.md)). **Security:** see [SECURITY.md](SECURITY.md) (security@rootmark.ai).

## License

Apache License 2.0 for the code, the clients and the specification ([LICENSE](LICENSE), [NOTICE](NOTICE), [spec/LICENSE](spec/LICENSE)). The embedder models (all-MiniLM-L6-v2, paraphrase-multilingual-MiniLM-L12-v2) are Apache-2.0 and are fetched at pinned revisions (or shipped in the container image with their licence).
