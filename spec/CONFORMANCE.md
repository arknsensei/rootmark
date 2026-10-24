# rootmark Conformance Tests v1.0

Each test has a fixture under `spec/fixtures/` (added in Phase 1 round A) and an expected output. A conforming implementation passes every test of its claimed level. Tests are the executable form of SPEC.md; where they disagree, SPEC.md wins and the test is fixed via the decision record.

## Level 1 — Core

| ID | Test | Expected |
|---|---|---|
| C1-01 | Normalization: NFC, line endings, whitespace, code block preservation | `normalized_text` equals fixture |
| C1-02 | Block boundaries for every block type, front matter, nested list, empty paragraph | block list equals fixture |
| C1-03 | Wikilink forms (4) | targets and headings equal fixture |
| C1-04 | `version_id` stable across runs and machines | hash equals fixture |
| C1-05 | `lineage_id` formula incl. duplicate ordinal `k` | ids equal fixture |
| C1-06 | Object store: identical write is no-op; conflicting write fails | behaviour |
| C1-07 | Event log replay reproduces latest snapshot ids | equality |
| C1-08 | Matcher: exact + move (LIS) | events equal fixture |
| C1-09 | Matcher: duplicate blocks, context anchor then ordinal | lineage preserved for untouched copies |
| C1-10 | Matcher: split 2/3 pieces, dominant-piece rule | events, `meta.dominant` |
| C1-11 | Matcher: merge 2/3 blocks, dominant rule | events |
| C1-12 | Matcher: similarity update | `BlockUpdated`, parents |
| C1-13 | Matcher: residual checks (split and merge direction) | revoked match, correct event |
| C1-14 | Matcher: create/delete remainder | events |
| C1-15 | Mixed matcher versions in one vault rejected | error |
| C1-16 | Chunk rebuild changes no lineage and emits no events | equality |
| C1-17 | Whole-vault determinism: fixture vault ingested twice → identical `events.jsonl` bytes | equality |

## Level 2 — Governance

| ID | Test | Expected |
|---|---|---|
| C2-01 | Default layer by `author_kind` | `human→canonical`, `agent→proposal` |
| C2-02 | `Promoted` event requires actor, reason, policy_version | validation |
| C2-03 | Kernel refuses to promote without a hook decision | error |
| C2-04 | Demote/prune marks inactive, deletes nothing | object count unchanged |
| C2-05 | Checkpoint id formula, immutability | hash, write-once |
| C2-06 | Query as of checkpoint ignores later events | result equals fixture |
| C2-07 | Consumption trace written for every retrieval; id formula | trace line equals fixture |
| C2-08 | Layer transitions replayable from events | equality |
| C2-09 | Invalid trace input in `traces.jsonl` (`query_hash` of 31 or 33 characters, non-hex, uppercase; `consumer` containing `"\n"`) rejected when the vault is opened (§8, 1.0.1) | error |
| C2-10 | Provenance of events: change and delete events of an attributed ingest carry `meta.actor` and `meta.source`, events without them replay, malformed or unpaired values are rejected when the vault is opened (§7, 1.0.3) | events equal fixture; error |
| C2-11 | `unverified` when chosen (§7, §7.1, 2.0.0; title 2.0.3, the default is `canonical`): under `unattested = unverified` an unattested external change of a `canonical` document is ingested with `meta.layer = "unverified"` on its block events, `meta.actor.kind = "unattested_fs"` on its change and delete events and a `LayerChanged` → `unverified` for every lineage it changes; the same change under `unattested = canonical` is ingested as `human`, `canonical`; an attested change as `canonical`; a log written before 2.0.0 replays with unchanged layers and fingerprint; a layer value or actor kind outside §7, `meta.layer` on an event that gives no layer, and an `unattested_fs` change without `meta.layer = "unverified"` are rejected when the vault is opened | events equal fixture; error |
| C2-12 | Attestation matching (§7.1, §8, 2.0.0): with explicit observation and recording times, an attestation within `W` before the observation → `canonical`; within `W` after → `unverified` then `LayerChanged` with `cause: late_attestation` to the layer an attested ingest gives (`canonical`; `proposal` for a `proposal` lineage the change touched); beyond `W`, or of another `content_hash` → `unverified` stays; `attestation_id` formula; an invalid `attestations.jsonl` line, and a raising `LayerChanged` without a matching attestation, are rejected when the vault is opened | events and attestation log equal fixture; error |
| C2-13 | Partial match (§7.1, §8, 2.0.1; P2-0.1): an attestation carrying the saved bytes, observed together with other bytes in one change → the saved bytes ingested as an attested change (`canonical`), the line `partial: true`, then the rest unattested (`unverified`); attested late → only the lineages holding the saved blocks are raised; an exact match with content is an ordinary match; a copy of an existing block is never raised; content that is not the attested bytes is refused; a change without the saved blocks stays `unverified`; `partial` other than `true` is rejected when the vault is opened | events and attestation log equal fixture; error |
| C2-14 | Rename and link update (§7.1 "Link update", "Rename", 2.0.2; P2-0.2): a rename of a note holding an agent's `unverified` paragraph, its own links (wikilink and Markdown link with the note's name as label) rewritten in the same attestation → every block keeps its layer (human `canonical`, agent `unverified`); another note's Markdown link whose label is the old name follows its target → accepted, `canonical`; a label that is not the name → refused; a rename attested late → only what was `canonical` at the old path is raised | layers, events and attestation log equal fixture; error |

## Level 3 — Retrieval

| ID | Test | Expected |
|---|---|---|
| C3-01 | Lexical index present; `proposal` excluded/down-weighted and labelled | ranking equals fixture |
| C3-02 | Hybrid fusion deterministic: same checkpoint + query → identical ranking, 10 runs | equality |
| C3-03 | Index rebuild from checkpoint yields identical ranking | equality |
| C3-04 | Retrieval never returns `derived` as content | property |
| C3-05 | With `proposal` results requested, the `canonical` prefix equals the result without the request, every mode (§9, 1.0.2) | property |
| C3-06 | Canonical invariance: agent writes allowed by the default edit policy leave the `canonical` ranking of every query unchanged, every mode; writes touching `canonical` content are denied (§7, §9, 1.0.3) | decisions and rankings equal fixture |
| C3-07 | Response length with `proposal` results requested: at most `k` per layer, `2k` in all, every mode (§9, 1.0.3; since 2.0.0 the `k` non-`canonical` results are `proposal` and `unverified` together) | property |
| C3-08 | `unverified` in retrieval (§7.1, §9, 2.0.0): `unverified` chunks are excluded by default; with `proposal` results requested they are ranked with the `proposal` chunks after every `canonical` result, labelled `unverified`, score ×0.5, the `canonical` prefix unchanged, at most `k` non-`canonical` results, every mode | ranking equals fixture; property |

## Reporting
A conformance run produces `conformance.json` with one entry per test (`pass`/`fail`, actual vs expected hash). Level is claimed only if all tests of that level and below pass.
