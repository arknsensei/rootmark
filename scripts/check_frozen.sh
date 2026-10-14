#!/usr/bin/env bash
# Frozen files unchanged since their freeze commit (research/FROZEN.md;
# P1-A VERIFY V6, P1-B VERIFY V7).
#
#   scripts/check_frozen.sh [<repo-root>]
#
# 1. Rows: for every row `| `pattern` | `<subject prefix>` | note |` between
#    the frozen-table markers, the freeze commit is the first (oldest) commit
#    whose subject starts with the prefix; every file matching the pattern
#    that existed in that commit must be byte-identical in the working tree
#    (no edit, no deletion). Rows whose commit does not exist yet are reported
#    as not yet frozen.
#    - Re-freeze: a later row with the same pattern supersedes an earlier one
#      (its files are then checked from the later commit). The later commit
#      must descend from the earlier one and the note must cite a freeze
#      exception `<round>/F<n>` that exists as a `## F<n> ` heading in
#      docs/rounds/<round>/progress/FREEZE_EXCEPTIONS.md.
#    - `[append-only]` in the note: the file's frozen-table rows are checked as
#      in 2 (from the row's commit) instead of byte equality.
# 2. Table append-only (P1-A verifier recommendation Ö1): for every
#    docs/rounds/*/progress/ROUND_START sha, the frozen table may only have
#    gained rows since that sha — no row removed or rewritten in any commit
#    since (`git log -p`) nor in the working tree, no table marker added or
#    removed (the file must hold exactly one start and one end marker), and
#    every row of the table
#    at that sha is still present, unchanged and in order. At least one
#    ROUND_START must exist.
# Exit 1 on any violation.
set -euo pipefail

root="$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)"
cd "$root"
frozen=research/FROZEN.md
table_of() { # table_of < file → the table rows between the markers
  sed -n '/<!-- frozen-table -->/,/<!-- \/frozen-table -->/p' | grep -E '^\| `' || true
}
table="$(table_of <"$frozen")"
[ -n "$table" ] || { echo "check_frozen: no frozen table in $frozen" >&2; exit 1; }
# exactly one start and one end marker, start first: a second end marker
# could otherwise push later rows out of the table (B0 review)
markers="$(grep -nE '^<!-- /?frozen-table -->$' "$frozen" | cut -d: -f2 | tr '\n' ' ')"
if [ "$markers" != "<!-- frozen-table --> <!-- /frozen-table --> " ]; then
  echo "check_frozen: $frozen must hold exactly one frozen-table start and end marker (found: $markers)" >&2
  exit 1
fi

fail=0
violation() { echo "check_frozen: VIOLATION: $*" >&2; fail=1; }

first_commit() { # first commit whose subject starts with $1 ("" if none)
  git log --reverse --format='%H %s' | awk -v p="$1" '!found && index(substr($0, 42), p) == 1 {print $1; found = 1}'
}

# append_only <sha> <label>: table rows only added since <sha>
append_only() {
  local sha="$1" label="$2" removed old_rows new_rows fail0="$fail"
  if ! git merge-base --is-ancestor "$sha" HEAD 2>/dev/null; then
    violation "$label: $sha is not an ancestor of HEAD"
    return
  fi
  # every commit since <sha>, then the uncommitted working tree
  removed="$( { git log -p --no-merges --format= "$sha"..HEAD -- "$frozen"; git diff HEAD -- "$frozen"; } |
    grep -E '^-(\| `|<!-- /?frozen-table -->)|^\+.*<!-- /?frozen-table -->' || true)"
  if [ -n "$removed" ]; then
    violation "$label: frozen-table rows removed or rewritten since ${sha:0:7}:"
    echo "$removed" | sed 's/^/  /' >&2
  fi
  old_rows="$(git show "$sha:$frozen" | table_of)"
  new_rows="$table"
  # ordered containment (also covers merges, which git log -p does not diff)
  if ! awk 'NR == FNR {want[++n] = $0; next} i < n && $0 == want[i + 1] {i++} END {exit !(i == n)}' \
    <(printf '%s\n' "$old_rows") <(printf '%s\n' "$new_rows"); then
    violation "$label: a frozen-table row of ${sha:0:7} is missing, changed or reordered"
  fi
  [ "$fail" -ne "$fail0" ] || echo "check_frozen: table append-only since ${sha:0:7} ($label)"
}

