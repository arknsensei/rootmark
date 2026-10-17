#!/usr/bin/env bash
# The 0.1.0 release package (GOAL P1-E "0.1.0 yayım paketi"; TASKS E5),
# built locally into dist/ (git-ignored). Tags nothing and publishes
# nothing: docs/RELEASE.md lists the human steps (the v0.1.0 tag is set
# only after the independent verification says "Güvenilir").
#
#   scripts/release.sh [<dist-dir>]        (default dist/)
#
# Produces:
#   bin/mdk-<ver>-<target>.tar.gz   each target buildable here, with its
#                                   check record bin/<target>.json (mdk
#                                   --version, --spec-version, mdk
#                                   conformance; or why it was not built)
#   wheel/                          mdkernel_core wheel (maturin)
#   obsidian/mdkernel-<ver>.zip     main.js, manifest.json, styles.css
#   spec-<spec>/                    SPEC.md, CONFORMANCE.md, fixtures/,
#                                   conformance.json (level 3), LICENSE
#   homebrew/mdk.rb                 formula draft (macOS arm64 sha256)
#   SHA256SUMS
# Targets: aarch64-apple-darwin (native), x86_64-unknown-linux-gnu
# (cross-compiled in a named Debian trixie container, run in a named amd64
# container), x86_64-apple-darwin and x86_64-pc-windows-msvc (attempted /
# CI only: .github/workflows/release.yml). A target that cannot be built is
# recorded with its reason, never silently skipped.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist="${1:-$root/dist}"
mkdir -p "$dist"
dist="$(cd "$dist" && pwd)"
cd "$root"
ver="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
spec="$(sed -n 's/^pub const SPEC_VERSION: &str = "\(.*\)";$/\1/p' server/src/tools.rs)"
[ -n "$ver" ] && [ -n "$spec" ] || { echo "release: cannot read versions" >&2; exit 1; }
rm -rf "$dist/bin" "$dist/wheel" "$dist/obsidian" "$dist/spec-$spec" "$dist/homebrew" "$dist/SHA256SUMS"
mkdir -p "$dist/bin" "$dist/wheel" "$dist/obsidian" "$dist/homebrew"
models="$root/models"
[ -f "$models/MANIFEST.json" ] || { echo "release: models/ missing (make models)" >&2; exit 1; }

# record <target> <json>
record() { echo "$2" | jq --arg t "$1" '{target: $t} + .' >"$dist/bin/$1.json"; }
# check <target> <runner…>: version, spec version, conformance of a built binary
check() {
  local t="$1"; shift
  local v s c rc=0
  v="$("$@" --version 2>&1 || true)"
  s="$("$@" --spec-version 2>&1 || true)"
  c="$("$@" conformance 2>/dev/null)" || rc=$?
  record "$t" "$(jq -n --arg v "$v" --arg s "$s" --argjson rc "$rc" --arg c "$c" --arg ver "$ver" --arg spec "$spec" '
    ($c | try fromjson catch null) as $r
    | {built: true, version: $v, spec_version: $s,
       conformance: (if $r == null then {error: ($c | .[0:400])} else {level_claimed: $r.level_claimed, passed: $r.passed, total: $r.total, fixtures_manifest: $r.fixtures_manifest} end),
       conformance_exit: $rc,
       ok: ($v == ("mdk " + $ver) and $s == $spec and $r != null and $r.level_claimed == 3 and $r.passed == 34 and $r.total == 34)}')"
}
pack() { # pack <target> <binary>
  local t="$1" b="$2" d="$dist/bin/mdk-$ver-$1"
  rm -rf "$d"; mkdir -p "$d/models"
  cp "$b" "$d/mdk"; cp LICENSE README.md "$d/"; cp "$models/MANIFEST.json" "$d/models/"
  cp scripts/fetch_model.sh "$d/models/fetch_model.sh"
  (cd "$dist/bin" && tar -czf "mdk-$ver-$t.tar.gz" "mdk-$ver-$t" && rm -rf "mdk-$ver-$t")
}

# ---- aarch64-apple-darwin (native)
cargo build -q --release --locked -p mdkernel-server --bin mdk
MDK_MODELS="$models" check aarch64-apple-darwin "$root/target/release/mdk"
pack aarch64-apple-darwin "$root/target/release/mdk"

# ---- x86_64-apple-darwin: the pinned onnxruntime crate (ort =2.0.0-rc.13,
# download-binaries) ships no prebuilt library for this target
if cargo build -q --release --locked --target x86_64-apple-darwin -p mdkernel-server --bin mdk 2>"$dist/bin/x86_64-apple-darwin.log"; then
  MDK_MODELS="$models" check x86_64-apple-darwin arch -x86_64 "$root/target/x86_64-apple-darwin/release/mdk"
  pack x86_64-apple-darwin "$root/target/x86_64-apple-darwin/release/mdk"
