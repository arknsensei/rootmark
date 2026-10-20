#!/usr/bin/env bash
# Checks every item of docs/RELEASE.md (R1–R20, R4a) against the tree and
# the release package built by scripts/release.sh. No pipeline ends in an
# early-exiting reader (`grep -q`, `head`) under pipefail: a listing is
# captured first and then searched (P1-E verification V7: R7's `unzip |
# grep -q` failed at random on SIGPIPE; P1-E.1 E1-4). Prints one line per item and
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
spec_head="$(sed -n 1p spec/SPEC.md)"
item R2 "spec version $spec" "$( grep -q "\"spec_version\": \"$spec\"" core/tests/conformance.rs && grep -q "v$spec\$" <<<"$spec_head"; echo $?)"
rootmark="$root/target/release/rootmark"
item R3 "rootmark --version / --spec-version" "$( [ "$("$rootmark" --version 2>/dev/null)" = "rootmark $ver" ] && [ "$("$rootmark" --spec-version 2>/dev/null)" = "$spec" ]; echo $?)"
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
  # K-E1.1: the required target (aarch64-apple-darwin) passes with its hash;
  # the other targets are information (recorded, or recorded as skipped)
  item R4 "release binary aarch64-apple-darwin: --version, --spec-version, rootmark conformance 39/39, sha256" \
    "$(jq -e --arg ver "$ver" --arg spec "$spec" '.required_ok == true and .required_targets == ["aarch64-apple-darwin"]
      and all(.targets[]; .ok or .skipped == true or .required == false)
      and (.targets[] | select(.target == "aarch64-apple-darwin")
        | .ok and .version == ("rootmark " + $ver) and .spec_version == $spec and (.sha256 | test("^[0-9a-f]{64}$"))
          and .conformance.passed == 39 and .conformance.total == 39)' "$dist/release.json" >/dev/null; echo $?)" \
    "$(jq -r '[.targets[] | "\(.target)\(if .required then "" else " (CI info)" end): \(if .ok then "ok" elif .built then "built, check failed" else "skipped" end)"] | join("; ")' "$dist/release.json")"
else
  item R4a "release package fresh" 1 "dist/release.json missing (scripts/release.sh)"
  item R4 "release binaries pass rootmark conformance 39/39" 1 "dist/release.json missing (scripts/release.sh)"
fi
tmp="$(mktemp -d -t rootmark-relcheck.XXXXXX)"; trap 'rm -rf "$tmp"' EXIT
# a fresh make report (a stale reports/conformance.json never counts) and
# both exit codes
rm -f reports/conformance.json
mrc=0; make -s conformance >"$tmp/make.log" 2>&1 || mrc=$?
crc=0; ROOTMARK_MODELS="$root/models" "$rootmark" conformance >"$tmp/rootmark.json" 2>/dev/null || crc=$?
erc=1; [ "$mrc" = 0 ] && [ "$crc" = 0 ] && [ -f reports/conformance.json ] && erc="$(t bash scripts/conformance_equiv.sh reports/conformance.json "$tmp/rootmark.json")"
item R5 "rootmark conformance = make conformance" "$erc" "make exit $mrc, rootmark conformance exit $crc"
# every archive, the wheel and the zip are listed, and every listed file checks
unlisted="$(cd "$dist" 2>/dev/null && for f in bin/*.tar.gz wheel/*.whl obsidian/*.zip homebrew/rootmark.rb; do [ -f "$f" ] && ! grep -q "  $f\$" SHA256SUMS && echo "$f"; done | tr '\n' ' ')"
item R6 "archives and SHA256SUMS" "$( [ -s "$dist/SHA256SUMS" ] && [ -z "$unlisted" ] && (cd "$dist" && shasum -a 256 -c SHA256SUMS >/dev/null 2>&1) && grep -q 'tar.gz' "$dist/SHA256SUMS"; echo $?)" "${unlisted:+unlisted: $unlisted}"
whl="$(ls "$dist"/wheel/*-"$ver"-*.whl 2>/dev/null | head -1)"
[ -z "$whl" ] || unzip -Z1 "$whl" >"$tmp/wheel.list" 2>/dev/null || : >"$tmp/wheel.list"
item R7 "Python wheel (module rootmark_core)" "$( [ -n "$whl" ] && grep -qE "^rootmark_core/" "$tmp/wheel.list"; echo $?)" "${whl##*/}"
zipl="$(unzip -Z1 "$dist/obsidian/rootmark-$ver.zip" 2>/dev/null | grep -v '/$' | sed 's#^[^/]*/##' | sort | tr '\n' ' ')"
size="$(wc -c <"$dist/obsidian/rootmark-$ver/main.js" 2>/dev/null || echo 999999999)"
item R8 "Obsidian package" "$( [ "$zipl" = "main.js manifest.json styles.css " ] && [ "$size" -lt 512000 ] && jq -e '(.dependencies // {}) == {}' clients/obsidian/package.json >/dev/null; echo $?)" "files: $zipl; main.js $size B"
sd="$dist/spec-$spec"
# the published copies are the tree's, byte for byte, and pass
item R9 "spec publication dist/spec-$spec" "$( for f in SPEC.md CONFORMANCE.md LICENSE fixtures/MANIFEST.json conformance.json; do [ -f "$sd/$f" ] || exit 1; done
  cmp -s "$sd/SPEC.md" spec/SPEC.md && cmp -s "$sd/CONFORMANCE.md" spec/CONFORMANCE.md && cmp -s "$sd/LICENSE" LICENSE \
  && diff -rq -x .DS_Store -x .rootmark "$sd/fixtures" spec/fixtures >/dev/null \
  && jq -e '.level_claimed == 3 and .passed == 39 and .total == 39 and .fixtures_manifest == "ok"' "$sd/conformance.json" >/dev/null; echo $?)"
