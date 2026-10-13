#!/usr/bin/env bash
# Independent end-to-end reproduction (docs/04_VERIFY.md + docs/v3/04_VERIFY_DELTA.md).
#
#   scripts/verify.sh chain <source-repo> <workdir>
#       clone <source-repo> into <workdir>/repo, set it up, rebuild and verify
#       every dataset (D2, D3 v2+v3 hashes, D1, D1-easy(K4), D1-hard) against
#       its committed manifest, run freeze-check (v2) and freeze-check-v3, the
#       v2 phases (H1-S, H1-R under matcher v2; H2 is not run in the v3 round)
#       and every v3 phase (H1-S v3 tune/test, H1-R v3, H3, H4, H4-V2) and the
#       v3 report; prints "RUN=<id>" and "RUN_DIR=<path>".
#   scripts/verify.sh pair <source-repo> <workdir-A> <workdir-B>
#       two independent chains, then `compare` → H4-2.
#   scripts/verify.sh compare <repo> <run-dir-A> <run-dir-B> [<out.json>]
#       eval/compare_runs.py --set v3 run in <repo>; exits non-zero when H4-2 ≠ 0.
#   scripts/verify.sh            (what `make verify` runs)
#       pair mode with <source-repo> = this repository and two fresh temp dirs.
#
# Nothing here refers to the main working directory except as the clone
# source; every path is derived from the arguments.
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
  make setup >"$work/setup.log" 2>&1
  log "datasets (d2 d3 d1 d1k4 d1h, manifests verified)"
  make d2 d3 d1 d1k4 d1h
  log "freeze-check (v2) and freeze-check-v3"
  make freeze-check
  make freeze-check-v3
  local run
  run="$(uv run --frozen python -m eval.runinfo)"
  log "RUN=$run"
  log "v2 phases (matcher v2)"
  make eval-h1s-tune RUN="$run"
  make eval-h1s RUN="$run"
  make eval-h1r RUN="$run"
  log "v3 phases (matcher v3)"
  make eval-h1s-v3 RUN="$run" PHASE=tune
  make eval-h1s-v3 RUN="$run" PHASE=test
  make eval-h1r-v3 RUN="$run"
  make eval-h3 RUN="$run"
  make eval-h4 RUN="$run"
  make eval-h4-v2repro RUN="$run"
  make report-v3 RUN="$run"
  log "chain finished in $((SECONDS - t0)) s"
  echo "RUN=$run" >&3
  echo "RUN_DIR=$work/repo/reports/runs/$run" >&3
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
  compare "$(cd "$a" && pwd)/repo" "$dir_a" "$dir_b" "$dir_a/h4_2.json"
}

compare() {
  local repo="$1" dir_a="$2" dir_b="$3" out="${4:-}"
  log "H4-2: compare $dir_a $dir_b"
  local rc=0
  (cd "$repo" && uv run --frozen python -m eval.compare_runs "$dir_a" "$dir_b" --set v3 \
    ${out:+--out "$out"}) >&2 || rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "error: H4-2 comparison failed (compare_runs exit $rc)" >&2
    exit "$rc"
  fi
  log "H4-2 = 0"
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
