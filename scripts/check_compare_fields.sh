#!/usr/bin/env bash
# K-E0.2 (P1-E/E0): research/eval/DETERMINISTIC_FIELDS.md against the run
# comparators (research/eval/check_compare_fields.py; Python lives only in
# research/). Exit 0 when every declared file and field matches the code.
#
#   scripts/check_compare_fields.sh [--runs RUN_DIR … ] [--out path.json]
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
args=()
for a in "$@"; do
  # run directories and the output path are taken from the caller's directory
  if [ -e "$a" ] || [ -e "$(dirname "$a")" ] && [ "${a#-}" = "$a" ]; then
    args+=("$(cd "$(dirname "$a")" && pwd)/$(basename "$a")")
  else
    args+=("$a")
  fi
done
cd "$here/research"
uv run --frozen python -m eval.check_compare_fields "${args[@]+"${args[@]}"}"
