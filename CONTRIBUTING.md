# Contributing to rootmark

Thank you for looking. rootmark is built in **rounds**: every change belongs to a round whose goal, exit criteria and thresholds are written down *before* the work starts, and every round ends with an independent verification. This page is the outside contributor's summary of [docs/DEV_PROCESS.md](docs/DEV_PROCESS.md) (Turkish, authoritative).

## The rules that do not bend

- **Criteria first.** A round's `docs/rounds/<id>/GOAL.md` (exit criteria, thresholds, decision rule) does not change once the round has started. If a result is bad, the result is reported, not the threshold.
- **The Markdown files are the truth.** Nothing writes identifiers or metadata into a Markdown file. Identifiers carry no time stamp and no randomness.
- **No silent failure.** An unresolved reference, a missing file or a manifest mismatch is an error.
- **Languages.** Runtime code is Rust only (`core`, `index`, `server`, `bindings`); Python only in `research/`; TypeScript only in `clients/`. Dependencies point one way: `core ← index ← server ← clients`; `research` only uses `bindings/python`.
- **Frozen surfaces.** `spec/SPEC.md` §1–§6 and the conformance fixtures change only through a decision record and a spec version bump. Measurement code (`research/eval/**`, `scripts/verify.sh`, the freeze checkers) is frozen within a round; an exception needs an entry in the round's `FREEZE_EXCEPTIONS.md`, a before/after bit-equality record and a reviewer's approval (`research/FROZEN.md`, `scripts/check_frozen.sh`).
- **Tests are not changed to make them pass.**
- **Pinned dependencies.** Exact versions, published at least 14 days before they are adopted (checked in CI).

## How a change lands

1. **Issue first.** A bug, a hypothesis to measure, or a decision request (`-T`), using the templates.
2. **A round or a task.** Small fixes go into the active round's tasks; larger work needs its own round directive (GOAL, TASKS, VERIFY, PROMPT; template in DEV_PROCESS §1).
3. **Progress records.** Each task keeps `docs/rounds/<id>/progress/<task>.md` (started / done / partial / parked), its acceptance criteria one by one, and a reviewer's note (`<task>.review.md`). One commit per task: `<round>/<task>: …`.
4. **Pull request.** Use the template: which round, which acceptance criterion, any freeze exception. CI must be green (tests, lints, conformance, `check_deps`, `check_frozen`, `check_compare_fields`) and `main` takes linear history only.
5. **Verification.** A round is "ready" only with `make test`, `make lint`, `make conformance` and `make verify` (two independent clones, determinism compare) clean. An independent verifier then reproduces it from a clean clone and gives one of "reliable", "reliable on conditions" or "not reliable". The release tag is a human step.

## Running the gates locally

```sh
make setup models        # toolchains, research environment, the pinned embedder model
make test                # cargo + pytest + vitest
make lint                # fmt, clippy -D warnings, ruff, eslint, dependency direction, freeze checks
make conformance         # spec/CONFORMANCE.md Levels 1–3
make verify              # scripts/verify.sh: two independent chains, compared field by field
```

Questions about direction belong in Discussions; decisions are recorded in [docs/DECISIONS.md](docs/DECISIONS.md).
