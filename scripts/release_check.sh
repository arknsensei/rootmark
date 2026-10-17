#!/usr/bin/env bash
# Checks every item of docs/RELEASE.md (R1–R14) against the tree and the
# release package built by scripts/release.sh. Prints one line per item and
# writes <dist>/release_check.json; exit 0 only if every item passes.
#
#   scripts/release_check.sh [<dist-dir>]      (default dist/)
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist="${1:-$root/dist}"
cd "$root"
items=()
item() { # item <id> <label> <rc> [detail]
  local ok=false; [ "$3" = 0 ] && ok=true
  items+=("$(jq -n --arg id "$1" --arg l "$2" --argjson ok "$ok" --arg d "${4:-}" '{id: $id, item: $l, ok: $ok, detail: $d}')")
  printf 'release_check: %s %s %s%s\n' "$($ok && echo 'ok  ' || echo FAIL)" "$1" "$2" "${4:+ — $4}"
}
t() { "$@" >/dev/null 2>&1; echo $?; }

ver="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
spec="$(sed -n 's/^pub const SPEC_VERSION: &str = "\(.*\)";$/\1/p' server/src/tools.rs)"
vers="$(for f in core/Cargo.toml index/Cargo.toml server/Cargo.toml bindings/python/Cargo.toml; do sed -n 's/^version = "\(.*\)"$/\1/p' "$f" | head -1; done; jq -r .version clients/obsidian/package.json clients/obsidian/manifest.json)"
item R1 "one version $ver" "$( [ "$ver" = 0.1.0 ] && [ "$(echo "$vers" | sort -u)" = "$ver" ]; echo $?)" "$(echo $vers)"
item R2 "spec version $spec" "$( grep -q "\"spec_version\": \"$spec\"" core/tests/conformance.rs && head -1 spec/SPEC.md | grep -q "v$spec\$"; echo $?)"
mdk="$root/target/release/mdk"
item R3 "mdk --version / --spec-version" "$( [ "$("$mdk" --version 2>/dev/null)" = "mdk $ver" ] && [ "$("$mdk" --spec-version 2>/dev/null)" = "$spec" ]; echo $?)"
# the package's inputs: a dist/ built at a commit whose inputs equal HEAD's,
# from a clean tree, is fresh (later commits of records only are fine)
INPUTS=(Cargo.toml Cargo.lock core index server bindings clients/obsidian spec models/MANIFEST.json
  scripts/release.sh scripts/linux_cross_build.sh scripts/fetch_model.sh packaging LICENSE README.md)
if [ -f "$dist/release.json" ]; then
  pc="$(jq -r '.commit // ""' "$dist/release.json")"
  why=""
  [ "$(jq -r '.version // ""' "$dist/release.json")" = "$ver" ] || why="version $(jq -r '.version // "?"' "$dist/release.json") != $ver"
  [ "$(jq -r '.worktree_dirty' "$dist/release.json")" = false ] || why="${why:+$why; }built from a dirty work tree"
  if [ -z "$pc" ] || ! git cat-file -e "$pc^{commit}" 2>/dev/null; then why="${why:+$why; }unknown commit ${pc:-none}"
  elif ! git diff --quiet "$pc" HEAD -- "${INPUTS[@]}"; then why="${why:+$why; }inputs changed since ${pc:0:7}: $(git diff --name-only "$pc" HEAD -- "${INPUTS[@]}" | head -3 | tr '\n' ' ')"; fi
  item R4a "release package fresh (inputs of ${pc:0:7} = HEAD, clean tree, version)" "$( [ -z "$why" ]; echo $?)" "$why"
  item R4 "release binaries pass mdk conformance 34/34" "$(jq -e '.ok == 4' "$dist/release.json" >/dev/null; echo $?)" \
    "$(jq -r '[.targets[] | "\(.target): \(if .ok then "ok" elif .built then "built, check failed" else "not built" end)"] | join("; ")' "$dist/release.json")"
else
  item R4a "release package fresh" 1 "dist/release.json missing (scripts/release.sh)"
  item R4 "release binaries pass mdk conformance 34/34" 1 "dist/release.json missing (scripts/release.sh)"
