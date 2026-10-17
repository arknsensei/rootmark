#!/usr/bin/env bash
# P1-E/E1: clients/obsidian/src/contract.gen.ts from the contract — the
# argument JSON Schemas of a live `rootmark serve --mcp stdio` (`tools/list`,
# docs/CLIENT_CONTRACT.md §2 points there) and the §2/§3 tables of
# docs/CLIENT_CONTRACT.md (clients/obsidian/scripts/gen-types.mjs).
#
#   scripts/gen_client_types.sh           write the file
#   scripts/gen_client_types.sh --check   exit 1 if the committed file differs
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# ROOTMARK_CLIENT_TYPES: another file to write/check (deliberate tests)
target="${ROOTMARK_CLIENT_TYPES:-$here/clients/obsidian/src/contract.gen.ts}"
work="$(mktemp -d -t rootmark-gen-types.XXXXXX)"
trap 'rm -rf "$work"' EXIT
(cd "$here" && cargo build -q --release -p rootmark-server --bin rootmark --example mcp_pipeline)
rootmark="$here/target/release/rootmark"
mkdir -p "$work/vault"
printf '# A\n\nText.\n' >"$work/vault/a.md"
"$rootmark" ingest-dir "$work/vault" "$work/vault" >/dev/null
cat >"$work/list.jsonl" <<'J'
{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"gen-client-types","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
J
"$here/target/release/examples/mcp_pipeline" "$work/list.jsonl" "$work/list.out" -- \
  "$rootmark" serve --vault "$work/vault" --mcp stdio 2>"$work/serve.err"
jq -e 'select(.request.id == 1) | .response.result' "$work/list.out" >"$work/tools.json"
node "$here/clients/obsidian/scripts/gen-types.mjs" "$work/tools.json" "$here/docs/CLIENT_CONTRACT.md" >"$work/contract.gen.ts"
if [ "${1:-}" = "--check" ]; then
  if ! cmp -s "$work/contract.gen.ts" "$target"; then
    diff -u "$target" "$work/contract.gen.ts" >&2 || true
    echo "gen_client_types: $target is out of date (run scripts/gen_client_types.sh)" >&2
    exit 1
  fi
  echo "gen_client_types: up to date" >&2
else
  cp "$work/contract.gen.ts" "$target"
  echo "gen_client_types: wrote $target" >&2
fi