# --- 1. rows
rows=0
declare -a pats=() prefixes=() notes=()
while IFS= read -r line; do
  pats+=("$(echo "$line" | cut -d'|' -f2 | tr -d ' `')")
  prefixes+=("$(echo "$line" | cut -d'|' -f3 | sed -E 's/^ *`//; s/` *$//')")
  notes+=("$(echo "$line" | cut -d'|' -f4-)")
  rows=$((rows + 1))
done <<<"$table"
[ "$rows" -gt 0 ] || { echo "check_frozen: empty table" >&2; exit 1; }

for ((i = 0; i < rows; i++)); do
  pat="${pats[$i]}" prefix="${prefixes[$i]}" note="${notes[$i]}"
  commit="$(first_commit "$prefix")"
  if [ -z "$commit" ]; then
    echo "check_frozen: $pat — not yet frozen (no commit '${prefix}...')"
    continue
  fi
  # a later committed row with the same pattern supersedes this one
  superseded=""
  for ((j = i + 1; j < rows; j++)); do
    [ "${pats[$j]}" = "$pat" ] || continue
    later="$(first_commit "${prefixes[$j]}")"
    [ -n "$later" ] || continue
    if ! git merge-base --is-ancestor "$commit" "$later"; then
      violation "$pat: re-freeze ${prefixes[$j]} does not descend from $prefix"
    fi
    ref="$(echo "${notes[$j]}" | grep -Eo '[A-Z0-9]+-[A-Z0-9]+/F[0-9]+' | head -1 || true)"
    exc="docs/rounds/${ref%%/*}/progress/FREEZE_EXCEPTIONS.md"
    if [ -z "$ref" ] || ! grep -Eq "^## ${ref##*/} " "$exc" 2>/dev/null; then
      violation "$pat: re-freeze ${prefixes[$j]} cites no freeze exception (<round>/F<n> in $exc)"
    fi
    superseded="${prefixes[$j]}"
  done
  if [ -n "$superseded" ]; then
    echo "check_frozen: $pat — $prefix superseded by re-freeze $superseded"
    continue
  fi
  if [[ "$note" == *"[append-only]"* ]]; then
    if [ "$pat" != "$frozen" ]; then
      violation "$pat: [append-only] is defined for $frozen only"
      continue
    fi
    append_only "$commit" "$pat row, $prefix"
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
    violation "$pat matched no file in ${commit:0:7} ($prefix)"
    continue
  fi
  changed="$(git diff --name-only "$commit" -- "${files[@]}")"
  if [ -n "$changed" ]; then
    violation "changed since ${commit:0:7} ($prefix):"
    echo "$changed" | sed 's/^/  /' >&2
  else
    echo "check_frozen: $pat — ${#files[@]} files unchanged since ${commit:0:7} ($prefix)"
  fi
done

# --- 2. table append-only since every round start
starts=0
for f in docs/rounds/*/progress/ROUND_START; do
  [ -f "$f" ] || continue
  sha="$(tr -d ' \n' <"$f")"
  if ! git rev-parse --verify --quiet "$sha^{commit}" >/dev/null; then
    violation "$f: $sha is not a commit"
    continue
  fi
  starts=$((starts + 1))
  append_only "$(git rev-parse "$sha^{commit}")" "$f"
done
[ "$starts" -gt 0 ] || violation "no docs/rounds/*/progress/ROUND_START (table append-only check impossible)"

if [ "$fail" -ne 0 ]; then
  echo "check_frozen: FAILED" >&2
  exit 1
fi
echo "check_frozen: OK"
