# rootmark Specification v2.0.0

Status: **Frozen for §1–§6 (validated by prototype rounds v1–v3, Oct 2026); Validated for §7, §8, §11 (Phase 1 round P1-B, Oct 2026; conformance Level 2); Validated for §9 (Phase 1 rounds P1-C, P1-C.1, Oct 2026; conformance Level 3); Draft for §10 (to be validated in Phase 1); Draft for §7.1 and the attestation log of §8 (Phase 2 round P2-0, hypothesis round).** Version 2.0.0 (Phase 2 round P2-0): §7 adds the layer `unverified` and attested writes (§7.1): an external change of a document that no human-channel client attested is ingested as `unverified`, its change and delete events carry the actor kind `unattested_fs`, a late attestation within the window gives it the layer an attested ingest would have given, and `unverified` is promoted like `proposal`; §8 adds the attestation log; §9 retrieves `unverified` as `proposal`; §10 counts the attestations among the inputs of the determinism contract; informative notes in §1 and §4.2. Major: a log written under 2.0.0 with an unattested change cannot be read by the reading rules of 1.0.x (a fourth layer value, a fourth actor kind; `docs/SPEC_EDITS_CRITERION.md`, K-E0.1). §1–§6 are unchanged except for informative notes. Version 1.0.3 (Phase 1 round P1-D.1): §7 change and delete events carry their actor and source; §8 an unnamed checkpoint can be named (`CheckpointNamed`); §9 the length of a response with `proposal` results is bounded; informative notes on the edit policy (§7), the provenance of events (§4.2, §4.3) and the lexical document length (§9). Version 1.0.2 (Phase 1 round P1-C.1): §9 requested `proposal` results are ranked apart from, and appended after, unchanged `canonical` results; empty queries are rejected; lexical scoring is declared in the index manifest (informative). Version 1.0.1 (Phase 1 round P1-B.1): §7 undeclared `synthetic` → `canonical`; §8 `trace_id` with separators and input validation; §7/§8 serialization normative. §1–§6 are unchanged. License: Apache-2.0 (specification). Language: English, for publication; implementation directives are maintained in Turkish under `docs/`.

The key words MUST, MUST NOT, SHOULD, MAY are to be interpreted as in RFC 2119.

## 0. Scope and relationship to prior work

rootmark specifies a block-level lineage, provenance and governance layer over plain Markdown files. Markdown files are the single source of truth; every structure defined here is derived from them and can be rebuilt from them.

ContextNest (2026) specifies document-level governance for Markdown context (typed documents, hash-chained version history, checkpoints, publication gate, consumption traces). rootmark adopts its vocabulary where meanings coincide and extends governance to the block level with lineage inferred from diffs, deterministic derived indexes and incremental indexing. rootmark does not reuse ContextNest's implementation. A ContextNest-compatible export is a possible extension and is out of scope here.

Non-goals: realtime multi-user editing (Phase 2), encrypted/zero-knowledge servers, corpora beyond ~10^5 documents, entity extraction and graph ranking (research).

## 1. Terminology

- **Vault**: a directory tree of Markdown files plus a derived store under `<vault>/.rootmark/`.
- **Document**: one Markdown file, addressed by `doc_path` (vault-relative, `/`-separated).
- **Block**: the structural atom of Markdown (§2). Immutable, content-addressed.
- **Version**: a concrete block content, identified by `version_id`.
- **Lineage**: the identity of a block across versions, identified by `lineage_id`.
- **Snapshot**: the ordered list of `(lineage_id, version_id)` of a document at a point.
- **Event**: an append-only record of a change to a document's blocks.
- **Checkpoint**: an immutable vault-wide map `doc_path → snapshot_id` with an event-log offset.
- **Layer**: the governance status of a version: `canonical`, `proposal`, `derived`.

  Informative (2.0.0): §7 adds a fourth layer, `unverified` (content of an external change that no human-channel client attested, §7.1); a `Promoted` event can also move a version from `unverified` to `canonical`.
- **Promotion**: the event moving a version from `proposal` to `canonical`.
- **Consumption trace**: an append-only record of which block versions a consumer received for a query at a checkpoint.
- **Chunk**: a derived retrieval unit composed of consecutive blocks; never a source of identity.

## 2. Blocks

