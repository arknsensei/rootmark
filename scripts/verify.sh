#!/usr/bin/env bash
# Independent end-to-end reproduction (docs/04_VERIFY.md).
#
#   scripts/verify.sh chain <source-repo> <workdir>
#       clone <source-repo> into <workdir>/repo, set it up, rebuild and verify
#       every dataset against its committed manifest, run freeze-check, every
#       eval phase and the report; prints "RUN=<id>" and "RUN_DIR=<path>".
#   scripts/verify.sh pair <source-repo> <workdir-A> <workdir-B>
#       two independent chains, then eval/compare_runs.py A B → H4-2.
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
  log "clone $src -> $work/repo"
  git clone --quiet "$src" "$work/repo"
  cd "$work/repo"
  log "commit $(git rev-parse HEAD)"
  log "setup"
  make setup >"$work/setup.log" 2>&1
  log "datasets (d2 d3 d1, manifests verified)"
  make d2 d3 d1
  log "freeze-check"
  make freeze-check
  local run
  run="$(uv run --frozen python -m eval.runinfo)"
  log "RUN=$run"
  make eval-h1s-tune RUN="$run"
  make eval-h1s RUN="$run"
  make eval-h1r RUN="$run"
  if [ -f data/d4/single.jsonl ] && [ -f data/splits/d4.json ]; then
    make eval-h2-tune RUN="$run"
    make eval-h2 RUN="$run"
  else
    log "H2 SKIPPED: data/d4 missing (T12 parked); the report will fail on the missing h2.json"
  fi
  make eval-h3 RUN="$run"
  make eval-h4 RUN="$run"
  make report RUN="$run"
  log "chain finished in $((SECONDS - t0)) s"
  echo "RUN=$run" >&3
  echo "RUN_DIR=$work/repo/reports/runs/$run" >&3
}

pair() {
  local src="$1" a="$2" b="$3"
  local out_a out_b dir_a dir_b
  out_a="$(chain "$src" "$a")"
  out_b="$(chain "$src" "$b")"
  dir_a="$(printf '%s\n' "$out_a" | sed -n 's/^RUN_DIR=//p')"
  dir_b="$(printf '%s\n' "$out_b" | sed -n 's/^RUN_DIR=//p')"
  log "H4-2: compare $dir_a $dir_b"
  (cd "$(cd "$a" && pwd)/repo" && uv run --frozen python -m eval.compare_runs "$dir_a" "$dir_b" --out "$dir_a/h4_2.json")
  printf '%s\n%s\n' "$out_a" "$out_b"
}

case "${1:-}" in
  chain) chain "${2:?source repo}" "${3:?workdir}" ;;
  pair) pair "${2:?source repo}" "${3:?workdir A}" "${4:?workdir B}" ;;
  "")
    src="$(git rev-parse --show-toplevel)"
    base="$(mktemp -d -t mdk-verify.XXXXXX)"
    log "workdirs under $base"
    pair "$src" "$base/A" "$base/B"
    ;;
  *)
    echo "usage: $0 [chain <src> <dir> | pair <src> <dirA> <dirB>]" >&2
    exit 2
    ;;
esac
