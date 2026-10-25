#!/usr/bin/env bash
# The dependency age rule (GOAL GH-0 G2; the pinning policy of every
# package CLAUDE.md): a direct dependency's declared version must have been
# published at least 14 days before the commit that introduced the
# declaration (`git blame` of its line; an uncommitted line is dated now).
#   - Rust: [dependencies], [dev-dependencies], [build-dependencies] of
#     core/, index/, server/, bindings/python/ Cargo.toml with a version
#     (`=x.y.z` or the lower bound `x.y.z`; path dependencies skipped) —
#     crates.io `created_at`;
#   - npm: clients/*/package.json (obsidian, vscode) dependencies, devDependencies —
#     registry `time`;
#   - Python: research/pyproject.toml `name==version` lines (dependencies and
#     dependency groups) — PyPI's first upload time of that version;
#   - GitHub Actions (P2-A/A3): every `uses: <owner>/<repo>@<ref>` of
#     .github/workflows/*.yml — the GitHub release of that tag, else the
#     commit date of the tag (`git/refs/tags`); a ref that is neither (a
#     branch) needs an exception. `GITHUB_TOKEN`/`GH_TOKEN` is used when set.
# Exceptions (e.g. a security fix adopted early) are listed one per line in
# scripts/dep_age_exceptions.txt as `<ecosystem> <name> <version> <decision>`
# (`#` comments); an exception without a decision is an error.
# Prints one line per dependency; with <out.json> also a record. Exit 0 when
# every dependency passes, 1 on a violation, 2 when a registry answer is
# missing or unreadable (never a silent pass).
#
#   scripts/check_dep_age.sh [<out.json>]
#   ROOTMARK_DEP_AGE_DAYS: the minimum age (default 14)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
out="${1:-}"
days="${ROOTMARK_DEP_AGE_DAYS:-14}"
exc_file="scripts/dep_age_exceptions.txt"
ua="rootmark-check-dep-age (https://github.com/arknsensei/rootmark)"
now="$(date -u +%s)"

exceptions=""
if [ -f "$exc_file" ]; then
  while IFS= read -r l; do
    l="${l%%#*}"
    [ -n "${l// /}" ] || continue
    # shellcheck disable=SC2086
    set -- $l
    [ "$#" -ge 4 ] || { echo "check_dep_age: $exc_file: exception without a decision: $l" >&2; exit 2; }
    exceptions="$exceptions|$1 $2 $3|"
  done <"$exc_file"
fi

