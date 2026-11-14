#!/usr/bin/env bash
# The documentation site (https://docs.rootmark.ai, mdBook). The pages whose
# source is elsewhere in the repository are copied in here, never kept twice:
# docs/GUIDE.md, docs/CLIENT_CONTRACT.md, docs/THREAT_MODEL.md, CHANGELOG.md,
# SECURITY.md, spec/SPEC.md, spec/CONFORMANCE.md (their links between each
# other are rewritten to the site's pages, any other repository link to the
# file at this release's tag on GitHub). The conformance report and the badge
# come from running `rootmark conformance` (a measured number, never typed).
# Then `mdbook build` and a check of every internal link of the built site:
# a broken one fails the build.
#
#   site/build.sh <rootmark binary> [<mdbook binary>]     (ROOTMARK_MODELS set)
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
rm_bin="${1:?usage: site/build.sh <rootmark> [<mdbook>]}"
mdbook="${2:-mdbook}"
ver="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$root/server/Cargo.toml" | head -1)"
[ -n "$ver" ] || { echo "site: no version in server/Cargo.toml" >&2; exit 1; }
gh="https://github.com/rootmark-ai/rootmark/blob/v$ver"
src="$here/src"

# copy <repository file> <site page>: the links between the copied documents
# point to their pages; a link to another repository file goes to GitHub
copy() {
  local from="$1" to="$2" dir
  dir="$(dirname "$from")"; [ "$dir" = . ] && dir=""
  mkdir -p "$(dirname "$src/$to")"
  sed -E \
    -e 's#\]\((\.\./)?(docs/)?GUIDE\.md#](/guide.md#g' \
    -e 's#\]\((\.\./)?(docs/)?CLIENT_CONTRACT\.md#](/contract.md#g' \
    -e 's#\]\((\.\./)?(docs/)?THREAT_MODEL\.md#](/threat-model.md#g' \
    -e 's#\]\((\.\./)?CHANGELOG\.md#](/changelog.md#g' \
    -e 's#\]\((\.\./)?SECURITY\.md#](/security.md#g' \
    -e 's#\]\((\.\./)?(spec/)?SPEC\.md#](/spec/index.md#g' \
    -e 's#\]\((\.\./)?(spec/)?CONFORMANCE\.md#](/spec/conformance.md#g' \
    -e "s#\\]\\((\\.\\./)?(LICENSE|NOTICE|CONTRIBUTING\\.md|RELEASING\\.md|spec/LICENSE|spec/fixtures/?)\\)#](${gh}/\\2)#g" \
    "$root/$from" >"$src/$to"
  # mdBook resolves "/x.md" from the book root: make it relative to the page
  local depth up=""
  depth="$(awk -F/ '{print NF-1}' <<<"$to")"
  for ((i = 0; i < depth; i++)); do up="../$up"; done
  sed -i.bak -E "s#\\]\\(/([a-z-]+(/[a-z-]+)?\\.md)#](${up}\\1#g" "$src/$to" && rm -f "$src/$to.bak"
}
copy docs/GUIDE.md guide.md
copy docs/CLIENT_CONTRACT.md contract.md
copy docs/THREAT_MODEL.md threat-model.md
copy CHANGELOG.md changelog.md
copy SECURITY.md security.md
copy spec/SPEC.md spec/index.md
copy spec/CONFORMANCE.md spec/conformance.md

# the measured conformance report and the badge's data (a shields.io endpoint)
"$rm_bin" conformance >"$src/spec/conformance.json"
jq -e '.level_claimed == 3 and .passed == .total and .total > 0' "$src/spec/conformance.json" >/dev/null \
  || { echo "site: rootmark conformance did not pass" >&2; exit 1; }
jq '{schemaVersion: 1, label: "conformance",
     message: "Level \(.level_claimed) · \(.passed)/\(.total)",
     color: (if .passed == .total then "2ea44f" else "e05d44" end)}' \
  "$src/spec/conformance.json" >"$src/spec/badge.json"

rm -rf "$here/book"
"$mdbook" build "$here" >/dev/null

# every internal link of the built site resolves (a file, or a directory's index.html)
book="$here/book"
broken=0; checked=0
while IFS= read -r -d '' page; do
  pdir="$(dirname "$page")"
  while IFS= read -r href; do
    case "$href" in
      http:* | https:* | mailto:* | \#* | javascript:* | data:* | "") continue ;;
    esac
    path="${href%%#*}"; path="${path%%\?*}"
    [ -n "$path" ] || continue
    case "$path" in
      /*) target="$book$path" ;;
      *) target="$pdir/$path" ;;
    esac
    checked=$((checked + 1))
    if [ -d "$target" ]; then target="$target/index.html"; fi
    if [ ! -e "$target" ]; then
      echo "site: broken link in ${page#"$book"/}: $href" >&2
      broken=$((broken + 1))
    fi
  done < <(grep -oE '(href|src)="[^"]*"' "$page" | sed -E 's/^(href|src)="(.*)"$/\2/')
done < <(find "$book" -name '*.html' -print0)
echo "site: $(find "$book" -name '*.html' | wc -l | tr -d ' ') pages, $checked internal links, $broken broken"
[ "$broken" -eq 0 ]
