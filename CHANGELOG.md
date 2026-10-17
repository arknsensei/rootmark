# Changelog

All notable changes of the `mdk` binary, the Rust crates, the Python
binding and the Obsidian client. One version number for all of them
(docs/PACKAGES.md); the spec carries its own (`mdk --spec-version`).

## 0.1.0 — unreleased (phase 1)

The first release candidate: the first product scene (NORTH_STAR) end to end.
Not tagged until the independent verification of round P1-E says "Güvenilir"
(docs/RELEASE.md).

### Kernel (`mdkernel-core`)
- Markdown blocks with content-addressed versions and diff-inferred lineages
  (matcher v3), append-only event log without time stamps, object store,
  structural chunks (spec §1–6, frozen).
- Layers `canonical` / `proposal` / `derived`, promotion through a policy hook
  (`human_only` by default), checkpoints (named), consumption traces, raw
  journal and version trigger bounds (spec §7, §8, §11).
- Edit policy `append_only` by default: no agent write changes a canonical
  block or retrieval unit; change and delete events carry their actor (spec
  1.0.3).

### Index (`mdkernel-index`)
- tantivy BM25 (declared in the manifest), exact-cosine vectors with the ONNX
  MiniLM embedder, hybrid RRF, queries at a checkpoint, hot tier with
  compaction; `include_proposal` with a byte-equal canonical prefix (spec §9,
  §10).
- A missing or old-format index is `index_stale`, not `internal` (P1-E).

### Server and CLI (`mdk`)
- `mdk serve`: MCP over stdio (the agent channel) and a 127.0.0.1 HTTP API
  with a bearer token (the human channel), one writer queue per vault, file
  watcher, stable error codes (docs/CLIENT_CONTRACT.md).
- `[index] auto_rebuild` (default true) rebuilds an old-format index at start.
- A heading-less agent `append` to human blocks answers `policy_violation`
  with a `hint`.
- `mdk --version`, `mdk --spec-version`, `mdk conformance` (Levels 1–3 on the
  fixtures embedded in the binary).

### Clients
- `clients/obsidian` 0.1.0: blame gutter, history, proposal view with Promote
  (reason required; Demote not in the contract yet, K-E2-T), search with
  proposals apart, checkpoints with a read-only past view, trace viewer. Talks
  to `mdk serve` only through the HTTP contract, never writes a vault file, no
  runtime dependency besides the Obsidian API.

### Spec
- `spec/` 1.0.3 (Apache-2.0): §1–6 frozen, §7–§11 validated; conformance
  Levels 1–3, 34 cases.

### Known limits
- Release binaries: macOS x86_64 has no prebuilt ONNX Runtime in the pinned
  `ort` (K-E5-T); Windows is built by CI only.
- See docs/THREAT_MODEL.md "Bilinen açık sınırlar".
