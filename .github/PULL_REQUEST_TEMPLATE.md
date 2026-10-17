## Round and task
- Round: `docs/rounds/<id>/` · Task: `<task>` · Commit prefix `<round>/<task>:`

## Acceptance criteria
<!-- one line per criterion of the task: met / not met, with the evidence (test, report field, command output) -->
- [ ] 

## Exit criteria touched
<!-- the GOAL rows this PR moves (code, measured value, threshold) — thresholds are never edited -->

## Freeze exception
- [ ] none
- [ ] yes: `docs/rounds/<id>/progress/FREEZE_EXCEPTIONS.md` entry `<id>/F<n>`, before/after bit-equality record, the `research/FROZEN.md` re-freeze row, reviewer approval (`<task>.review.md`)

## Checks
- [ ] `make test`
- [ ] `make lint` (includes `check_deps`, `check_frozen`, `check_compare_fields`)
- [ ] `make conformance`
- [ ] no identifier, metadata, time stamp or randomness written into Markdown or identities
- [ ] no secret in the diff
