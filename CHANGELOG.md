# Changelog

All notable changes of the `rootmark` binary, the Rust crates, the Python
binding and the Obsidian client. One version number for all of them
(docs/PACKAGES.md); the spec carries its own (`rootmark --spec-version`).

## Unreleased

Rounds P2-L and P2-P (block 1 of phase 2). Spec 2.0.3: §7.1 (Draft) default
`canonical`, rename back onto an earlier path; §7 (Validated) the optional
`meta.source` of `Promoted`/`Demoted`; §8 an informative note on the search
path filter.

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
  `[index] embedder = "multilingual-minilm-l12-v2"` (`make models-multilingual`,
  then `rootmark migrate` rebuilds the index). Measured on P2-L's D-TR set
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
