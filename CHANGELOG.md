# Changelog

All notable changes of the `rootmark` binary, the Rust crates, the Python
binding and the Obsidian client. One version number for all of them
(docs/PACKAGES.md); the spec carries its own (`rootmark --spec-version`).

## 0.3.2 — 2026-11-08 (P2-Q.2: current Claude Code over MCP; the removal hold)

A patch of the MCP server and of `rootmark serve --watch` (round P2-Q.2;
two product bugs found in P2-Q.1, reports/rounds/P2-Q.1/FINDINGS.md PQ1-1
and PQ1-4). The MCP tools — names, input schemas, answers — are unchanged
(`tools/list` is byte for byte 0.3.1's); no event kind, field, format or
spec version changes: vaults written by 0.3.1 open as they are
(`events_hash` unchanged, no migration).

### Fixes
- **Current Claude Code lists rootmark's tools again (PQ1-1).** Claude
  Code 2.1.318 and later first send a `server/discover` request naming the
  2026-07-28 protocol; the MCP library 0.3.0 and 0.3.1 were built with
  (`rmcp` 3.5.0) then held the whole session to that protocol's per-request
  metadata, and refused the client's `tools/list` once it had fallen back to
  `initialize` 2025-11-25 (`Failed to fetch tools: request _meta is missing
  or has malformed required fields …`). Built with `rmcp` 3.6.0, where a
  discovery request no longer selects the session's lifecycle. The server
  still offers the protocol versions up to 2025-11-25 (a request naming
  2026-07-28 gets the explicit error `-32022`); clients that send
  `initialize` and no request metadata work as before. Checked with real
  Claude Code 2.1.319, headless: 11 tools, `search`, `propose`.
- **The removal hold starts after the batch's ingest (PQ1-4).** The watcher
  computed when to take a removed note's removal before it had taken in the
  rest of the batch; when that ingest was slow (a busy server) the removal
  could be taken before a rename's late attestation, which was then refused
  and left the renamed note `unverified` (only with `unattested =
  "unverified"`). The hold (`window_s` + 0.5 s) now starts after the batch's
  ingest; a batch that ingests notes and began before the removal's first
  due time extends it to that batch's end plus the hold, a batch that began
  later does not (so it is never held indefinitely).

### Documentation
- docs/GUIDE.md §4.1: checking the Claude Code connection headless
  (`claude -p`), reading the server's error from the debug log; "Known
  issues": the Claude Code item removed; an attested rename's old path is
  closed with `unattested_fs` under `"unverified"` (P2-Q.1 verification,
  recommendation 3a); Claude Code's SIGKILL at the end of a session (PQ2-1).

### Known issues
- Claude Code kills its session's server about half a second after SIGTERM;
  a `--watch` server still stopping then skips its final compaction and the
  removals it holds (PQ2-1). With `--mcp stdio`, SIGTERM alone does not stop
  the server (PQ-14). Planned for P2-C.
- As in 0.3.1: `ingest-dir` skips hidden directories (PQ-13); a hybrid
  "contains" query can rank its target below the first 10 (PQ-15); the
  pruning and shared-chunk issue of 0.3.0; the release archive carries only
  the default model's manifest.

## 0.3.1 — 2026-11-05 (P2-Q.1: the watcher's removals and new folders)

A patch of `rootmark serve --watch` (round P2-Q.1; two product bugs the P2-Q
pilot measured, reports/rounds/P2-Q/FINDINGS.md). No event kind, field,
format or spec version changes: vaults written by 0.3.0 open as they are
(`events_hash` unchanged, no migration).

### Fixes
- **Deletions and moves are ingested (PQ-18).** A Markdown file removed
  from the vault (deleted, or moved out of it), and every note of a folder
  removed or moved out, is taken into the log as the document without
  blocks: each of its live blocks gets a `BlockDeleted` (spec §5) with the
  actor and source of the watcher's other changes (`unattested_fs` under
  `unattested = "unverified"` when nobody attested it, spec §7.1), so it
  leaves search and `blame`. A move inside the vault is the old path's
  removal and the new path's ingest. The removal is taken after the
  attestation window plus 0.5 s (`[attestation] window_s`, 2 s by default),
  so that a client's attested rename — Obsidian's — is still checked against
  the old path; the renamed note is ingested once, its old path closed once.
  A file deleted and written again within that time is a change, not a
  removal. Pending removals are taken before the server stops.
