# Changelog

All notable changes of the `rootmark` binary, the Rust crates, the Python
binding and the Obsidian client. One version number for all of them
(docs/PACKAGES.md); the spec carries its own (`rootmark --spec-version`).

## 0.1.0 — 2026-10-18 (phase 1)

The first release: the first product scene (NORTH_STAR) end to end. Tagged
after the independent verification of round P1-E.1 said "Güvenilir"
(reports/rounds/P1-E.1/VERIFICATION.md) and round GH-0 renamed the project
and moved it to github.com/arknsensei/rootmark (docs/RELEASE.md).
Release target: `aarch64-apple-darwin` only (K-E1.1).

The project was named `mdkernel` (binary `mdk`, crates `mdkernel-*`, vault
directory `.mdkernel/`) until round GH-0 renamed it to `rootmark` before this
release (K-GH.1); no behaviour changed. A vault written under the old name is
opened after `rootmark migrate --rename-store <vault>` (`events_hash` unchanged).

### Kernel (`rootmark-core`)
- Markdown blocks with content-addressed versions and diff-inferred lineages
  (matcher v3), append-only event log without time stamps, object store,
  structural chunks (spec §1–6, frozen).
- Layers `canonical` / `proposal` / `derived`, promotion through a policy hook
  (`human_only` by default), checkpoints (named), consumption traces, raw
  journal and version trigger bounds (spec §7, §8, §11).
- Edit policy `append_only` by default: no agent write changes a canonical
  block or retrieval unit; change and delete events carry their actor (spec
  1.0.3).

### Index (`rootmark-index`)
- tantivy BM25 (declared in the manifest), exact-cosine vectors with the ONNX
  MiniLM embedder, hybrid RRF, queries at a checkpoint, hot tier with
  compaction; `include_proposal` with a byte-equal canonical prefix (spec §9,
  §10).
- A missing or old-format index is `index_stale`, not `internal` (P1-E); so is
  an unreadable manifest (P1-E.1, K-E1.2).

### Server and CLI (`rootmark`)
- `rootmark serve`: MCP over stdio (the agent channel) and a 127.0.0.1 HTTP API
  with a bearer token (the human channel), one writer queue per vault, file
  watcher, stable error codes (docs/CLIENT_CONTRACT.md).
- `[index] auto_rebuild` (default true) rebuilds an old-format index at start.
- `status` answers whatever the index state: `index {state: ok|stale|missing|
  damaged, format, expected_format, auto_rebuild}` replaces the `index` string
  (P1-E.1, K-E1.2).
- A heading-less agent `append` to human blocks answers `policy_violation`
  with a `hint`.
- `rootmark --version`, `rootmark --spec-version`, `rootmark conformance` (Levels 1–3 on the
  fixtures embedded in the binary).

### Clients
- `clients/obsidian` 0.1.0: blame gutter, history, proposal view with Promote
  (reason required; Demote not in the contract yet, K-E2-T), search with
  proposals apart, checkpoints with a read-only past view, trace viewer. Talks
  to `rootmark serve` only through the HTTP contract, never writes a vault file, no
  runtime dependency besides the Obsidian API.
- The connection status names a stale, missing or damaged index and the way to
  rebuild it (P1-E.1). End to end in a real Obsidian instance
  (`scripts/plugin_e2e_real.sh`, K-E1.3).

### Spec
- `spec/` 1.0.3 (Apache-2.0): §1–6 frozen, §7–§11 validated; conformance
  Levels 1–3, 34 cases.

### Known limits
- Release binary: macOS arm64 only (K-E1.1). Linux x86_64 and Windows x86_64
  are built and checked by CI for information, not released; macOS x86_64 is
  not a target (no prebuilt ONNX Runtime in the pinned `ort`, K-E5-T).
  Installers and signing are phase 2.
- See docs/THREAT_MODEL.md "Bilinen açık sınırlar".
