#!/usr/bin/env bash
# P2-V/V3 (GOAL P2-V V3, V-5): no host value in a record the pair
# comparison reads — research/eval/portable.py (the work directory, $HOME,
# $TMPDIR, the user name in a path, the host name; the chain's values from
# its host.json and this process's). Exit 0 clean, 3 found, 1 missing file.
#
#   scripts/check_records_portable.sh <run-dir> [--host host.json …] [--fast] [--skip-missing] [--out path.json]
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
run="$(cd "${1:?run dir}" && pwd)"; shift
args=()
while [ $# -gt 0 ]; do
  case "$1" in
    --host | --out) args+=("$1" "$(cd "$(dirname "$2")" && pwd)/$(basename "$2")"); shift 2 ;;
    *) args+=("$1"); shift ;;
  esac
done
cd "$root/research" && exec uv run --frozen python -m eval.portable "$run" ${args[@]+"${args[@]}"}