item R10 "LICENSE Apache-2.0" "$( grep -q 'Apache License' LICENSE && grep -q 'Version 2.0, January 2004' LICENSE && for a in "$dist"/bin/*.tar.gz; do tar -tzf "$a" >"$tmp/tar.list" && grep -q '/LICENSE$' "$tmp/tar.list" || exit 1; done; echo $?)"
item R11 "CHANGELOG, THREAT_MODEL, RELEASE" "$( grep -q "^## $ver" CHANGELOG.md && [ -f docs/THREAT_MODEL.md ] && [ -f docs/RELEASE.md ]; echo $?)"
item R12 "cargo-binstall metadata, Homebrew draft" "$( grep -q '^\[package.metadata.binstall\]' server/Cargo.toml && grep -q "version \"$ver\"" "$dist/homebrew/rootmark.rb" && ! grep -q '@SHA256' "$dist/homebrew/rootmark.rb"; echo $?)"
# R13 (K-E1.1): started by hand only; aarch64-apple-darwin required, Linux
# and Windows CI information (continue-on-error), no Intel macOS; every job
# checks through release_target.sh (rootmark conformance) and dist/release.json
# is assembled
wf=.github/workflows/release.yml
on_block="$(sed -n '/^on:/,/^jobs:/p' "$wf" | grep -vE '^(on:|jobs:)$' | grep -vE '^\s*(#.*)?$' | tr -d ' ')"
item R13 "CI release.yml: workflow_dispatch only, aarch64 required, Linux/Windows continue-on-error, rootmark conformance per job" "$(
  [ "$on_block" = "workflow_dispatch:" ] \
  && grep -qE '^\s*- \{ target: aarch64-apple-darwin, .*required: true \}' "$wf" \
  && grep -qE '^\s*- \{ target: x86_64-unknown-linux-gnu, .*required: false \}' "$wf" \
  && grep -qE '^\s*- \{ target: x86_64-pc-windows-msvc, .*required: false \}' "$wf" \
  && ! grep -q 'x86_64-apple-darwin' "$wf" \
  && grep -qF 'continue-on-error: ${{ !matrix.required }}' "$wf" \
  && grep -qF 'bash scripts/release_target.sh check' "$wf" \
  && grep -qF 'bash scripts/release_target.sh assemble' "$wf"; echo $?)" "on: $on_block"
# R16 (P2-0/Z0; P1-E.1 verification, recommendation 1): RELEASE.md human
# step 1 names the verification the release waits for; that report exists
# and its last verdict line is "Güvenilir"
step1="$(sed -n '/^## İnsan adımları/,$p' docs/RELEASE.md | grep -E '^1\. ' | head -1 || true)"
vrep="$(grep -oE 'reports/rounds/[^/` ]+/VERIFICATION\.md' <<<"$step1" | head -1 || true)"
verdict=""
if [ -n "$vrep" ] && [ -f "$vrep" ]; then
  verdict="$(grep -oE '^\*\*Hüküm: [^*]+\*\*' "$vrep" | tail -1 | sed -E 's/^\*\*Hüküm: //; s/\.?\*\*$//' || true)"
