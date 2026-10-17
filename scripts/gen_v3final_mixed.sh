#!/usr/bin/env bash
# Writes core/tests/fixtures/v3_final_mixed/<scenario>/{store/, expected.json}
# from the **v3-final** core library (P1-B.1 TASKS B1.1, GOAL B1-1): the
# scenarios of scripts/v3final-mixed-gen/scenarios.json (non-human authors and
# mixed-author edits, which the v3-final CLI cannot write) are ingested through
# the v3-final public API; expected.json holds v3-final's own replay
# snapshot_ids and events_hash. The current core never runs here.
#
#   scripts/gen_v3final_mixed.sh [<out-dir>]   # default: the committed fixture dir
#
# Rerunning on a clean checkout must leave the fixtures byte-identical
# (`git status` clean).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-$here/core/tests/fixtures/v3_final_mixed}"
sha="$(git -C "$here" rev-parse --verify 'v3-final^{commit}')"
work="$(mktemp -d -t rootmark-v3mixed.XXXXXX)"
trap 'rm -rf "$work"' EXIT

git -C "$here" archive "$sha" Cargo.lock rust-toolchain.toml .cargo core | tar -xf - -C "$work"
mkdir -p "$work/gen/src"
cp "$here/scripts/v3final-mixed-gen/Cargo.toml.in" "$work/gen/Cargo.toml"
cp "$here/scripts/v3final-mixed-gen/src/main.rs" "$work/gen/src/main.rs"
cat >"$work/Cargo.toml" <<TOML
[workspace]
resolver = "2"
members = ["core", "gen"]
TOML
# --offline: resolve the generator's dependencies from the v3-final lock only
(cd "$work" && cargo build --offline --release -q -p rootmark-v3final-mixed-gen)
mkdir -p "$out"
"$work/target/release/rootmark-v3final-mixed-gen" "$here/scripts/v3final-mixed-gen/scenarios.json" "$out" "$sha"
echo "v3_final_mixed fixtures written from v3-final core $sha into $out"
