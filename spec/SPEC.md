# mdkernel Specification v1.0.1

Status: **Frozen for §1–§6 (validated by prototype rounds v1–v3, Oct 2026); Validated for §7, §8, §11 (Phase 1 round P1-B, Oct 2026; conformance Level 2); Draft for §9–§10 (to be validated in Phase 1).** Version 1.0.1 (Phase 1 round P1-B.1): §7 undeclared `synthetic` → `canonical`; §8 `trace_id` with separators and input validation; §7/§8 serialization normative. §1–§6 are unchanged. License: Apache-2.0 (specification). Language: English, for publication; implementation directives are maintained in Turkish under `docs/`.

The key words MUST, MUST NOT, SHOULD, MAY are to be interpreted as in RFC 2119.

## 0. Scope and relationship to prior work

mdkernel specifies a block-level lineage, provenance and governance layer over plain Markdown files. Markdown files are the single source of truth; every structure defined here is derived from them and can be rebuilt from them.

ContextNest (2026) specifies document-level governance for Markdown context (typed documents, hash-chained version history, checkpoints, publication gate, consumption traces). mdkernel adopts its vocabulary where meanings coincide and extends governance to the block level with lineage inferred from diffs, deterministic derived indexes and incremental indexing. mdkernel does not reuse ContextNest's implementation. A ContextNest-compatible export is a possible extension and is out of scope here.

Non-goals: realtime multi-user editing (Phase 2), encrypted/zero-knowledge servers, corpora beyond ~10^5 documents, entity extraction and graph ranking (research).

## 1. Terminology

- **Vault**: a directory tree of Markdown files plus a derived store under `<vault>/.mdkernel/`.
- **Document**: one Markdown file, addressed by `doc_path` (vault-relative, `/`-separated).
- **Block**: the structural atom of Markdown (§2). Immutable, content-addressed.
- **Version**: a concrete block content, identified by `version_id`.
- **Lineage**: the identity of a block across versions, identified by `lineage_id`.
- **Snapshot**: the ordered list of `(lineage_id, version_id)` of a document at a point.
- **Event**: an append-only record of a change to a document's blocks.
- **Checkpoint**: an immutable vault-wide map `doc_path → snapshot_id` with an event-log offset.
- **Layer**: the governance status of a version: `canonical`, `proposal`, `derived`.
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
`.mdkernel/objects/<vid[0:2]>/<vid>.json` (blocks), `.mdkernel/snapshots/<sid>.json`, `.mdkernel/checkpoints/<cid>.json`. Writing an existing identifier with different bytes MUST fail; identical bytes MUST be a no-op.

### 4.2 Event log
`.mdkernel/events.jsonl`, append-only. Each event carries `seq` (monotonic, vault-wide, no gaps), `doc_path`, `kind`, `lineage_ids`, `old_version_ids`, `new_version_ids`, `snapshot_id`, `meta`. Kinds: `BlockCreated`, `BlockUpdated`, `BlockMoved`, `BlockSplit`, `BlockMerged`, `BlockDeleted`, `SnapshotCommitted`, and (§7–§8) `LayerChanged`, `Promoted`, `Demoted`, `CheckpointCreated`.

Replaying the log MUST reconstruct every document's latest snapshot with an identical `snapshot_id`. The BLAKE3 hash of the whole log is the vault's determinism fingerprint (§10).

### 4.3 Provenance
Every version carries `provenance = {author_kind ∈ {human, agent, transcript, synthetic}, source}`. Provenance is immutable.

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

Every version has `layer ∈ {canonical, proposal, derived}`; layers MUST be serialized in lowercase, as written here. Default on ingest: `human` → `canonical`; `agent`, `transcript` → `proposal`; `synthetic` → as declared, and `canonical` if no layer is declared.

