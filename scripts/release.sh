#!/usr/bin/env bash
# The 0.1.0 release package (GOAL P1-E "0.1.0 yayım paketi"; TASKS E5),
# built locally into dist/ (git-ignored). Tags nothing and publishes
# nothing: docs/RELEASE.md lists the human steps (the v0.1.0 tag is set
# only after the independent verification says "Güvenilir").
#
#   scripts/release.sh [<dist-dir>]        (default dist/)
#
# Produces:
#   bin/rootmark-<ver>-<target>.tar.gz   each target buildable here, with its
#                                   check record bin/<target>.json (rootmark
#                                   --version, --spec-version, rootmark
#                                   conformance; or why it was not built)
#   wheel/                          rootmark_core wheel (maturin)
#   obsidian/rootmark-<ver>.zip     main.js, manifest.json, styles.css
#   spec-<spec>/                    SPEC.md, CONFORMANCE.md, fixtures/,
#                                   conformance.json (level 3), LICENSE
#   homebrew/rootmark.rb                 formula draft (macOS arm64 sha256)
#   SHA256SUMS
#   release.json                    every target's record
#                                   (scripts/release_target.sh assemble)
#   plugin_e2e_real/                the real-Obsidian e2e (K-E1.3)
# Targets (K-E1.1): aarch64-apple-darwin (native) is the one release
# target; x86_64-unknown-linux-gnu (cross-compiled in a named Debian trixie
# container, run in a named amd64 container) and x86_64-pc-windows-msvc
# (CI only: .github/workflows/release.yml) are CI information. A target
# that cannot be built is recorded with its reason, never silently skipped.
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

# one target's record (scripts/release_target.sh: the same as in CI)
rt="$root/scripts/release_target.sh"
pack() { # pack <target> <binary>
  local t="$1" b="$2" d="$dist/bin/rootmark-$ver-$1"
  rm -rf "$d"; mkdir -p "$d/models"
  cp "$b" "$d/rootmark"; cp LICENSE README.md "$d/"; cp "$models/MANIFEST.json" "$d/models/"
  cp scripts/fetch_model.sh "$d/models/fetch_model.sh"
  (cd "$dist/bin" && tar -czf "rootmark-$ver-$t.tar.gz" "rootmark-$ver-$t" && rm -rf "rootmark-$ver-$t")
}

# ---- aarch64-apple-darwin (native): the one release target (K-E1.1)
cargo build -q --release --locked -p rootmark-server --bin rootmark
ROOTMARK_MODELS="$models" bash "$rt" check aarch64-apple-darwin true "$dist/bin/aarch64-apple-darwin.json" "$root/target/release/rootmark"
pack aarch64-apple-darwin "$root/target/release/rootmark"

# ---- x86_64-unknown-linux-gnu (CI info, not a release criterion; K-E1.1):
# cross-compiled in a named arm64 Debian trixie container (the prebuilt
# onnxruntime needs glibc >= 2.38), run in a named amd64 container; only
# containers this script starts are touched. ROOTMARK_RELEASE_LINUX=0 skips it.
lt="$root/target/docker-linux-trixie"
if [ "${ROOTMARK_RELEASE_LINUX:-1}" = 0 ]; then
  bash "$rt" skipped x86_64-unknown-linux-gnu false "$dist/bin/x86_64-unknown-linux-gnu.json" "not attempted here (ROOTMARK_RELEASE_LINUX=0); built and checked by .github/workflows/release.yml"
