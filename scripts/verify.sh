#!/usr/bin/env bash
# Independent end-to-end reproduction (docs/DEV_PROCESS.md §5, docs/rounds/P1-A/VERIFY.md).
#
#   scripts/verify.sh chain <source-repo> <workdir>
#       clone <source-repo> into <workdir>/repo, `make setup`, rebuild and
#       verify every research dataset against its committed manifest, run both
#       freeze checkers, `make conformance`, then under one RUN id the quality
#       regression (`make regress`: v2 phases, v3 D1-easy/hard, D2, H3, H4,
#       H4-V2, compared field by field with the v3-final reference run), the
#       holdout (A-2), the THETA_SPLIT grid (A-7) and `make p1a-checks` (A-1,
#       A-5, A-6 into the RUN); prints "RUN=<id>" and "RUN_DIR=<path>".
#   scripts/verify.sh pair <source-repo> <workdir-A> <workdir-B>
#       two independent chains, then `compare` A ↔ B.
#   scripts/verify.sh compare <repo> <run-dir-A> <run-dir-B> [<out.json>]
#       research/eval/compare_p1a.py (v3 result set + holdout + split_grid)
#       run inside <repo>/research; exits non-zero when any quality field
#       differs (A-4) or a file/field is missing.
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
  log "datasets (d2 d3 d1 d1k4 d1h, manifests verified)"
  make datasets
  log "freeze-check (v2) and freeze-check-v3"
  make freeze-check
  make freeze-check-v3
  log "conformance (level 1)"
  make conformance
  local run
  run="$(cd research && uv run --frozen python -m eval.runinfo)"
  log "RUN=$run"
  log "regress (v2 + v3 phases, compared with v3-final)"
  make regress RUN="$run"
  log "holdout (A-2) and THETA_SPLIT grid (A-7)"
  make eval-holdout RUN="$run"
  make eval-split-grid RUN="$run"
  log "A-1 / A-5 / A-6 into the RUN"
  make p1a-checks RUN="$run"
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
  compare "$(cd "$a" && pwd)/repo" "$dir_a" "$dir_b" "$dir_a/pair_compare.json"
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

case "${1:-}" in
  chain) chain "${2:?source repo}" "${3:?workdir}" ;;
  pair) pair "${2:?source repo}" "${3:?workdir A}" "${4:?workdir B}" ;;
  compare) compare "${2:?repo}" "${3:?run dir A}" "${4:?run dir B}" "${5:-}" ;;
  "")
    src="$(git rev-parse --show-toplevel)"
    base="$(mktemp -d -t mdk-verify.XXXXXX)"
    log "workdirs under $base"
    pair "$src" "$base/A" "$base/B"
    ;;
  *)
    echo "usage: $0 [chain <src> <dir> | pair <src> <dirA> <dirB> | compare <repo> <runA> <runB> [out]]" >&2
    exit 2
    ;;
esac