- **Linux: files in a new folder are not missed (PQ-22).** A folder created
  in (or moved into) the vault is read when the watcher sees it, so a file
  written into it before inotify watched the folder is ingested; a file read
  twice is ingested once.

### Documentation
- docs/GUIDE.md §5.1: Obsidian's "Do you trust the author of this vault?"
  window and the current button name "Exit Restricted mode" (P2-Q
  verification VF-1, PQ-24); §4.1: keeping a desktop agent out of
  `.rootmark/` and the vault's files (P2-Q verification, recommendation 2).

### Known issues
- With `--mcp stdio`, SIGTERM logs `stopping` but the process exits only
  when the MCP stream closes (PQ-14); `ingest-dir` skips hidden directories
  (PQ-13); a hybrid "contains" query can rank its target below the first 10
  (PQ-15). Planned for P2-C.
- The pruning and shared-chunk issue of 0.3.0 (below) is unchanged.
- The release archive carries only the default model's manifest (as in
  0.3.0).

## 0.3.0 — 2026-11-03 (phase 2, blocks 1 and 2: Turkish vaults, promotion rules, contract v2, retention, CPU-independent embeddings)

Rounds P2-L (after 0.2.0's release), P2-P, P2-B, P2-R and P2-V.1. Spec
2.0.3: §7.1 (Draft) default `canonical`, rename back onto an earlier path;
§7 (Validated) the optional `meta.source` of `Promoted`/`Demoted`; §8 an
informative note on the search path filter. Start here:
[docs/GUIDE.md](docs/GUIDE.md) (install, a vault's recipe, `serve`,
connecting an agent, the Obsidian plugin, daily use, upgrading, headless
use, collecting data for an evaluation, troubleshooting).

### Upgrading from 0.2.0 (P2-V.1)
- The embedder's arithmetic no longer depends on the processor: the same
  vault gives the same vector bytes on macOS arm64 and Linux x86_64/aarch64
  (`embedder` id `…+fixed1`, e.g. `minilm-l6-v2/e73168f9+fixed1`). An index
  built by 0.2.0 (or an earlier build of this line) is reported
  `migration_required` by `status` (searches answer `migration_required`
  until then): run `rootmark migrate <vault>` once (it re-embeds every chunk;
  `--dry-run` shows the plan, `--rollback <id>` undoes it). Events, ids and
  `events_hash` are unchanged.
- The fixed-order embedding is slower than 0.2.0's: on an Apple M4 about
  50 ms per chunk (about 5× 0.2.0), on x86_64 servers about 2–3×; the first
  index build and every migration re-embed the whole vault. A speed round
  (P2-H) is planned.

### Client contract version 2 (P2-B)
- Block locations: `blame`, `replay`, `search` (`loc: true`) and
  `read_block` carry each block's `loc` (line range and byte offsets in the
  file) and `loc_stale` (CLIENT_CONTRACT §2.5); the Obsidian client aligns its
  gutter with them. Experimental (K-P2B-T1).
- Event stream: `events {since_seq, kinds, path_prefix, limit}` (HTTP
  `POST /v1/events`, MCP `events`), read-only, the log's lines byte for byte.
- Date filter: `search` takes `date_from`/`date_to` (`YYYY-MM-DD`); a
  document's date comes only from its path (`[index] date_from_path`) or its
  front matter (`[index] date_field`), never from the file time.
- `include_proposal` shows `unverified` blocks with their label; a VS Code
  client scaffold (`clients/vscode`, not published) speaks the same contract.

