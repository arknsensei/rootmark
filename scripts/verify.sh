#!/usr/bin/env bash
# Independent end-to-end reproduction (docs/DEV_PROCESS.md §5,
# docs/rounds/P1-B/VERIFY.md; P1-B TASKS B9; P1-C TASKS C9).
#
# P1-C (re-freeze P1-C/F8): after `make setup` the chain fetches and checks
# the embedder model (`make models`); conformance is level 3 (30 tests);
# after p1b-checks it records `make p1b1-checks` (P1-B.1 set, C-11 input)
# and `make p1c-checks` (C-1..C-13 into p1c_checks.json; the D3
# determinism vault in <workdir>/p1c_work/<RUN>/.mdkernel/index/) and runs
# the P1-C deliberate corruptions (`scripts/test_tamper_p1c.sh`); `pair`
# compares A ↔ B with `eval.compare_p1c pair` (the P1-B set plus the P1-B.1
# and P1-C files, the 200 rankings and every index file hash, the
# governance vault bytes) and `compare-p1b` is kept unchanged.
#
# P1-C.1 (re-freeze P1-C.1/F6): after the P1-C corruptions the chain records
# `make p1c1-checks` (C1-1 Python variant, C1-2..C1-6 tests, the CLI
# deliberate tests `scripts/test_p1c1.sh`, C1-8 regress against the P1-C
# official RUN, C1-9 → p1c1_checks.json); `pair` compares A ↔ B with
# `eval.compare_p1c1 pair` (the P1-C pair set plus c1_variant.json and
# p1c1_checks.json, no exclusion); `compare-p1c` is kept unchanged and
# `compare-p1c1` is new.
#
# P1-D (re-freeze P1-D/F4): after the P1-C.1 checks the chain records `make
# p1d-checks` (D-1..D-11: the official MCP Inspector — Node.js and network
# needed —, the MCP e2e, the HTTP/service tests, the watcher latency and
# SIGTERM tests, the agent scene, the P1-D deliberate corruptions, D-9
# against the P1-C.1 official RUN → p1d_checks.json); `pair` compares A ↔ B
# with `eval.compare_p1d pair` (the P1-C.1 pair set plus the P1-D files and
# the P1-D artefacts byte for byte: MCP transcripts, the e2e vault's
# events.jsonl/traces.jsonl/proposals.jsonl and Markdown, the scene, the
# Inspector's tool list); `compare-p1c1` is kept unchanged and `compare-p1d`
# is new.
#
#   scripts/verify.sh chain <source-repo> <workdir>
#       clone <source-repo> into <workdir>/repo, `make setup`, rebuild and
#       verify every research dataset against its committed manifest (d1hs
#       included), run both freeze checkers and `scripts/check_frozen.sh`,
#       `make conformance` (level 2), then under one RUN id: `make regress`
#       (v2 phases, v3 D1-easy/hard, D2, H3, H4, H4-V2 compared with the
#       v3-final reference run; P1-A holdout and THETA_SPLIT grid compared
#       with the P1-A official RUN; d1hs holdout-split), `make p1b-checks`
#       (conformance, v3-final compat, policy bounds, governance fuzz with two
#       seeds, the deterministic governance scenario, dependency direction
#       → p1b_checks.json) and the deliberate corruptions
#       (`scripts/test_tamper.sh`: fixtures, holdout data, RUN files, moved
#       target/ B-7, FROZEN.md rows B-8; its log goes into the RUN); prints
#       "RUN=<id>" and "RUN_DIR=<path>".
#   scripts/verify.sh pair <source-repo> <workdir-A> <workdir-B>
#       two independent chains, then `compare` A ↔ B.
#   scripts/verify.sh compare <repo> <run-dir-A> <run-dir-B> [<out.json>]
#       research/eval/compare_p1a.py (P1-A set: v3 + holdout + split_grid;
#       unchanged P1-A semantics) run inside <repo>/research.
#   scripts/verify.sh compare-p1b <repo> <run-dir-A> <run-dir-B> [<out.json>]
#       research/eval/compare_p1b.py (P1-A set + holdout_split +
#       p1b_checks + governance summary, and the governance vault's
#       events.jsonl, traces.jsonl and checkpoints/ byte for byte) run inside
#       <repo>/research; exits non-zero when any quality field or artefact
#       differs (B-4) or a file/field is missing.
#   scripts/verify.sh            (what `make verify` runs)
#       pair mode with <source-repo> = this repository and two fresh temp dirs.
#
# Nothing here refers to the main working directory except as the clone
# source; every path is derived from the arguments. Any failing step fails
# the whole run (set -e; each chain is its own process).
set -euo pipefail

