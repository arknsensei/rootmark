#!/usr/bin/env bash
# Frozen files unchanged since their freeze commit (research/FROZEN.md; VERIFY V6).
#
#   scripts/check_frozen.sh [<repo-root>]
#
# For every row `| `pattern` | `<subject prefix>` | … |` between the
# frozen-table markers: the freeze commit is the first (oldest) commit whose
# subject starts with the prefix; every file matching the pattern that existed
# in that commit must be byte-identical in the working tree (no edit, no
# deletion). Exit 1 on any change; rows whose commit does not exist yet are
# reported as not yet frozen.
set -euo pipefail

root="$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)"
cd "$root"
table="$(sed -n '/<!-- frozen-table -->/,/<!-- \/frozen-table -->/p' research/FROZEN.md |
  grep -E '^\| `' || true)"
[ -n "$table" ] || { echo "check_frozen: no frozen table in research/FROZEN.md" >&2; exit 1; }

fail=0
rows=0
while IFS= read -r line; do
  pat="$(echo "$line" | cut -d'|' -f2 | tr -d ' `')"
  prefix="$(echo "$line" | cut -d'|' -f3 | sed -E 's/^ *`//; s/` *$//')"
  rows=$((rows + 1))
  # the FIRST commit with the prefix is the freeze point (a later commit with
  # the same prefix must not move it); awk reads the whole log (no SIGPIPE)
  commit="$(git log --reverse --format='%H %s' | awk -v p="$prefix" '!found && index(substr($0, 42), p) == 1 {print $1; found = 1}')"
  if [ -z "$commit" ]; then
    echo "check_frozen: $pat — not yet frozen (no commit '${prefix}...')"
    continue
  fi
  base="$(dirname "${pat%%\**}x")"
  glob="${pat//\*\*/*}"
  files=()
  while IFS= read -r f; do
    # shellcheck disable=SC2053
    [[ "$f" == $glob ]] && files+=("$f")
  done < <(git ls-tree -r --name-only "$commit" -- "$base")
  if [ "${#files[@]}" -eq 0 ]; then
    echo "check_frozen: VIOLATION: $pat matched no file in ${commit:0:7} ($prefix)" >&2
    fail=1
    continue
  fi
  changed="$(git diff --name-only "$commit" -- "${files[@]}")"
  if [ -n "$changed" ]; then
    echo "check_frozen: VIOLATION: changed since ${commit:0:7} ($prefix):" >&2
    echo "$changed" | sed 's/^/  /' >&2
    fail=1
  else
    echo "check_frozen: $pat — ${#files[@]} files unchanged since ${commit:0:7} ($prefix)"
  fi
done <<<"$table"

[ "$rows" -gt 0 ] || { echo "check_frozen: empty table" >&2; exit 1; }
if [ "$fail" -ne 0 ]; then
  echo "check_frozen: FAILED" >&2
  exit 1
fi
echo "check_frozen: OK"
