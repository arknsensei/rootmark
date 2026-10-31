#!/usr/bin/env bash
# P2-V Mac set (`make verify-mac`; GOAL P2-V V2, V-6; K-P2V.1): the
# timing-sensitive (Z) and real-Obsidian/macOS (O) sets of the full chain,
# which the tier 2 pair leaves out of its byte comparison by name
# (reports/rounds/P2-V/inventory.json `mac_sets`), each measured 3 times on
# this Mac with its own round's threshold.
# P2-V.1/F7 (K-P2V.3 T2 (b)): the deterministic O criteria — C1-8 (the
# chain's steps holdout … p1c1-checks) and E-7 (release_local.sh) — run in
# the first repetition only (E1-4 is read from that repetition too); the
# reading is P2-V.1's inventory's `mac_reading` (eval.verify_mac).
#
# A clean clone of this repository's committed HEAD, `make setup`, the
# models, the conformance report, the session harness and the
# serve_watch test binary are built once; then every repetition is its own
# RUN and runs, in this order, the chain's own commands for the sets:
#   - `eval.run_p2_0_2 --phase test --obsidian /nonexistent/obsidian` (as
#     `make p2a-checks` runs it: D-A, D-A2, D-C, D-B2 under `serve --watch`;
#     D-E and the rules with them),
#   - `eval.run_p2_0_3 test` (the real Obsidian: D-B test-4, D-A′, D-C′),
#   - the chain's steps `holdout` … `p1c1-checks` (P1-C.1 C1-8 against the
#     P1-C RUN: its vectors and scores are bound to the release Mac),
#   - the `serve_watch` test (P1-D D-10 and the SIGTERM case),
#   - `release_local.sh` (P1-E E-7), `release.sh` with the real-Obsidian e2e
#     and `release_check.sh` (P1-E.1 E1-3, E1-4, E1-9's e2e step),
#     `test_tamper_p1e1_e2e.sh` (the e2e's deliberate cases).
# `eval.verify_mac rep` applies the frozen recorders' formulas to the
# repetition (written to <RUN>/mac/rep<i>/), `eval.verify_mac summary` the
# 3/3 rule (<RUN>/mac_checks.json). A set that fails is recorded, not
# fatal; a harness that stops is a failed repetition of its sets (its
# error recorded); any other missing record fails. Nothing is optimised: a set that does not
# hold its threshold here is reported (GOAL P2-V decision rule).
#
#   scripts/verify_mac.sh [<workdir>]   (default: a fresh temp dir)
# Prints "RUN=<id>" and "RUN_DIR=<path>".
set -euo pipefail