### 2.1 Normalization
`normalized_text` = NFC(text) → line endings to `\n` → trim leading/trailing whitespace → collapse runs of whitespace to one space. For `code` blocks only NFC and line-ending normalization apply. Emphasis markers are preserved.

### 2.2 Block boundaries
Implementations MUST derive top-level blocks from a CommonMark parser with tables enabled:
heading → `heading` (level 1–6); paragraph outside lists → `paragraph`; each top-level list item → `list_item` (nested items are part of the parent's text); fenced/indented code → `code`; table → `table`; blockquote → `blockquote` (not parsed inside); everything else → `other`. Leading YAML front matter is stored on the snapshot and produces no block. Empty paragraphs produce no block.

`heading_path` is the list of heading texts above a block, root to leaf; a heading's own path excludes itself.

### 2.3 Wikilinks
`[[target]]`, `[[target|label]]`, `[[target#heading]]`, `[[target#heading|label]]`. `target` is a file name without extension, path-independent, trimmed.

## 3. Identifiers

All hashes are BLAKE3; identifiers are the first 32 hex characters. Timestamps MUST NOT enter any identifier.

- `version_id = H(block_type + "\n" + normalized_text)`
- `snapshot_id = H(version_id_1 + "\n" + … + version_id_n)`
- `lineage_id = H("lineage\n" + doc_path + "\n" + version_id + "\n" + k)` where `k` is the 0-based ordinal of this block among blocks with the same `version_id` that receive a new lineage in the same snapshot, in document order.
- `chunk_id = H(version_id_1 + "\n" + … )`
- `checkpoint_id = H(sorted "doc_path\tsnapshot_id" lines + "\n" + event_seq)`

## 4. Object store and event log

### 4.1 Object store
`.rootmark/objects/<vid[0:2]>/<vid>.json` (blocks), `.rootmark/snapshots/<sid>.json`, `.rootmark/checkpoints/<cid>.json`. Writing an existing identifier with different bytes MUST fail; identical bytes MUST be a no-op.

### 4.2 Event log
`.rootmark/events.jsonl`, append-only. Each event carries `seq` (monotonic, vault-wide, no gaps), `doc_path`, `kind`, `lineage_ids`, `old_version_ids`, `new_version_ids`, `snapshot_id`, `meta`. Kinds: `BlockCreated`, `BlockUpdated`, `BlockMoved`, `BlockSplit`, `BlockMerged`, `BlockDeleted`, `SnapshotCommitted`, and (§7–§8) `LayerChanged`, `Promoted`, `Demoted`, `CheckpointCreated`.

Replaying the log MUST reconstruct every document's latest snapshot with an identical `snapshot_id`. The BLAKE3 hash of the whole log is the vault's determinism fingerprint (§10).

Informative (1.0.3): §7 adds `meta.actor` and `meta.source` to the change and delete events written through a governed write path, and §8 adds the vault-wide kind `CheckpointNamed`; neither changes the fields or kinds defined here, their meaning, nor any identifier.

Informative (2.0.0): §7.1 records attestation decisions in the events it already defines (`meta.layer` of block events, `LayerChanged`); the attestation log of §8 is a separate file. No kind, field or identifier defined here changes, and opening a vault written before 2.0.0 changes neither its log nor its fingerprint.

### 4.3 Provenance
Every version carries `provenance = {author_kind ∈ {human, agent, transcript, synthetic}, source}`. Provenance is immutable.

Informative (1.0.3): the provenance of events — who caused a version to change, move, split, merge or disappear — is recorded by §7 in the `meta` of those events; the provenance of versions above is unchanged.

## 5. Lineage inference (normative algorithm, "matcher v3")

Input: previous snapshot (or none) and the newly parsed block drafts. Output: blocks with `lineage_id` and events. Steps run in order; each step consumes only blocks unmatched by previous steps.

1. **Exact match.** Equal `version_id` → same lineage. Duplicates with identical content are anchored first by context hash `H(prev_vid|"" + "\n" + vid + "\n" + next_vid|"")`, then by ordinal. Position change (outside the longest increasing subsequence of old order) → `BlockMoved`.
2. **Split candidates.** For an unmatched old block `o`, consecutive unmatched new blocks `n_i..n_j` (2–4) with `sim(o, concat) ≥ THETA_SPLIT`; every piece ≥ 1 word; best `sim` wins. **Dominant-piece rule:** the longest new piece (characters; first on tie) inherits `o`'s lineage; the others receive new lineages; all have `parents = [o]`. Event `BlockSplit` with `meta.dominant`.
3. **Merge candidates.** Symmetric: consecutive old `o_i..o_j` (2–4) vs one new `n` with `sim(concat, n) ≥ THETA_SPLIT`. The new block inherits the longest old block's lineage; `parents = [o_i..o_j]`. Event `BlockMerged` with `meta.dominant`.
4. **Similarity assignment.** Remaining old × new of equal `block_type`: `s = 0.70·sim + 0.15·[heading_path equal] + 0.15·max(0, 1 − |pos_old − pos_new| / max(n_old, n_new))`, `sim` = normalized Levenshtein. Optimal assignment (Hungarian); pairs with `s ≥ THETA` → `BlockUpdated`, lineage inherited.
5. **Residual check, split direction.** If a matched `(o, n)` has adjacent unmatched new blocks `r` and `sim(o, concat(n, r)) > sim(o, n) + 0.10`, the match is revoked and a `BlockSplit` is emitted.
6. **Residual check, merge direction.** If a matched `(o, n)` has an adjacent unmatched (apparently deleted) old neighbour `o'` and `sim(concat(o, o'), n) > sim(o, n) + 0.10` (up to 3 old blocks), the match is revoked and a `BlockMerged` is emitted.
7. **Remainder.** Unmatched new → `BlockCreated` (new lineage); unmatched old → `BlockDeleted`.

Validated defaults: `THETA = 0.50`, `THETA_SPLIT = 0.65`. Implementations MAY use other values but MUST record `matcher_version` and parameters in `meta` of every event, and MUST NOT mix matcher versions within one vault.

Measured behaviour (prototype v3, reference implementation): lineage retention 1.000 (easy) / 0.988 (hard synthetic) / 0.9998 untouched blocks in real git history; split/merge event F1 0.997; false linkage 0.000. Retention degrades to ~0.91 when more than 30–45 % of words change and sentence order is shuffled; this is a known limit of similarity-based identity.

## 6. Chunks (derived)

Chunks reference blocks; blocks never reference chunks. Rebuilding chunks with a different strategy MUST NOT change any `lineage_id` and MUST NOT emit events. Reference strategies: `structural` (consecutive blocks under one `heading_path`, ≤ 512 tokens, oversized single blocks stand alone) and `fixed` (512-token window, 64 overlap; baseline only). Chunk `text` is prefixed with `heading_path` joined by ` > `.

## 7. Layers and promotion (Validated, P1-B)

Every version has `layer ∈ {canonical, proposal, unverified, derived}` (`unverified`: 2.0.0, §7.1); layers MUST be serialized in lowercase, as written here. Default on ingest: `human` → `canonical`; `agent`, `transcript` → `proposal`; `synthetic` → as declared, and `canonical` if no layer is declared; content of an unattested external change → `unverified`, whatever its author kind (§7.1). Where a rule takes the lowest of several layers (a new lineage among the lineages it derives from, a chunk among its blocks, §9), layers are ordered `canonical` > `proposal` > `unverified` > `derived`.

- `Promoted` event: `proposal → canonical`, or (2.0.0) `unverified → canonical`; carries `actor` (human id or policy name), `reason`, and `policy_version`. The kernel MUST NOT decide promotion; it exposes a hook. A `proposal` MAY be `Demoted` or pruned by policy; pruning never deletes objects, only marks the lineage inactive.
- Retrieval defaults: `canonical` full weight; `proposal` and `unverified` MUST be excluded or clearly down-weighted and labelled; `derived` is never returned as content.
- Layer transitions are events and therefore replayable and auditable.
- Provenance of events (1.0.3): `BlockUpdated`, `BlockMoved`, `BlockSplit`, `BlockMerged` and `BlockDeleted` events written through a governed write path (every path through which an agent, or a server on behalf of any client, writes) MUST carry `meta.actor = {kind, id}` (`kind` ∈ {`human`, `agent`, `policy`, `unattested_fs`}, `id` not empty; `unattested_fs`: 2.0.0, §7.1) — who caused the ingest — and `meta.source` (a string, the source of the ingested content), both or neither. Events of any other kind, except `Promoted` and `Demoted` (whose `meta.actor` is that of their transition, above), MUST NOT carry `meta.actor` or `meta.source`. A reader MUST accept their absence (logs written before 1.0.3, or by an unattributed core path), MUST reject a malformed or unpaired value and one on another kind, and MUST NOT invent an actor for an event without one.
- Informative (1.0.3): an implementation decides an agent's write through an edit hook, as it decides promotion through a policy hook; the kernel computes the effect of the write before anything is written (the existing lineages it would change or remove, the retrieval units made of `canonical` blocks it would change) and the policy decides. The reference implementation's default policy `append_only` refuses any agent write that changes or removes a `canonical` block or a `canonical` retrieval unit, so agent writes never change the `canonical` results of any query (conformance C3-06); a write allowed by an explicit `unrestricted` policy still carries its actor.

### 7.1 Attested writes (2.0.0; Draft, Phase 2 round P2-0)

The governed write paths of §7 attribute every write they carry. A process that can write the vault's files directly bypasses them; without this section such an external change is ingested as `human` content and becomes `canonical`. Attested writes let a human-channel client vouch for the bytes it saves, and keep every other external change out of `canonical`.

- **External change**: a change of a document's file that reaches the vault outside every governed write path; the implementation observes it on disk (for instance by watching the files, or by syncing the file before a governed write to the same document).
- **Attestation**: a statement, made only through the implementation's authenticated human channel (never through an agent's channel), that a client saved the bytes whose BLAKE3 hash is `content_hash` to `doc_path`. It is recorded in the attestation log (§8).
- **Window `W`**: a policy parameter in seconds (reference default 2.0). An external change of `doc_path`, observed with file bytes whose hash is `h`, is **attested** if an attestation of `(doc_path, h)` was recorded at most `W` seconds before the observation, or is recorded at most `W` seconds after it (a **late** attestation). Wall-clock time serves only this decision: it MUST NOT be written to any log or enter any identifier. The decision is recorded by the events below, so replay never decides it again.
- **Policy** `unattested ∈ {unverified, canonical}`, default `unverified`. Under `canonical` every external change is ingested as in 1.0.3 (`human`, `canonical`); the implementation MUST report which policy is in force.
- Under `unverified`, an attested external change is ingested as in 1.0.3. An external change that is not attested at its observation (an **unattested ingest**) is ingested with its versions' provenance `{author_kind: "human", source: "unattested_fs"}`, with every block event that gives a version a layer (`BlockCreated`, `BlockUpdated`, `BlockSplit`, `BlockMerged`) declaring `meta.layer = "unverified"`, and with its change and delete events carrying `meta.actor = {kind: "unattested_fs", id: "fs"}` and `meta.source = "unattested_fs"`. The layer rules apply unchanged: a lineage it changes drops to `unverified` by a `LayerChanged` event before the `SnapshotCommitted`; a lineage it moves keeps its layer; a lineage it deletes is deleted with its actor.
- Reading (2.0.0): a reader MUST reject, when the vault is opened, `meta.layer` on an event that gives no layer; a declared `meta.layer` other than `unverified` for content that is not `synthetic`; `meta.actor.kind = "unattested_fs"` on a `Promoted` or `Demoted` event, or on an event that gives a layer without declaring `meta.layer = "unverified"`; and a layer or actor kind outside the values of §7.
- **Late attestation**: an attestation recorded within `W` after an unattested change, whose `content_hash` equals the hash of the bytes that change ingested, gives each lineage that change left `unverified`, and that is still live at the version that change gave it, `unverified` and not pruned (a lineage changed again since, by anyone, is not raised: its content is no longer what was attested), the layer the same ingest would have given it had it been attested (the layer rules of §7 with the author `human`: `canonical`, or a lower layer it inherited or derived from — a `proposal` lineage the change touched returns to `proposal`; a late attestation never promotes). It does so by one `LayerChanged` per lineage, outside any ingest, with `meta` carrying `from: "unverified"`, `to` (that layer, `proposal` or `canonical`), `cause: "late_attestation"` and the `attestation_id`. It is the only `LayerChanged` that raises a layer. Replay MUST reject a `LayerChanged` that raises a layer other than from `unverified` to `proposal` or `canonical` with `cause: "late_attestation"`, inside an ingest, of a lineage that is not live and `unverified`, or whose `attestation_id` is not in the attestation log for the same `doc_path` with `at_seq` ≤ the event's `seq`.
- **Promotion and retrieval**: `unverified` is promoted, demoted and pruned like `proposal` (`Promoted` with `meta.from = "unverified"`; the kernel still decides nothing). Retrieval treats `unverified` as `proposal` (§9): excluded by default, and when `proposal` results are requested ranked in the same separate pass, labelled `unverified`, score multiplied by 0.5.
- **Migration**: opening a vault written before 2.0.0 changes no layer, no event and no fingerprint; only external changes observed afterwards are subject to this section.