fi
tmp="$(mktemp -d -t mdk-relcheck.XXXXXX)"; trap 'rm -rf "$tmp"' EXIT
# a fresh make report (a stale reports/conformance.json never counts) and
# both exit codes
rm -f reports/conformance.json
mrc=0; make -s conformance >"$tmp/make.log" 2>&1 || mrc=$?
crc=0; MDK_MODELS="$root/models" "$mdk" conformance >"$tmp/mdk.json" 2>/dev/null || crc=$?
erc=1; [ "$mrc" = 0 ] && [ "$crc" = 0 ] && [ -f reports/conformance.json ] && erc="$(t bash scripts/conformance_equiv.sh reports/conformance.json "$tmp/mdk.json")"
item R5 "mdk conformance = make conformance" "$erc" "make exit $mrc, mdk conformance exit $crc"
# every archive, the wheel and the zip are listed, and every listed file checks
unlisted="$(cd "$dist" 2>/dev/null && for f in bin/*.tar.gz wheel/*.whl obsidian/*.zip homebrew/mdk.rb; do [ -f "$f" ] && ! grep -q "  $f\$" SHA256SUMS && echo "$f"; done | tr '\n' ' ')"
item R6 "archives and SHA256SUMS" "$( [ -s "$dist/SHA256SUMS" ] && [ -z "$unlisted" ] && (cd "$dist" && shasum -a 256 -c SHA256SUMS >/dev/null 2>&1) && grep -q 'tar.gz' "$dist/SHA256SUMS"; echo $?)" "${unlisted:+unlisted: $unlisted}"
whl="$(ls "$dist"/wheel/*-"$ver"-*.whl 2>/dev/null | head -1)"
item R7 "Python wheel (module mdkernel_core)" "$( [ -n "$whl" ] && unzip -Z1 "$whl" | grep -qE "^mdkernel_core/"; echo $?)" "${whl##*/}"
zipl="$(unzip -Z1 "$dist/obsidian/mdkernel-$ver.zip" 2>/dev/null | grep -v '/$' | sed 's#^[^/]*/##' | sort | tr '\n' ' ')"
size="$(wc -c <"$dist/obsidian/mdkernel-$ver/main.js" 2>/dev/null || echo 999999999)"
item R8 "Obsidian package" "$( [ "$zipl" = "main.js manifest.json styles.css " ] && [ "$size" -lt 512000 ] && jq -e '(.dependencies // {}) == {}' clients/obsidian/package.json >/dev/null; echo $?)" "files: $zipl; main.js $size B"
sd="$dist/spec-$spec"
# the published copies are the tree's, byte for byte, and pass
item R9 "spec publication dist/spec-$spec" "$( for f in SPEC.md CONFORMANCE.md LICENSE fixtures/MANIFEST.json conformance.json; do [ -f "$sd/$f" ] || exit 1; done
  cmp -s "$sd/SPEC.md" spec/SPEC.md && cmp -s "$sd/CONFORMANCE.md" spec/CONFORMANCE.md && cmp -s "$sd/LICENSE" LICENSE \
  && diff -rq -x .DS_Store -x .mdkernel "$sd/fixtures" spec/fixtures >/dev/null \
  && jq -e '.level_claimed == 3 and .passed == 34 and .total == 34 and .fixtures_manifest == "ok"' "$sd/conformance.json" >/dev/null; echo $?)"
item R10 "LICENSE Apache-2.0" "$( grep -q 'Apache License' LICENSE && grep -q 'Version 2.0, January 2004' LICENSE && for a in "$dist"/bin/*.tar.gz; do tar -tzf "$a" | grep -q '/LICENSE$' || exit 1; done; echo $?)"
item R11 "CHANGELOG, THREAT_MODEL, RELEASE" "$( grep -q "^## $ver" CHANGELOG.md && [ -f docs/THREAT_MODEL.md ] && [ -f docs/RELEASE.md ]; echo $?)"
item R12 "cargo-binstall metadata, Homebrew draft" "$( grep -q '^\[package.metadata.binstall\]' server/Cargo.toml && grep -q "version \"$ver\"" "$dist/homebrew/mdk.rb" && ! grep -q '@SHA256' "$dist/homebrew/mdk.rb"; echo $?)"
item R13 "CI cross build, four targets" "$( for tg in x86_64-unknown-linux-gnu aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc; do grep -q "target: $tg" .github/workflows/release.yml || exit 1; done; echo $?)"
item R14 "no v$ver tag (set by a human after verification)" "$( [ -z "$(git tag -l "v$ver")" ]; echo $?)"

printf '%s\n' "${items[@]}" | jq -s '{items: ., passed: ([.[] | select(.ok)] | length), total: length} | .pass = (.passed == .total)' >"$dist/release_check.json"
jq -r '"release_check: \(.passed)/\(.total)"' "$dist/release_check.json"
jq -e .pass "$dist/release_check.json" >/dev/null