entries=() # "<eco>\t<name>\t<version>\t<file>\t<line>"
for f in core/Cargo.toml index/Cargo.toml server/Cargo.toml bindings/python/Cargo.toml; do
  n=0; sect=""
  while IFS= read -r line; do
    n=$((n + 1))
    case "$line" in
      "["*)
        sect="$line"
        # a dependency table this check does not read is an error, never a skip
        case "$sect" in
          "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]") ;;
          *dependencies*) echo "check_dep_age: $f:$n: unsupported dependency table $sect" >&2; exit 2 ;;
        esac
        continue ;;
    esac
    case "$sect" in
      "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]") ;;
      *) continue ;;
    esac
    [[ "$line" =~ ^([A-Za-z0-9_-]+)[[:space:]]*=[[:space:]]*(.*)$ ]] || continue
    name="${BASH_REMATCH[1]}"; rest="${BASH_REMATCH[2]}"
    [[ "$rest" == *"path ="* ]] && continue
    if [[ "$rest" =~ ^\"=?([0-9][^\"]*)\" ]] || [[ "$rest" =~ version[[:space:]]*=[[:space:]]*\"=?([0-9][^\"]*)\" ]]; then
      entries+=("cargo	$name	${BASH_REMATCH[1]}	$f	$n")
    else
      echo "check_dep_age: $f:$n: no version in the declaration of $name" >&2; exit 2
    fi
  done <"$f"
done
# every client package (P2-B/B4: clients/obsidian and clients/vscode)
for pj in clients/*/package.json; do
  while IFS=$'\t' read -r name ver; do
    [[ "$ver" =~ ^[0-9] ]] || { echo "check_dep_age: $pj: $name $ver is not an exact version" >&2; exit 2; }
    n="$(grep -n "\"$name\": \"$ver\"" "$pj" | head -1 | cut -d: -f1)"
    [ -n "$n" ] || { echo "check_dep_age: $pj: the line of $name $ver was not found" >&2; exit 2; }
    entries+=("npm	$name	$ver	$pj	$n")
  done < <(jq -r '((.dependencies // {}) + (.devDependencies // {})) | to_entries[] | "\(.key)\t\(.value)"' "$pj")
done
pp=research/pyproject.toml
n=0
inlist=false
while IFS= read -r line; do
  n=$((n + 1))
  # requirement strings live in `dependencies = [ … ]` and the dependency groups
  [[ "$line" =~ ^(dependencies|dev)[[:space:]]*=[[:space:]]*\[ ]] && inlist=true
  if $inlist; then
    rest="$line"
    while [[ "$rest" =~ \"([^\"]*)\"(.*)$ ]]; do
      req="${BASH_REMATCH[1]}"; rest="${BASH_REMATCH[2]}"
      [[ "$req" =~ ^[A-Za-z0-9_.-]+==[0-9] ]] || { echo "check_dep_age: $pp:$n: \"$req\" is not pinned with ==" >&2; exit 2; }
    done
  fi
  [[ "$line" == *"]"* ]] && inlist=false
  while [[ "$line" =~ \"([A-Za-z0-9_.-]+)==([0-9][^\"]*)\"(.*)$ ]]; do
    entries+=("pypi	${BASH_REMATCH[1]}	${BASH_REMATCH[2]}	$pp	$n")
    line="${BASH_REMATCH[3]}"
  done
done <"$pp"

for wf in .github/workflows/*.yml; do
  n=0
  while IFS= read -r line; do
    n=$((n + 1))
    [[ "$line" =~ uses:[[:space:]]*([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)@([^[:space:]#]+) ]] || continue
    entries+=("actions	${BASH_REMATCH[1]}	${BASH_REMATCH[2]}	$wf	$n")
  done <"$wf"
done

gh_api() { # <path> → JSON (empty on failure)
  local tok="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
  if [ -n "$tok" ]; then curl -sf -A "$ua" -H "Authorization: Bearer $tok" "https://api.github.com/$1"
  else curl -sf -A "$ua" "https://api.github.com/$1"; fi
}

adopted_at() { # <file> <line> → unix time of the commit that wrote the line (now if uncommitted)
  local b t sha
  b="$(git blame --porcelain -L "$2,$2" -- "$1")" || { echo "check_dep_age: git blame $1:$2 failed" >&2; return 1; }
  sha="$(printf '%s\n' "$b" | head -1 | cut -d' ' -f1)"
  t="$(printf '%s\n' "$b" | awk '/^committer-time /{print $2; exit}')"
  if [ "$sha" = "0000000000000000000000000000000000000000" ]; then echo "$now"
  elif [ -n "$t" ]; then echo "$t"
  else echo "check_dep_age: no commit time for $1:$2" >&2; return 1; fi
}
iso_to_unix() { jq -rn --arg d "$1" '$d | sub("\\.[0-9]+"; "") | sub("\\+00:00$"; "Z") | fromdateiso8601'; }
published_at() { # <eco> <name> <version> → ISO time or empty
  case "$1" in
    cargo) curl -sf -A "$ua" "https://crates.io/api/v1/crates/$2/$3" | jq -r '.version.created_at // empty' ;;
    npm) curl -sf "https://registry.npmjs.org/$(printf '%s' "$2" | sed 's#/#%2F#')" | jq -r --arg v "$3" '.time[$v] // empty' ;;
    pypi) curl -sf "https://pypi.org/pypi/$2/$3/json" | jq -r '[.urls[].upload_time_iso_8601] | min // empty' ;;
    actions)
      local t sha
      # the same action and version is declared in many places: asked once
      # (a plain string: macOS bash 3.2 has no associative arrays)
      t="$(printf '%s' "$act_cache" | tr '|' '\n' | awk -v k="$2@$3" 'index($0, k "=") == 1 {print substr($0, length(k) + 2); exit}')"
      if [ -n "$t" ]; then printf '%s' "$t"; return; fi
      t="$(gh_api "repos/$2/releases/tags/$3" | jq -r '.published_at // empty' || true)"
      if [ -z "$t" ]; then
        sha="$(gh_api "repos/$2/git/refs/tags/$3" | jq -r 'if type == "object" then .object.url // empty else empty end' || true)"
        [ -n "$sha" ] && t="$(curl -sf -A "$ua" ${GITHUB_TOKEN:+-H "Authorization: Bearer $GITHUB_TOKEN"} "$sha" | jq -r '(.committer.date // .tagger.date // .object.sha) // empty' | grep -E '^[0-9]{4}-' || true)"
      fi
      printf '%s' "$t" ;;
  esac
}

act_cache="|"
fail=0; missing=0; rows=()
for e in "${entries[@]}"; do
  IFS=$'\t' read -r eco name ver file line <<<"$e"
  pub="$(published_at "$eco" "$name" "$ver" || true)"
  if [ "$eco" = actions ] && [ -n "$pub" ]; then act_cache="$act_cache$name@$ver=$pub|"; fi
  if [ -z "$pub" ] && [[ "$exceptions" == *"|$eco $name $ver|"* ]]; then
    # a ref with no publication time (an action's branch) listed as an exception
    echo "check_dep_age: ok   $eco $name $ver — no publication time (not a release or tag), exception ($exc_file)"
    rows+=("$(jq -n --arg e "$eco" --arg n "$name" --arg v "$ver" --arg f "$file:$line" \
      '{ecosystem: $e, name: $n, version: $v, declared: $f, published: null, ok: true, exception: "no publication time"}')")
    continue
  fi
  if [ -z "$pub" ]; then
    echo "check_dep_age: MISSING $eco $name $ver ($file:$line): no publication time from the registry" >&2
    missing=1; continue
  fi
  p="$(iso_to_unix "$pub")"; a="$(adopted_at "$file" "$line")" || exit 2
  age=$(( (a - p) / 86400 ))
  ok=true; why=""
  if [ "$age" -lt "$days" ]; then
    if [[ "$exceptions" == *"|$eco $name $ver|"* ]]; then why="exception ($exc_file)"; else ok=false; fail=1; fi
  fi
  printf 'check_dep_age: %s %s %s %s — published %s, adopted %s days later%s\n' \
    "$($ok && echo 'ok  ' || echo FAIL)" "$eco" "$name" "$ver" "${pub%%T*}" "$age" "${why:+, $why}"
  rows+=("$(jq -n --arg e "$eco" --arg n "$name" --arg v "$ver" --arg f "$file:$line" --arg p "$pub" \
    --argjson age "$age" --argjson ok "$ok" --arg why "$why" \
    '{ecosystem: $e, name: $n, version: $v, declared: $f, published: $p, age_days_at_adoption: $age, ok: $ok}
     + (if $why == "" then {} else {exception: $why} end)')")
done
if [ -n "$out" ]; then
  printf '%s\n' "${rows[@]}" | jq -s --argjson d "$days" --argjson m "$missing" --argjson n "${#entries[@]}" \
    '{min_age_days: $d, declared: $n, dependencies: ., registry_missing: ($m == 1),
      pass: ($m == 0 and length == $n and $n > 0 and all(.ok))}' >"$out"
fi
[ "${#entries[@]}" -gt 0 ] || { echo "check_dep_age: no dependency declaration found" >&2; exit 2; }
echo "check_dep_age: ${#entries[@]} dependencies, minimum age $days days"
[ "$missing" -eq 0 ] || exit 2
exit "$fail"