## 8. Checkpoints and consumption traces (Validated, P1-B)

- **Checkpoint**: `{checkpoint_id, name?, event_seq, docs: {doc_path: snapshot_id}}`. Immutable. Any query MAY be evaluated "as of" a checkpoint; the result MUST be identical for the same checkpoint and query (§10).
- **Naming** (1.0.3): a checkpoint created without a name MAY be named once, later, by a vault-wide `CheckpointNamed` event with `meta.{checkpoint_id, name}`. Its `checkpoint_id`, `event_seq`, `docs` and stored object do not change. A name is unique in the vault and MUST NOT have the form of a checkpoint id; naming a named or unknown checkpoint, or giving a taken name, MUST be rejected (also on replay). `CheckpointNamed` is not a content event: it does not change the `event_seq` of a later checkpoint.
- **Consumption trace**: `.rootmark/traces.jsonl`, append-only: `{trace_id, checkpoint_id, consumer, query_hash, returned: [(doc_path, lineage_id, version_id)], at_seq}`; each `returned` entry MUST be serialized as an object with the keys `doc_path`, `lineage_id`, `version_id`. `trace_id = H(checkpoint_id + "\n" + consumer + "\n" + query_hash + "\n" + at_seq)`, where `at_seq` is written in decimal. `query_hash` MUST be exactly 32 lowercase hexadecimal characters and `consumer` MUST NOT contain `"\n"`; a trace violating either MUST be rejected. Traces answer "which block versions did this answer depend on".
- **Attestation log** (2.0.0; Draft, §7.1): `.rootmark/attestations.jsonl`, append-only: `{attestation_id, doc_path, content_hash, at_seq, saved_at_seq?}`. `content_hash` is the first 32 lowercase hexadecimal characters of the BLAKE3 hash of the saved file bytes; `at_seq` is the `seq` the next event of the log would take when the attestation is recorded (one more than the last event's `seq`; 0 for an empty log), written in decimal; `saved_at_seq` (optional, decimal) is the client's last known `seq` when it saved, informative. `attestation_id = H(doc_path + "\n" + content_hash + "\n" + at_seq)`. A line with another id, a `content_hash` that is not 32 lowercase hexadecimal characters, an `at_seq` greater than the `seq` the log's next event would take when the vault is opened, an `at_seq` lower than that of the line before it, or a `doc_path` that is not a vault-relative Markdown path MUST be rejected when the vault is opened. Recording an attestation whose id is already in the log is a no-op. The log holds no wall-clock time.

## 9. Indexes and retrieval (Validated, P1-C.1)

- A lexical index (BM25 or equivalent) MUST be provided and is part of the default retrieval path.
- A vector index MAY be provided. Hybrid fusion (reciprocal rank fusion) is the reference default.
- Retrieval over the same checkpoint with the same query and the same index configuration MUST return identical ranked results. Approximate nearest neighbour structures are permitted only if they satisfy this (fixed insertion order, exhaustive fallback, or exact search).
- Link-graph ranking is not part of this specification (no measured benefit over hybrid retrieval in the reference evaluation).
- When `proposal` chunks are requested, the `canonical` results MUST be identical to the results of the same query without the request (same chunks, order and scores). `proposal` chunks MUST be ranked in a separate pass whose statistics are computed over the `canonical` chunks of the queried checkpoint only; they MUST be labelled `proposal`, their scores multiplied by 0.5, and listed after every `canonical` result.
- `unverified` (2.0.0, §7.1): every rule of this section for `proposal` chunks applies to `unverified` chunks too — excluded by default; when `proposal` results are requested, `proposal` and `unverified` chunks are ranked together in the separate pass, each labelled with its own layer, and count together towards the `k` non-`canonical` results.
- A query that is empty or consists only of whitespace (Unicode `White_Space`) MUST be rejected with an error; no consumption trace is recorded for it.
- Response length (1.0.3): a retrieval for `k` results returns at most `k` `canonical` results; when `proposal` results are requested, at most `k` `proposal` results more — at most `2k` in all. (2.0.0: the `k` non-`canonical` results are `proposal` and `unverified` together.)
- Informative: lexical scoring is declarative. An implementation chooses its BM25 variant (or equivalent); the reference implementation records it in its index manifest as `lexical.variant`, `lexical.idf`, `lexical.k1`, `lexical.b` and `lexical.tokenizer` and refuses to open an index whose record is missing or differs. It records `lucene-bm25` (the BM25 of Lucene and tantivy), IDF `ln(1 + (N − n + 0.5)/(n + 0.5))`, `k1` 1.2, `b` 0.75, document length = token count, tokenizer `mdk_lower_alnum` (lower case, maximal runs of `[a-z0-9]`). Since 1.0.3 it also records the document length as `lexical.dl = "exact token count, f64, no fieldnorm quantization"`: the length of a chunk is its exact number of tokens, used as a 64-bit float, and the average over the `canonical` chunks of the checkpoint is not quantized (unlike Lucene's one-byte field norms), so another implementation can reproduce every score.
- Informative: the reference implementation (Phase 1 rounds P1-C, P1-C.1) takes the BM25 statistics `N`, `n` and the average length over the `canonical` chunks at the queried checkpoint, so that an index also holding other chunk versions ranks exactly as one rebuilt from that checkpoint; vectors are compared by exact cosine; hybrid is reciprocal rank fusion `1/(60 + rank)` of the top 50 of each side; ties are broken by `(doc_path, chunk_id)`. A chunk takes the lowest layer of its blocks as of the checkpoint (§7; `canonical` > `proposal` > `unverified` > `derived`); `proposal` and (2.0.0) `unverified` chunks are excluded by default and, when requested, ranked among themselves in one pass (hybrid: the top 50 such chunks of each side), labelled with their layer, their score multiplied by 0.5 and listed after the `canonical` results — up to `k` `canonical` and `k` others; `derived` chunks are never returned. A query only reads the index files. Every retrieval records one consumption trace (§8).

## 10. Determinism contract

For the same vault contents ingested in the same order with the same matcher version and parameters, two conforming implementations (or two runs of one) MUST produce byte-identical `events.jsonl`, identical `snapshot_id`s and `lineage_id`s, and identical retrieval results at any checkpoint. Evaluation truth MUST reference content `(doc_path, version_id, k)`, never lineage ids.

(2.0.0) Where §7.1 applies, "the same vault contents ingested in the same order" includes the attestations: the requirement above, and byte-identical `attestations.jsonl`, hold for two runs that observe the same external changes and record the same attestations in the same order, each attestation on the same side of its window (before or after its change, within or beyond `W`). The attestation decision depends on arrival order and timing only through these; the events record it, and replay never decides it again.

## 11. Raw journal and version triggers (Validated, P1-B)

Implementations SHOULD keep a prunable raw journal of every save (keystroke-level data is never lost) separate from versions. A **version** is created only by a trigger policy (idle time, semantic distance to the previous version, block boundary exit). The kernel defines bounds (a version never carries less than the journal; unversioned content is never indexed; minimum journal retention); policies supply parameters within those bounds. (Informative: the reference implementation's minimum journal retention is 24 hours.)

## 12. Conformance

- **Level 1 — Core:** §2–§6, §10 (events, lineage, chunks, determinism).
- **Level 2 — Governance:** Level 1 + §7–§8 (layers, promotion, checkpoints, traces).
- **Level 3 — Retrieval:** Level 2 + §9 (deterministic lexical and hybrid retrieval as of a checkpoint).

Conformance tests are listed in `spec/CONFORMANCE.md`. The reference implementation is the `rootmark` Rust workspace.