else
  record x86_64-apple-darwin "$(jq -n --arg e "$(grep -m1 -E '^error' "$dist/bin/x86_64-apple-darwin.log" || tail -1 "$dist/bin/x86_64-apple-darwin.log")" \
    '{built: false, reason: $e, ok: false}')"
fi

# ---- x86_64-unknown-linux-gnu: cross-compiled in a named arm64 Debian
# trixie container (the prebuilt onnxruntime needs glibc >= 2.38), run in a
# named amd64 container; only containers this script starts are touched
lt="$root/target/docker-linux-trixie"
if command -v docker >/dev/null && docker run --rm --name "mdk-release-linux-build-$$" --platform linux/arm64 \
    -v "$root":/src:ro -v "$lt":/target -v mdk-cargo-home-arm:/usr/local/cargo/registry \
    -v "$root/scripts/linux_cross_build.sh":/cross.sh:ro -w /src -e CARGO_TARGET_DIR=/target -e RUSTUP_TOOLCHAIN=1.92.0 \
    rust:1.92.0-trixie bash /cross.sh >"$dist/bin/x86_64-unknown-linux-gnu.log" 2>&1; then
  lb="$lt/x86_64-unknown-linux-gnu/release/mdk"
  run_linux() { docker run --rm --name "mdk-release-linux-run-$$-$RANDOM" --platform linux/amd64 -v "$lb":/mdk:ro -v "$models":/models:ro \
    -e MDK_MODELS=/models debian:trixie-slim /mdk "$@"; }
  check x86_64-unknown-linux-gnu run_linux
  pack x86_64-unknown-linux-gnu "$lb"
else
  record x86_64-unknown-linux-gnu "$(jq -n --arg e "$(grep -m1 -E 'error' "$dist/bin/x86_64-unknown-linux-gnu.log" || echo 'docker unavailable')" '{built: false, reason: $e, ok: false}')"
fi

# ---- x86_64-pc-windows-msvc: needs MSVC (or a Windows runner); CI only
record x86_64-pc-windows-msvc '{"built": false, "reason": "no MSVC toolchain or Windows host here; built and checked by .github/workflows/release.yml (windows-latest)", "ok": false}'

# ---- wheel
(cd research && uv run --frozen maturin build --release --quiet -m ../bindings/python/Cargo.toml -o "$dist/wheel") >/dev/null

# ---- Obsidian plugin package (community store submission: not done)
(cd clients/obsidian && { [ -d node_modules ] || npm ci --no-audit --no-fund >/dev/null; } && npm run -s build)
o="$dist/obsidian/mdkernel-$ver"
mkdir -p "$o"
cp clients/obsidian/main.js clients/obsidian/manifest.json clients/obsidian/styles.css "$o/"
(cd "$dist/obsidian" && zip -qr "mdkernel-$ver.zip" "mdkernel-$ver")

# ---- spec publication
sd="$dist/spec-$spec"
mkdir -p "$sd"
cp spec/SPEC.md spec/CONFORMANCE.md LICENSE "$sd/"
cp -R spec/fixtures "$sd/fixtures"
find "$sd/fixtures" -name '.DS_Store' -delete
MDK_MODELS="$models" "$root/target/release/mdk" conformance --fixtures "$sd/fixtures" >"$sd/conformance.json"

# ---- Homebrew formula draft (macOS arm64 archive; the URL is filled at
# publication, docs/RELEASE.md)
sha_arm="$(shasum -a 256 "$dist/bin/mdk-$ver-aarch64-apple-darwin.tar.gz" | cut -d' ' -f1)"
sed -e "s/@VERSION@/$ver/g" -e "s/@SHA256_AARCH64_APPLE_DARWIN@/$sha_arm/g" packaging/homebrew/mdk.rb.in >"$dist/homebrew/mdk.rb"

(cd "$dist" && find bin wheel obsidian homebrew "spec-$spec" -type f \( -name '*.tar.gz' -o -name '*.whl' -o -name '*.zip' -o -name '*.rb' -o -name 'conformance.json' \) | LC_ALL=C sort | xargs shasum -a 256 >SHA256SUMS)
commit="$(git rev-parse HEAD)"
dirty="$( [ -z "$(git status --porcelain --untracked-files=no)" ] && echo false || echo true)"
jq -s --arg c "$commit" --argjson d "$dirty" '{version: "'"$ver"'", spec_version: "'"$spec"'", commit: $c, worktree_dirty: $d, targets: ., built: ([.[] | select(.built)] | length), ok: ([.[] | select(.ok)] | length)}' "$dist"/bin/*.json >"$dist/release.json"
jq -r '.targets[] | "release: \(if .ok then "ok  " else "FAIL" end) \(.target)\(if .built then "" else " (not built: \(.reason))" end)"' "$dist/release.json"
jq -r '"release: \(.ok)/4 targets built and checked (E-7)"' "$dist/release.json"