log() { printf '[verify %s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2; }

chain() {
  local src="$1" work="$2"
  # stdout carries only the RUN=/RUN_DIR= result lines; everything else → stderr
  exec 3>&1 1>&2
  mkdir -p "$work"
  work="$(cd "$work" && pwd)"
  if [ -e "$work/repo" ]; then
    echo "error: $work/repo already exists; use an empty directory" >&2
    exit 1
  fi
  local t0=$SECONDS
  if [ -n "$(git -C "$src" status --porcelain --untracked-files=no 2>/dev/null)" ]; then
    log "warning: $src has uncommitted changes; only its committed HEAD is verified"
  fi
  log "clone $src -> $work/repo"
  git clone --quiet "$src" "$work/repo"
  cd "$work/repo"
  log "commit $(git rev-parse HEAD)"
  log "setup"
  make setup >"$work/setup.log" 2>&1 || { tail -40 "$work/setup.log" >&2; exit 1; }
  log "models (fetch + manifest check)"
  make models >"$work/models.log" 2>&1 || { tail -20 "$work/models.log" >&2; exit 1; }
  log "datasets (d2 d3 d1 d1k4 d1h d1ho d1sh d1hs, manifests verified)"
  make datasets
  log "freeze-check (v2), freeze-check-v3, check_frozen"
  make freeze-check
  make freeze-check-v3
  bash scripts/check_frozen.sh
  log "conformance (level 3)"
  make conformance
  local run
  run="$(cd research && uv run --frozen python -m eval.runinfo)"
  log "RUN=$run"
  log "holdout (P1-A A-2) and THETA_SPLIT grid (A-7)"
  make eval-holdout RUN="$run"
  make eval-split-grid RUN="$run"
  log "regress (v2 + v3 phases vs v3-final; holdout + split grid vs P1-A; holdout-split)"
  make regress RUN="$run"
  log "A-1 / A-5 / A-6 into the RUN (level-1 view)"
  make p1a-checks RUN="$run"
  log "governance: conformance, v3-final compat, policy bounds, fuzz, scenario (p1b_checks)"
  make p1b-checks RUN="$run"
  log "P1-B.1 checks (B1-1..B1-6 recorded again; C-11 input)"
  make p1b1-checks RUN="$run"
  log "P1-C checks (C-1..C-13; D3 determinism vault under $work/p1c_work)"
  make p1c-checks RUN="$run" P1C_WORK="$work/p1c_work"
  log "P1-C deliberate corruptions (model manifest, embedder_id, fixtures)"
  bash scripts/test_tamper_p1c.sh >"research/reports/runs/$run/tamper_p1c.log" 2>&1 \
    || { cat "research/reports/runs/$run/tamper_p1c.log" >&2; exit 1; }
  tail -1 "research/reports/runs/$run/tamper_p1c.log" >&2
  log "P1-C.1 checks (C1-1..C1-9; CLI deliberate tests: lock file, empty query, manifest fields)"
  make p1c1-checks RUN="$run"
  tail -1 "research/reports/runs/$run/p1c1_cli.log" >&2
  log "P1-D checks (D-1..D-11: Inspector, MCP e2e, HTTP/service, watch, scene, tamper; D-9 vs P1-C.1)"
  make p1d-checks RUN="$run"
  log "deliberate corruptions (B-7, B-8 and the P1-A cases)"
  bash scripts/test_tamper.sh "research/reports/runs/$run" >"research/reports/runs/$run/tamper.log" 2>&1 \
    || { cat "research/reports/runs/$run/tamper.log" >&2; exit 1; }
  tail -1 "research/reports/runs/$run/tamper.log" >&2
  # every phase must have run on the clean committed tree (nothing written into it)
  local dirty
  dirty="$(jq -r '.phases | to_entries[] | select(.value.worktree_clean != true) | .key' \
    "research/reports/runs/$run/manifest.json")"
  if [ -n "$dirty" ]; then
    echo "error: phases ran on a dirty work tree: $(echo $dirty)" >&2
    git status --porcelain --untracked-files=all >&2
    exit 1
  fi
  log "chain finished in $((SECONDS - t0)) s"
  echo "RUN=$run" >&3
  echo "RUN_DIR=$work/repo/research/reports/runs/$run" >&3
}

pair() {
  local src="$1" a="$2" b="$3"
  local self res_a res_b dir_a dir_b
  self="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
  res_a="$(mktemp -t mdk-verify-a.XXXXXX)"
  res_b="$(mktemp -t mdk-verify-b.XXXXXX)"
  # each chain is its own process: a failing step fails the whole pair
  # (errexit is not inherited inside $(...) on bash 3.2)
  bash "$self" chain "$src" "$a" >"$res_a"
  bash "$self" chain "$src" "$b" >"$res_b"
  dir_a="$(sed -n 's/^RUN_DIR=//p' "$res_a")"
  dir_b="$(sed -n 's/^RUN_DIR=//p' "$res_b")"
  for d in "$dir_a" "$dir_b"; do
    if [ -z "$d" ] || [ ! -f "$d/manifest.json" ]; then
      echo "error: chain produced no run directory ($d)" >&2
      exit 1
    fi
  done
  cat "$res_a" "$res_b"
  compare_p1d "$(cd "$a" && pwd)/repo" "$dir_a" "$dir_b" "$dir_a/pair_compare.json"
}

compare() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1a "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1a exit $rc)" >&2
    exit "$rc"
  fi
  log "max relative difference of quality fields = 0"
}