### Known issues
- A live `canonical` block that shares a search chunk with a block a prune
  rule made inactive is not retrieved after that prune (Block 2
  verification F1; nothing is deleted, the block is read at an earlier
  checkpoint and by `blame`/`read_block`). Run `rootmark prune <vault>
  --dry-run` (or `POST /v1/prune {"dry_run": true}`) first; the fix is
  planned for P2-C.
- Measured by the P2-Q pilot (reports/rounds/P2-Q/FINDINGS.md): on Linux, `serve --watch`
  can miss the first file written into a newly created directory (PQ-22); the watcher does not ingest
  removals, so a deleted note — and the old path of a renamed one — stays live
  and searchable (PQ-18); with `--mcp stdio`, SIGTERM logs `stopping` but the
  process exits only when the MCP stream closes (PQ-14); `ingest-dir` skips
  hidden directories (`.github/`, …; PQ-13). PQ-22 and PQ-18 are fixed in
  0.3.1 (P2-Q.1), the others in P2-C.
- The release archive carries only the default model's manifest: the
  multilingual model of a Turkish vault is fetched with the manifest from the
  release tag (docs/GUIDE.md "Turkish and mixed-language vaults").

### Retention and pruning (P2-R)
- `rootmark prune <vault> [--dry-run] [--now-ms <ms>]` and `POST /v1/prune`
  (HTTP, the human channel; `{dry_run?, now_ms?}`): the vault's explicit
  prune. Never run by `rootmark serve` on its own; not an MCP tool.
  `rootmark prune <vault> <lineage> --actor … --reason …` is unchanged.
- Proposal decay: `[prune] version` and `[[prune.rules]]` (`id`, `layer =
  "proposal"|"unverified"`, `idle_checkpoints >= 1`, optional `path_prefix`
  list): a lineage idle for that many checkpoints gets a `Demoted` with
  `meta.prune` (layer unchanged, `policy:<rule>`, `rule:<rule>`); it is no
  longer retrieved (also with `include_proposal`) from then on, and is read
  as before at an earlier checkpoint. `canonical` is never pruned.
- Retention: `[retention] keep_checkpoints = K` moves trace lines behind
  the oldest of the last K checkpoints, byte for byte, to
  `.rootmark/traces/archive/` (`at_seq` and `trace_id` unchanged; the base in
  `.rootmark/traces/base`; `rootmark trace list --archived`), and unnamed
  checkpoints no live trace reads to `.rootmark/checkpoints/archive/` (a query
  `at` one is refused with the new code `archived`, HTTP 410); location
  records of versions no live checkpoint names are deleted; the raw journal
  is pruned as by `rootmark journal prune`. rootmark never deletes the
  archive, nor `events.jsonl`, `objects/`, `snapshots/`,
  `attestations.jsonl`.
- `status.retention`, `checkpoints[].archived`, `trace.archived_before`
  (contract version 2's addendum, §2.10).
- Fixes: a pruned lineage was still retrieved with `include_proposal`;
  `search` computes its locations before its trace (a failure leaves no
  trace); `events` reads the log from `since_seq` with a line counter;
  an indented code block's location starts at its line start
  (K-P2B-T1 (a); ids and events unchanged; the alignment error of the
  pre-registered D-NOTES measurement drops from 0.00551 to 0) and the
  clients' slice check accepts it (the block's text after its first four
  columns); `commit_plan` checks the planned snapshot before writing.