fi
# R14: the tag is a human step (RELEASE.md step 4): absent, or set after
# that verification said "Güvenilir" and not by the active round (the tag's
# commit is the newest ROUND_START or an ancestor of it)
newest_rs=""
for f in docs/rounds/*/progress/ROUND_START; do
  c="$(tr -d ' \n' <"$f")"
  if [ -z "$newest_rs" ] || git merge-base --is-ancestor "$newest_rs" "$c" 2>/dev/null; then newest_rs="$c"; fi
done
tagc="$(git rev-list -n1 "v$ver" 2>/dev/null || true)"
if [ -z "$tagc" ]; then
  item R14 "v$ver tag absent or set by a human after a Güvenilir verification" 0 "no tag"
else
  item R14 "v$ver tag absent or set by a human after a Güvenilir verification" \
    "$( [ "$verdict" = "Güvenilir" ] && git merge-base --is-ancestor "$tagc" "$newest_rs" 2>/dev/null; echo $?)" \
    "v$ver at ${tagc:0:7}, newest round start ${newest_rs:0:7}, ${vrep:-no report}: ${verdict:-no verdict}"
fi
# R15 (K-E1.3): the real-Obsidian e2e of this package passed (a skip is not a pass)
re="$dist/plugin_e2e_real/plugin_e2e_real.json"
item R15 "real-Obsidian e2e 11/11 and E1-9 (K-E1.3)" "$( [ -f "$re" ] && jq -e '.pass == true and .passed == 11 and .e1_9_connection.pass == true' "$re" >/dev/null; echo $?)" \
  "$( [ -f "$re" ] && jq -r 'if .skipped then "skipped: \(.reason)" else "\(.passed)/11, E1-9 \(.e1_9_connection.pass)" end' "$re" || echo "no record (scripts/release.sh runs scripts/plugin_e2e_real.sh)")"
item R16 "RELEASE.md human step 1 waits for a verification that says Güvenilir" "$( [ "$verdict" = "Güvenilir" ]; echo $?)" \
  "${vrep:-no report named in step 1}: ${verdict:-no verdict}"

# R17–R20 (P2-A/A3, freeze exception P2-A/F3; docs/RELEASE.md "Kurulum")
# R17: the installers exist, and install.sh installs this package's archive
# from a local SHA256SUMS (no model download) and refuses a changed archive
# (nothing installed)
host_t="$(case "$(uname -s)-$(uname -m)" in (Linux-x86_64) echo x86_64-unknown-linux-gnu ;; (Darwin-arm64) echo aarch64-apple-darwin ;; esac)"
r17=1; r17d="no archive of this host's target in $dist/bin"
ha="$(ls "$dist"/bin/rootmark-*-"$host_t".tar.gz 2>/dev/null | head -1 || true)"
if [ -f packaging/install.sh ] && [ -f packaging/install.ps1 ] && bash -n packaging/install.sh && [ -n "$ha" ]; then
  mkdir -p "$tmp/inst/src" && cp "$ha" "$tmp/inst/src/"
  (cd "$tmp/inst/src" && shasum -a 256 "${ha##*/}" >SHA256SUMS)
  if bash packaging/install.sh --from "$tmp/inst/src" --prefix "$tmp/inst/ok" --no-model >"$tmp/inst/ok.log" 2>&1 \
    && [ "$("$tmp/inst/ok/bin/rootmark" --version)" = "rootmark $ver" ]; then
    printf 'x' >>"$tmp/inst/src/${ha##*/}"
    if ! bash packaging/install.sh --from "$tmp/inst/src" --prefix "$tmp/inst/bad" --no-model >"$tmp/inst/bad.log" 2>&1 \
      && [ ! -e "$tmp/inst/bad/bin/rootmark" ]; then r17=0; r17d="installed ${ha##*/}; a changed archive refused"
    else r17d="a changed archive was not refused"; fi
  else r17d="install.sh failed: $(tail -1 "$tmp/inst/ok.log")"; fi
fi
item R17 "installers install.sh / install.ps1 (SHA256SUMS checked, a changed archive refused)" "$r17" "$r17d"
# R18: release.yml installs each archive with its installer and checks the
# installed binary; the last job writes SHA256SUMS and attests provenance
item R18 "CI release.yml: installer per job, SHA256SUMS, build provenance attestation" "$(
  grep -qF 'bash packaging/install.sh --from' "$wf" && grep -qF './packaging/install.ps1 -From' "$wf" \
  && grep -qF 'sha256sum rootmark-* install.sh install.ps1 >SHA256SUMS' "$wf" \
  && grep -qE 'uses: actions/attest-build-provenance@v[0-9]+\.[0-9]+\.[0-9]+' "$wf" \
  && grep -qE '^\s*id-token: write' "$wf" && grep -qE '^\s*attestations: write' "$wf"; echo $?)"
# R19: code signing and notarization are human steps, written down
item R19 "RELEASE.md: signing and notarization are written human steps" "$(
  sed -n '/^## İnsan adımları/,$p' docs/RELEASE.md >"$tmp/human.md"
  grep -q 'notarization' "$tmp/human.md" && grep -q 'Authenticode' "$tmp/human.md" && grep -q 'gh attestation verify' docs/RELEASE.md; echo $?)"
# R20: every action of every workflow pinned to a full version (age:
# scripts/check_dep_age.sh), the toolchain action excepted
unpinned="$(grep -hoE 'uses: [^ ]+@[^ ]+' .github/workflows/*.yml | grep -vE '@v[0-9]+\.[0-9]+\.[0-9]+$' | grep -v 'dtolnay/rust-toolchain@' | sort -u | tr '\n' ' ' || true)"
item R20 "workflow actions pinned to full versions" "$( [ -z "$unpinned" ]; echo $?)" "${unpinned:+unpinned: $unpinned}"

printf '%s\n' "${items[@]}" | jq -s '{items: ., passed: ([.[] | select(.ok)] | length), total: length} | .pass = (.passed == .total)' >"$dist/release_check.json"
jq -r '"release_check: \(.passed)/\(.total)"' "$dist/release_check.json"
jq -e .pass "$dist/release_check.json" >/dev/null