compare_p1c() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-C) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1c pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1c pair exit $rc)" >&2
    exit "$rc"
  fi
  log "quality fields, rankings and index file hashes equal; traces.jsonl, checkpoints/, events.jsonl byte-equal"
}

compare_p1c1() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-C.1) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1c1 pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1c1 pair exit $rc)" >&2
    exit "$rc"
  fi
  log "quality fields, rankings, index file hashes, P1-C.1 checks equal; traces.jsonl, checkpoints/, events.jsonl byte-equal"
}

compare_p1d() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare (P1-D) $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1d pair "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1d pair exit $rc)" >&2
    exit "$rc"
  fi
  log "P1-C.1 set and P1-D results equal; MCP transcripts, e2e vault, scene, Inspector list byte-equal"
}

compare_p1b() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "compare $dir_a $dir_b"
  local rc=0
  (cd "$repo/research" && uv run --frozen python -m eval.compare_p1b "$dir_a" "$dir_b" \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: run comparison failed (compare_p1b exit $rc)" >&2
    exit "$rc"
  fi
  log "max relative difference of quality fields = 0; traces.jsonl, checkpoints/, events.jsonl byte-equal"
}

case "${1:-}" in
  chain) chain "${2:?source repo}" "${3:?workdir}" ;;
  pair) pair "${2:?source repo}" "${3:?workdir A}" "${4:?workdir B}" ;;
  compare) compare "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1b) compare_p1b "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1c) compare_p1c "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1c1) compare_p1c1 "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  compare-p1d) compare_p1d "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  "")
    src="$(git rev-parse --show-toplevel)"
    base="$(mktemp -d -t mdk-verify.XXXXXX)"
    log "workdirs under $base"
    pair "$src" "$base/A" "$base/B"
    ;;
  *)
    echo "usage: $0 [chain <src> <dir> | pair <src> <dirA> <dirB> | compare|compare-p1b|compare-p1c|compare-p1c1|compare-p1d <repo> <runA> <runB> [out]]" >&2
    exit 2
    ;;
esac