### Declarative promotion rules (P2-P)
- `[[promotion.rules]]` in `.rootmark/policy.toml` (`id`, `actor =
  "agent:<name>"`, optional `source`, `path_prefix`, `to = "canonical"`;
  `[promotion] version` required with rules): an agent's `propose` under a
  rule's prefix is promoted to `canonical` at once, one `Promoted` per
  lineage the call wrote (`meta.actor = policy:<rule>`, `meta.reason =
  "rule:<rule>"`, `policy_version`), returned in the response's `promoted`.
  Only `propose` runs the rules: unattested file writes and `unverified`
  content are never promoted by a rule. A rule names the agent (its MCP
  client name, or the CLI's `--source`) — a self-declared name: keep prefixes
  narrow. `rootmark policy lint <vault>` checks the recipe; an invalid rule
  stops `rootmark serve`.
- `POST /v1/demote` (HTTP, the human channel): a human undoes a promotion,
  a rule's included.
- Channel label: a human's promotion or demotion over HTTP records its
  `X-Consumer` as `meta.source` (`rootmark promote/demote --source` on the
  command line) — e.g. a bot relaying a remote human's approval
  (docs/HEADLESS_CHANNEL.md).

### Upgrading from 0.1.0
- A vault set up by 0.1.0 has a `.rootmark/policy.toml` without an
  `[attestation]` table; this rootmark still reads such a recipe as
  `unattested = "unverified"` (the kernel's reading is unchanged), but now
  says so: `status.attestation.implicit` is `true`,
  `status.migrations.artifacts.policy` is `recipe/implicit`, and
  `rootmark serve` writes a warning line at start. `rootmark migrate <vault>`
  runs the new migration `policy_attestation_1`, which appends
  `[attestation]` / `unattested = "canonical"` to the recipe (every other byte
  kept, a rollback record under `.rootmark/migrations/`, planned by
  `--dry-run`); it is recommended (`status.migrations.recommended`), never
  `migration_required`. A recipe that has the table, or no recipe, is left
  alone. Decision K-P2L-T1, option (a) (K-P2P.1).

### Turkish vaults (P2-L, K-P2L-T2)
- Recommended configuration for a Turkish or Turkish/English vault:
  `[index] tokenizer = "uni_fold_tr"` and
  `[index] embedder = "multilingual-minilm-l12-v2"` (fetch the model as in
  docs/GUIDE.md, then `rootmark index build`, or `rootmark migrate` for an
  existing index). Measured on P2-L's D-TR set
  (400 documents, 800 queries): hybrid nDCG@10 0.8801 against 0.8029 with
  the defaults; the defaults (`mdk_lower_alnum`, `minilm-l6-v2`) are
  unchanged.

### Fixes (P2-P/P0)
- The lexical-variant knob `ROOTMARK_DEFAULT_TOKENIZER` (P2-L's conformance
  test) is gone: no environment variable of a released binary chooses the
  tokenizer.
- A filtered search's filter line is written before its trace line (and taken
  back if the trace cannot be written); `status.migrations` reports an error
  instead of empty lists when a rollback record cannot be read.

### Downgrading to 0.2.0 (P2-B/B0)
- A recipe with `[[promotion.rules]]` does not open in 0.2.0 (it refuses the
  unknown table, so `rootmark serve` does not start): remove the
  `[[promotion.rules]]` tables (and `[promotion] version`) from
  `.rootmark/policy.toml` before going back. The rules' `Promoted` events
  stay in the log; 0.2.0 reads them as a policy actor's promotions.

### Downgrading after a prune (P2-R)
- A vault whose traces or checkpoints were archived (`.rootmark/traces/base`,
  the archive directories) is refused by 0.2.0: its reader counts `at_seq`
  from the first line of `traces.jsonl` and expects every checkpoint file in
  `.rootmark/checkpoints/`. Move the archive files back (the trace archive
  files in name order, then `traces.jsonl`; the checkpoint files) and remove
  `.rootmark/traces/` before downgrading. `[retention]` and `[prune]` are
  unknown to 0.2.0's recipe reader: remove them first.

### Fixes (P2-B/B0)
- `rootmark migrate --dry-run`: the plan of `policy_attestation_1` says
  `recommended: true` and `required: false` (it said `required: true`).
- `status.migrations`: a `policy.toml` that cannot be read is an error of
  the matrix (`status.migrations.error`), not an `unknown` row.
- Reading rule: a rule's `Promoted` names its own rule (`meta.reason =
  "rule:<meta.actor.id>"`) and comes from `proposal`; a log that says
  otherwise is refused when the vault opens.
- A search `path_prefix` written in NFD keeps being folded to NFC (a read);
  a rule's `path_prefix` in NFD keeps being refused (an authority). Both are
  now documented in docs/CLIENT_CONTRACT.md §2.2 and tested.

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