log() { printf '[verify-mac %s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2; }

reps=3
[ "$(uname -s)" = Darwin ] || { echo "verify-mac: the Mac set runs on macOS (FSEvents, the real Obsidian)" >&2; exit 1; }
src="$(git rev-parse --show-toplevel)"
work="${1:-$(mktemp -d -t rootmark-verify-mac.XXXXXX)}"
mkdir -p "$work"
work="$(cd "$work" && pwd)"
[ ! -e "$work/repo" ] || { echo "error: $work/repo already exists; use an empty directory" >&2; exit 1; }
if [ -n "$(git -C "$src" status --porcelain --untracked-files=no)" ]; then
  log "warning: $src has uncommitted changes; only its committed HEAD is measured"
fi
t0=$SECONDS
log "clone $src -> $work/repo"
git clone --quiet "$src" "$work/repo"
cd "$work/repo"
log "commit $(git rev-parse HEAD)"
make setup >"$work/setup.log" 2>&1 || { tail -40 "$work/setup.log" >&2; exit 1; }
make models >"$work/models.log" 2>&1 || { tail -20 "$work/models.log" >&2; exit 1; }
make datasets >"$work/datasets.log" 2>&1 || { tail -20 "$work/datasets.log" >&2; exit 1; }
make conformance >"$work/conformance.log" 2>&1 || { tail -20 "$work/conformance.log" >&2; exit 1; }
(cd clients/obsidian && npm ci --no-audit --no-fund) >"$work/npm.log" 2>&1 || { tail -20 "$work/npm.log" >&2; exit 1; }
make p2-0-2-harness >"$work/harness.log" 2>&1 || { tail -20 "$work/harness.log" >&2; exit 1; }
cargo test -q -p rootmark-server --release --test serve_watch --no-run >"$work/serve_watch_build.log" 2>&1 \
  || { tail -20 "$work/serve_watch_build.log" >&2; exit 1; }
newrun() { (cd research && uv run --frozen python -m eval.runinfo); }
run="$(newrun)"
log "RUN=$run (setup $((SECONDS - t0)) s)"
main="research/reports/runs/$run"
mkdir -p "$main/mac"
prev="$run"
t1=$SECONDS
for i in $(seq 1 "$reps"); do
  rr="$(newrun)"
  while [ "$rr" = "$prev" ]; do sleep 1; rr="$(newrun)"; done
  prev="$rr"
  d="research/reports/runs/$rr"
  mkdir -p "$d/p1d" "$d/p1e1"
  log "rep $i/$reps: RUN $rr"
  ts=$SECONDS
  jq --arg r "$(pwd -P)/" '.fixtures_root |= (if startswith($r) then .[($r | length):] else . end)' \
    reports/conformance.json >"$d/conformance_p1d1.json"
  log "rep $i: P2-0.2 sets under serve --watch (D-A, D-A2, D-C, D-B2; D-E, rules)"
  # a harness that stops (e.g. the real Obsidian not writing a note) is a
  # failed repetition of its sets, recorded with its error — never skipped
  errs=()
  (cd research && uv run --frozen python -m eval.run_p2_0_2 --phase test --run "$rr" \
    --rootmark ../target/release/rootmark --obsidian /nonexistent/obsidian) >"$d/p2_0_2.log" 2>&1 \
    || { errs+=(--error "p2_0_2=$d/p2_0_2.log"); log "rep $i: run_p2_0_2 stopped (recorded)"; tail -5 "$d/p2_0_2.log" >&2; }
  log "rep $i: P2-0.3 real-Obsidian sets (D-B test-4, D-A′, D-C′)"
  (cd research && uv run --frozen python -m eval.run_p2_0_3 test --run "$rr" \
    --rootmark ../target/release/rootmark) >"$d/p2_0_3.log" 2>&1 \
    || { errs+=(--error "p2_0_3=$d/p2_0_3.log"); log "rep $i: run_p2_0_3 stopped (recorded)"; tail -5 "$d/p2_0_3.log" >&2; }
  # P2-V.1/F7 (K-P2V.3 T2 (b); inventory `mac_reading`): the deterministic O
  # criteria (C1-8, E-7, E1-4) run once, in the first repetition
  if [ "$i" -eq 1 ]; then
  log "rep $i: P1-C.1 C1-8 (the chain's steps holdout … p1c1-checks, against the P1-C RUN; once)"
  if ! { make eval-holdout eval-split-grid RUN="$rr" && make regress RUN="$rr" && make p1a-checks RUN="$rr" \
    && make p1b-checks RUN="$rr" && make p1b1-checks RUN="$rr" && make p1c-checks RUN="$rr" P1C_WORK="$work/p1c_work_$i" \
    && make p1c1-checks RUN="$rr"; } >"$d/p1c.log" 2>&1; then
    errs+=(--error "p1c=$d/p1c.log"); log "rep $i: the P1-C steps stopped (recorded)"; tail -5 "$d/p1c.log" >&2
  fi
  fi
  log "rep $i: serve_watch (D-10, SIGTERM)"
  ROOTMARK_D10_OUT="$(pwd)/$d/p1d/d10.json" ROOTMARK_SIGTERM_OUT="$(pwd)/$d/p1d/sigterm.json" \
    cargo test -q -p rootmark-server --release --test serve_watch -- --test-threads=1 >"$d/p1d/serve_watch.log" 2>&1 \
    || log "rep $i: serve_watch exit $? (recorded)"
  [ -f "$d/p1d/d10.json" ] && [ -f "$d/p1d/sigterm.json" ] || { tail -30 "$d/p1d/serve_watch.log" >&2; exit 1; }
  log "rep $i: local release binary (E-7), release package with the real-Obsidian e2e and its check (E1-3, E1-4, E1-9), the e2e's deliberate cases"
  mkdir -p "$d/p1e" "$d/p1e1/release"
  if [ "$i" -eq 1 ]; then
  bash scripts/release_local.sh "$d/p1e/release" >"$d/p1e/release.log" 2>&1 || log "rep $i: release_local exit $? (recorded)"
  [ -f "$d/p1e/release/release_local.json" ] || { tail -30 "$d/p1e/release.log" >&2; exit 1; }
  fi
  dist="$work/dist_$i"
  ROOTMARK_RELEASE_LINUX=0 bash scripts/release.sh "$dist" >"$d/p1e1/release.log" 2>&1 || log "rep $i: release.sh exit $? (recorded)"
  bash scripts/release_check.sh "$dist" >"$d/p1e1/release_check.log" 2>&1 || log "rep $i: release_check exit $? (recorded)"
  [ -f "$dist/release_check.json" ] && [ -f "$dist/plugin_e2e_real/plugin_e2e_real.json" ] || { tail -30 "$d/p1e1/release.log" >&2; exit 1; }
  cp "$dist/release.json" "$dist/release_check.json" "$dist/plugin_e2e_real/plugin_e2e_real.json" "$d/p1e1/release/"
  bash scripts/test_tamper_p1e1_e2e.sh "$d/p1e1/tamper_p1e1_e2e.json" >"$d/p1e1/tamper_p1e1_e2e.log" 2>&1 || log "rep $i: tamper e2e exit $? (recorded)"
  [ -f "$d/p1e1/tamper_p1e1_e2e.json" ] || { tail -30 "$d/p1e1/tamper_p1e1_e2e.log" >&2; exit 1; }
  (cd research && uv run --frozen python -m eval.verify_mac rep --run "$rr" --main "$run" --rep "$i" \
    --elapsed "$((SECONDS - ts))" ${errs[@]+"${errs[@]}"}) >&2
done
# a set that failed is the verdict (exit 3), recorded: the RUN lines and the
# work-tree check still come (P2-V/F4)
src=0
(cd research && uv run --frozen python -m eval.verify_mac summary --run "$run" --reps "$reps" \
  --elapsed "$((SECONDS - t1))" --setup "$((t1 - t0))") >&2 || src=$?
[ "$src" -eq 0 ] || [ "$src" -eq 3 ] || { echo "error: the Mac set's summary failed (exit $src)" >&2; exit "$src"; }
if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
  echo "error: the Mac set ran on a dirty work tree" >&2
  git status --porcelain --untracked-files=all >&2
  exit 1
fi
log "finished in $((SECONDS - t0)) s"
echo "RUN=$run"
echo "RUN_DIR=$work/repo/$main"
exit "$src"
