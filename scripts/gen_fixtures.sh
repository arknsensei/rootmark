#!/usr/bin/env bash
# Writes spec/fixtures/C1-*/expected.json and spec/fixtures/MANIFEST.json from
# the **v3-final** core (TASKS A3: "her fixture'ın beklenen değeri v3-final
# çekirdeğinden üretildi"). The evaluator source is core/tests/conformance/
# eval.rs — the same file `make conformance` compiles against the current core.
#
#   scripts/gen_fixtures.sh            # regenerate in place
#
# Fixtures are frozen (spec/CONFORMANCE.md, CLAUDE.md): rerunning this must
# leave every file byte-identical; a change needs a decision record.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sha="$(git -C "$here" rev-parse --verify 'v3-final^{commit}')"
work="$(mktemp -d -t mdk-fixture-gen.XXXXXX)"
trap 'rm -rf "$work"' EXIT

git -C "$here" archive "$sha" Cargo.lock rust-toolchain.toml .cargo core | tar -xf - -C "$work"
mkdir -p "$work/gen/src"
cp "$here/scripts/fixture-gen/Cargo.toml.in" "$work/gen/Cargo.toml"
cp "$here/scripts/fixture-gen/src/main.rs" "$work/gen/src/main.rs"
cp "$here/core/tests/conformance/eval.rs" "$work/gen/src/eval.rs"
cat >"$work/Cargo.toml" <<TOML
[workspace]
resolver = "2"
members = ["core", "gen"]
TOML
# --offline: resolve the generator's dependencies from the v3-final lock only
(cd "$work" && cargo build --offline --release -q -p mdk-fixture-gen)
"$work/target/release/mdk-fixture-gen" "$here/spec/fixtures" "$sha"
echo "fixtures written from v3-final core $sha"
