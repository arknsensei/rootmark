#!/usr/bin/env bash
# The real-Obsidian e2e (K-E1.3; GOAL P1-E.1 E1-3, E1-9): the plugin bundle
# in a fresh copy of spec/fixtures/mini_vault, a separate Obsidian instance
# (own --user-data-dir, CDP) driving the plugin's panels against a real
# `rootmark serve --mcp stdio --http` through a recording proxy; then the
# connection screen on the p1d-final `/2` index fixture
# (server/tests/fixtures/index_v2) and its rebuild
# (clients/obsidian/e2e/real/). The user's Obsidian and its configuration
# are never touched; only the instance started here is stopped.
#
# Without an Obsidian binary (CI) nothing runs: <out>/plugin_e2e_real.json
# says {"skipped": true, "reason": …} and the script exits 0 — a skip is
# reported, never a pass (K-E1.3; mandatory locally and in docs/RELEASE.md).
#
#   scripts/plugin_e2e_real.sh [<out-dir>]   (default reports/rounds/P1-E.1/plugin_e2e_real)
#   ROOTMARK_OBSIDIAN: the Obsidian binary (default
#   /Applications/Obsidian.app/Contents/MacOS/Obsidian); ROOTMARK_E2E_PLUGIN_DIR:
#   another directory with main.js, manifest.json, styles.css (deliberate
#   tests, scripts/test_tamper_p1e1_e2e.sh)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-$root/reports/rounds/P1-E.1/plugin_e2e_real}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
obsidian="${ROOTMARK_OBSIDIAN:-/Applications/Obsidian.app/Contents/MacOS/Obsidian}"
if [ ! -x "$obsidian" ]; then
  jq -n --arg r "no Obsidian binary at $obsidian (set ROOTMARK_OBSIDIAN)" '{skipped: true, reason: $r, pass: null}' >"$out/plugin_e2e_real.json"
  echo "plugin-e2e-real: SKIPPED — no Obsidian binary at $obsidian" >&2
  exit 0
fi
(cd "$root" && cargo build -q --release -p rootmark-server --bin rootmark)
cd "$root/clients/obsidian"
[ -d node_modules ] || npm ci --no-audit --no-fund >/dev/null
plugin_dir="${ROOTMARK_E2E_PLUGIN_DIR:-}"
if [ -z "$plugin_dir" ]; then
  npm run -s build
  plugin_dir="$root/clients/obsidian"
fi
npm run -s e2e:real -- --rootmark "$root/target/release/rootmark" --obsidian "$obsidian" \
  --vault-src "$root/spec/fixtures/mini_vault" --old-index "$root/server/tests/fixtures/index_v2/vault" \
  --plugin "$plugin_dir" --out "$out"
