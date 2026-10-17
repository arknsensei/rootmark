#!/usr/bin/env bash
# The P1-E.1 old-index fixture (K-E1.2, GOAL E1-1): a small vault whose
# index was written by the `p1d-final` `mdk` itself (format
# `mdkernel-index/2`), not a relabelled current manifest.
#
#   scripts/gen_old_index.sh [--check]
#       builds the p1d-final tree (git archive, its own lock and ONNX
#       Runtime; or MDK_OLD=<p1d-final mdk>), writes two documents into a
#       fresh vault, `mdk ingest-dir` + `mdk index build` with that binary and
#       puts the vault at server/tests/fixtures/index_v2/vault/. With
#       --check the vault is generated elsewhere and must equal the committed
#       one byte for byte (exit 1 otherwise).
# Needs the model files (`make models`, ROOTMARK_MODELS or MDK_MODELS).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dest="$here/server/tests/fixtures/index_v2/vault"
check=false
[ "${1:-}" = "--check" ] && check=true
work="$(mktemp -d -t rootmark-oldindex.XXXXXX)"
trap 'rm -rf "$work"' EXIT
# the p1d-final binary reads the model directory under its own name
export MDK_MODELS="${MDK_MODELS:-${ROOTMARK_MODELS:-$here/models}}"

old="${MDK_OLD:-}"
if [ -z "$old" ]; then
  sha="$(git -C "$here" rev-parse --verify 'p1d-final^{commit}')"
  mkdir -p "$work/p1d"
  git -C "$here" archive "$sha" | tar -xf - -C "$work/p1d"
  (cd "$work/p1d" && cargo build -q --release -p mdkernel-server --bin mdk)
  old="$work/p1d/target/release/mdk"
fi
[ -x "$old" ] || { echo "gen_old_index: no p1d-final mdk at $old" >&2; exit 1; }

v="$work/vault"
mkdir -p "$v"
cat >"$v/espresso.md" <<'EOF'
# Espresso

Dose: 18 g of coffee for a double shot.

## Grind

Grind fine enough that the shot runs in 28 seconds.

## Water

Use filtered water at 93 degrees.
EOF
cat >"$v/safety.md" <<'EOF'
# Safety

Intro written by a human.

## Fire

Keep a class D extinguisher next to the roaster.
EOF
"$old" ingest-dir "$v" "$v" >/dev/null
"$old" index build "$v" >/dev/null
fmt="$(jq -r .format "$v/.mdkernel/index/manifest.json")"
[ "$fmt" = "mdkernel-index/2" ] || { echo "gen_old_index: format $fmt, not mdkernel-index/2" >&2; exit 1; }

if $check; then
  diff -r "$dest" "$v" >&2 || { echo "gen_old_index: committed fixture differs" >&2; exit 1; }
  echo "gen_old_index: fixture up to date ($fmt)"
else
  rm -rf "$dest"
  mkdir -p "$(dirname "$dest")"
  cp -R "$v" "$dest"
  echo "gen_old_index: wrote $dest ($fmt)"
fi