elif command -v docker >/dev/null && docker run --rm --name "rootmark-release-linux-build-$$" --platform linux/arm64 \
    -v "$root":/src:ro -v "$lt":/target -v rootmark-cargo-home-arm:/usr/local/cargo/registry \
    -v "$root/scripts/linux_cross_build.sh":/cross.sh:ro -w /src -e CARGO_TARGET_DIR=/target -e RUSTUP_TOOLCHAIN=1.92.0 \
    rust:1.92.0-trixie bash /cross.sh >"$dist/bin/x86_64-unknown-linux-gnu.log" 2>&1; then
  lb="$lt/x86_64-unknown-linux-gnu/release/rootmark"
  bash "$rt" check x86_64-unknown-linux-gnu false "$dist/bin/x86_64-unknown-linux-gnu.json" "$lb" \
    docker run --rm --name "rootmark-release-linux-run-$$" --platform linux/amd64 -v "$lb":/rootmark:ro -v "$models":/models:ro \
    -e ROOTMARK_MODELS=/models debian:trixie-slim /rootmark
  pack x86_64-unknown-linux-gnu "$lb"
else
  bash "$rt" skipped x86_64-unknown-linux-gnu false "$dist/bin/x86_64-unknown-linux-gnu.json" \
    "$(grep -m1 -E 'error' "$dist/bin/x86_64-unknown-linux-gnu.log" 2>/dev/null || echo 'docker unavailable')"
fi

# ---- x86_64-pc-windows-msvc (CI info): needs MSVC or a Windows runner
bash "$rt" skipped x86_64-pc-windows-msvc false "$dist/bin/x86_64-pc-windows-msvc.json" "no MSVC toolchain or Windows host here; built and checked by .github/workflows/release.yml (windows-latest)"
# x86_64-apple-darwin is not a target (K-E1.1)

# ---- wheel
(cd research && uv run --frozen maturin build --release --quiet -m ../bindings/python/Cargo.toml -o "$dist/wheel") >/dev/null

# ---- Obsidian plugin package (community store submission: not done)
(cd clients/obsidian && { [ -d node_modules ] || npm ci --no-audit --no-fund >/dev/null; } && npm run -s build)
o="$dist/obsidian/rootmark-$ver"
mkdir -p "$o"
cp clients/obsidian/main.js clients/obsidian/manifest.json clients/obsidian/styles.css "$o/"
(cd "$dist/obsidian" && zip -qr "rootmark-$ver.zip" "rootmark-$ver")

# ---- spec publication
sd="$dist/spec-$spec"
mkdir -p "$sd"
cp spec/SPEC.md spec/CONFORMANCE.md LICENSE "$sd/"
cp -R spec/fixtures "$sd/fixtures"
find "$sd/fixtures" -name '.DS_Store' -delete
ROOTMARK_MODELS="$models" "$root/target/release/rootmark" conformance --fixtures "$sd/fixtures" >"$sd/conformance.json"

# ---- Homebrew formula draft (macOS arm64 archive; the URL is filled at
# publication, docs/RELEASE.md)
sha_arm="$(shasum -a 256 "$dist/bin/rootmark-$ver-aarch64-apple-darwin.tar.gz" | cut -d' ' -f1)"
sed -e "s/@VERSION@/$ver/g" -e "s/@SHA256_AARCH64_APPLE_DARWIN@/$sha_arm/g" packaging/homebrew/rootmark.rb.in >"$dist/homebrew/rootmark.rb"

(cd "$dist" && find bin wheel obsidian homebrew "spec-$spec" -type f \( -name '*.tar.gz' -o -name '*.whl' -o -name '*.zip' -o -name '*.rb' -o -name 'conformance.json' \) | LC_ALL=C sort | xargs shasum -a 256 >SHA256SUMS)
# ---- the real-Obsidian e2e of this tree's plugin (K-E1.3; docs/RELEASE.md
# R15): mandatory for a release, recorded (a skip is recorded as a skip)
bash scripts/plugin_e2e_real.sh "$dist/plugin_e2e_real" || echo "release: real-Obsidian e2e failed (recorded in $dist/plugin_e2e_real/)" >&2

commit="$(git rev-parse HEAD)"
dirty="$( [ -z "$(git status --porcelain --untracked-files=no)" ] && echo false || echo true)"
bash "$rt" assemble "$dist" "$commit" "$dirty" || echo "release: the required target did not pass (see $dist/release.json)" >&2
