# Changelog

All notable changes of the `rootmark` binary, the Rust crates, the Python
binding and the Obsidian client. One version number for all of them
(docs/PACKAGES.md); the spec carries its own (`rootmark --spec-version`).

## 0.2.0 — 2026-10-24 (phase 2: attested writes, migrations, installers)

Phase 2 rounds P2-0, P2-0.1, P2-0.2, P2-0.3 (attested writes) and P2-A
(migrations, installers). Spec 2.0.2. Release targets: `aarch64-apple-darwin`
and, new, `x86_64-unknown-linux-gnu` (K-P2A.6); Windows x86_64 is built,
installed and checked in CI for information; Intel macOS is not a target
(K-P2A.5). Nothing is code-signed (K-P2.4). Tagged after the independent
verifications of P2-A and P2-0.3 ("Şu koşullarla güvenilir", their conditions
closed by K-P2A.5 and K-P2-03.1; docs/RELEASE.md).

### Attested writes (spec 2.0.0–2.0.2 §7.1, §8; opt-in)
- A new layer `unverified` and the policy `[attestation] unattested =
  "unverified" | "canonical"`. The default recipe a new vault gets writes
  `canonical` (an external change is ingested as before); the `unverified`
  option keeps a file change no human-channel client vouched for out of
  `canonical` (actor `unattested_fs`), and is promoted like a proposal.
- Attestations of the human channel (`attest_session`, `attest` over HTTP with
  a session MAC; `rootmark attest` for humans outside Obsidian), recorded in
  `.rootmark/attestations.jsonl`; on-time and late (window `window_s`),
  partial matches, the link update and the rename of a note (layers carried
  block for block: a rename raises none and drops none).
- The Obsidian client attests its saves, renames and link updates.
- `rootmark serve --attest-log <file>` (outside the vault): a diagnostic log of
  every attestation and observation.
- Measured (P2-0.3, K-P2-03.1): with `unverified`, an agent writing the files
  directly is never `canonical` (0 leaks); a human's own edits stay
  `unverified` in a race-dependent share of renames of large self-linking
  notes and of the notes whose links Obsidian rewrites (0.0678 on the
  official seed). That is why `canonical` stays the default.

### Migrations and versions (P2-A)
- `rootmark migrate` (`--dry-run`, rollback records): store rename, index
  rebuild, log readers, checkpoints; an older index format is
  `migration_required` (its own error code), not `internal`; `events_hash`
  never changes.
- `status.migrations`: the version matrix of the vault's stored formats.

### Install (P2-A)
- `packaging/install.sh` (Linux x86_64, macOS arm64) and
  `packaging/install.ps1` (Windows x86_64): download, check against
  `SHA256SUMS`, install, fetch the pinned model; a changed archive is refused
  and nothing is installed. `install.sh` checks its prerequisites (`tar`,
  `sha256sum`/`shasum`, `curl`, `jq` for the model) before anything is
  downloaded (P2-A verification F2).
- Release archives with `SHA256SUMS` and GitHub build provenance attestations
  (`gh attestation verify <archive> -R arknsensei/rootmark`); every workflow
  action pinned to a full version.

### Upgrading from 0.1.0
- A vault set up by 0.1.0 has a `.rootmark/policy.toml` without an
  `[attestation]` table, and 0.2.0 reads such a recipe as
  `unattested = "unverified"` (the setting of rounds P2-0 to P2-0.2): a file
  edit no human-channel client attested is then `unverified` and left out of
  search until promoted. To keep 0.1.0's behaviour, add
  `[attestation]` / `unattested = "canonical"` to the recipe (a vault set up
  by 0.2.0 has it); `rootmark status` names the policy in force
  (`attestation.unattested`). A decision request is open (K-P2L-T1).

### Known limits
- `unverified` is opt-in for the race above; a rename back onto a path the
  note held earlier could drop a block added since to `unverified` (P2-0.3
  verification Ö3; fixed after 0.2.0, round P2-L).
- `rootmark migrate --dry-run` lists each migration's plan on its own, not
  chained (P2-A verification F1; round P2-L).

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
