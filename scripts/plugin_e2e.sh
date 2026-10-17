#!/usr/bin/env bash
# The plugin e2e (GOAL P1-E E-2..E-6; TASKS E4): the plugin's command layer
# (clients/obsidian/src/commands.ts) against a real `rootmark serve --mcp stdio
# --http` on a clean copy of spec/fixtures/mini_vault, through a recording
# proxy (clients/obsidian/e2e/). Obsidian itself cannot run headless here
# (docs/rounds/P1-E/progress/ASSUMPTIONS.md E4-1).
#
#   scripts/plugin_e2e.sh [<out-dir>]     (default reports/rounds/P1-E/plugin_e2e)
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-$root/reports/rounds/P1-E/plugin_e2e}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
(cd "$root" && cargo build -q --release -p rootmark-server --bin rootmark)
cd "$root/clients/obsidian"
[ -d node_modules ] || npm ci --no-audit --no-fund >/dev/null
npm run -s e2e -- --rootmark "$root/target/release/rootmark" --vault-src "$root/spec/fixtures/mini_vault" --out "$out"
