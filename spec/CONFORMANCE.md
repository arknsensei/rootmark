# mdkernel Conformance Tests v1.0

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

## Level 3 — Retrieval

| ID | Test | Expected |
|---|---|---|
| C3-01 | Lexical index present; `proposal` excluded/down-weighted and labelled | ranking equals fixture |
| C3-02 | Hybrid fusion deterministic: same checkpoint + query → identical ranking, 10 runs | equality |
| C3-03 | Index rebuild from checkpoint yields identical ranking | equality |
| C3-04 | Retrieval never returns `derived` as content | property |

## Reporting
A conformance run produces `conformance.json` with one entry per test (`pass`/`fail`, actual vs expected hash). Level is claimed only if all tests of that level and below pass.