- `Promoted` event: `proposal → canonical`; carries `actor` (human id or policy name), `reason`, and `policy_version`. The kernel MUST NOT decide promotion; it exposes a hook. A `proposal` MAY be `Demoted` or pruned by policy; pruning never deletes objects, only marks the lineage inactive.
- Retrieval defaults: `canonical` full weight; `proposal` MUST be excluded or clearly down-weighted and labelled; `derived` is never returned as content.
- Layer transitions are events and therefore replayable and auditable.

## 8. Checkpoints and consumption traces (Validated, P1-B)

- **Checkpoint**: `{checkpoint_id, name?, event_seq, docs: {doc_path: snapshot_id}}`. Immutable. Any query MAY be evaluated "as of" a checkpoint; the result MUST be identical for the same checkpoint and query (§10).
- **Consumption trace**: `.mdkernel/traces.jsonl`, append-only: `{trace_id, checkpoint_id, consumer, query_hash, returned: [(doc_path, lineage_id, version_id)], at_seq}`; each `returned` entry MUST be serialized as an object with the keys `doc_path`, `lineage_id`, `version_id`. `trace_id = H(checkpoint_id + "\n" + consumer + "\n" + query_hash + "\n" + at_seq)`, where `at_seq` is written in decimal. `query_hash` MUST be exactly 32 lowercase hexadecimal characters and `consumer` MUST NOT contain `"\n"`; a trace violating either MUST be rejected. Traces answer "which block versions did this answer depend on".

## 9. Indexes and retrieval (Draft)

- A lexical index (BM25 or equivalent) MUST be provided and is part of the default retrieval path.
- A vector index MAY be provided. Hybrid fusion (reciprocal rank fusion) is the reference default.
- Retrieval over the same checkpoint with the same query and the same index configuration MUST return identical ranked results. Approximate nearest neighbour structures are permitted only if they satisfy this (fixed insertion order, exhaustive fallback, or exact search).
- Link-graph ranking is not part of this specification (no measured benefit over hybrid retrieval in the reference evaluation).
- Informative: the reference implementation (Phase 1 round P1-C) ranks with BM25 (`k1` 1.2, `b` 0.75; IDF `ln(1 + (N − n + 0.5)/(n + 0.5))`) whose statistics `N`, `n` and the average length are taken over the chunks eligible at the queried checkpoint, so that an index also holding other chunk versions ranks exactly as one rebuilt from that checkpoint; vectors are compared by exact cosine; hybrid is reciprocal rank fusion `1/(60 + rank)` of the top 50 of each side; ties are broken by `(doc_path, chunk_id)`. A chunk takes the lowest layer of its blocks as of the checkpoint (§7); `proposal` chunks are excluded by default and, when requested, labelled, their score multiplied by 0.5 and listed after `canonical` results; `derived` chunks are never returned. Every retrieval records one consumption trace (§8).

## 10. Determinism contract

For the same vault contents ingested in the same order with the same matcher version and parameters, two conforming implementations (or two runs of one) MUST produce byte-identical `events.jsonl`, identical `snapshot_id`s and `lineage_id`s, and identical retrieval results at any checkpoint. Evaluation truth MUST reference content `(doc_path, version_id, k)`, never lineage ids.

## 11. Raw journal and version triggers (Validated, P1-B)

Implementations SHOULD keep a prunable raw journal of every save (keystroke-level data is never lost) separate from versions. A **version** is created only by a trigger policy (idle time, semantic distance to the previous version, block boundary exit). The kernel defines bounds (a version never carries less than the journal; unversioned content is never indexed; minimum journal retention); policies supply parameters within those bounds. (Informative: the reference implementation's minimum journal retention is 24 hours.)

## 12. Conformance

- **Level 1 — Core:** §2–§6, §10 (events, lineage, chunks, determinism).
- **Level 2 — Governance:** Level 1 + §7–§8 (layers, promotion, checkpoints, traces).
- **Level 3 — Retrieval:** Level 2 + §9 (deterministic lexical and hybrid retrieval as of a checkpoint).

Conformance tests are listed in `spec/CONFORMANCE.md`. The reference implementation is the `mdkernel` Rust workspace.
