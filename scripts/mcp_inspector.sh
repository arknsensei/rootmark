#!/usr/bin/env bash
# D-1 (GOAL P1-D): tool schemas through the official MCP Inspector.
#
#   scripts/mcp_inspector.sh <out-dir> [<tools-list.json>]
#
# 1. Vault: spec/fixtures/mini_vault (`mdk ingest-dir`), checkpoint `t0`,
#    one CLI read (one trace). Server: `mdk serve --vault vault --mcp stdio`
#    from an Inspector config file (cwd = <out-dir>).
# 2. `@modelcontextprotocol/inspector@2.9.0 --cli … --method tools/list` →
#    <out-dir>/tools_list.json (the list a client sees).
# 3. `server/tests/mcp_schemas.rs` checks that list (MDK_TOOLS_LIST): every
#    inputSchema valid against the JSON Schema 2020-12 meta-schema, an
#    object schema rejecting unknown arguments, same tools and schemas as
#    the server → <out-dir>/schemas.json.
# 4. Every tool is called once through the Inspector (`tools/call`, the
#    arguments typed by the Inspector from the schema) → calls/<tool>.json;
#    the call must not be an error.
# With <tools-list.json>, step 2 is skipped and that file is checked instead
# (deliberate-corruption tests). Result: <out-dir>/inspector.json; exit 1
# unless every listed tool is valid and every call succeeds.
# Needs Node.js (npx) and network access for the pinned Inspector package.
set -euo pipefail

INSPECTOR="@modelcontextprotocol/inspector@2.9.0"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: scripts/mcp_inspector.sh <out-dir> [<tools-list.json>]}"
given="${2:-}"
[ -z "$given" ] || given="$(cd "$(dirname "$given")" && pwd)/$(basename "$given")"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
cd "$root"
cargo build -q --release -p mdkernel-server --bin mdk
mdk="$root/target/release/mdk"

[ ! -e "$out/vault" ] || { echo "mcp_inspector: $out/vault exists" >&2; exit 1; }
"$mdk" ingest-dir "$out/vault" spec/fixtures/mini_vault >/dev/null
"$mdk" checkpoint create "$out/vault" --name t0 >/dev/null
lineage="$("$mdk" blame "$out/vault" espresso.md | jq -r '.blocks[0].lineage_id')"
"$mdk" read "$out/vault" --at t0 --consumer cli-prep "$lineage" >/dev/null
jq -n --arg mdk "$mdk" \
  '{mcpServers: {mdk: {command: $mdk, args: ["serve", "--vault", "vault", "--mcp", "stdio"]}}}' \
  >"$out/inspector_config.json"
insp() { npx -y "$INSPECTOR" --cli --config "$out/inspector_config.json" --server mdk --cwd "$out" "$@"; }

if [ -z "$given" ]; then
  insp --method tools/list >"$out/tools_list.json"
  list="$out/tools_list.json"
else
  list="$given"
fi

schema_ok=true
MDK_TOOLS_LIST="$list" MDK_SCHEMA_OUT="$out/schemas.json" \
  cargo test -q --release -p mdkernel-server --test mcp_schemas >"$out/schemas.log" 2>&1 || schema_ok=false
[ -f "$out/schemas.json" ] || { cat "$out/schemas.log" >&2; echo "mcp_inspector: no schemas.json" >&2; exit 1; }

# one Inspector call per tool, arguments as key=value (typed by the Inspector)
tool_args() {
  case "$1" in
    status|checkpoints) ;;
    history) echo "lineage_id=$lineage" ;;
    blame) echo "doc_path=espresso.md" ;;
    trace) echo "since_seq=0" ;;
    search) echo "query=espresso k=3" ;;
    read_block) echo "doc_path=espresso.md lineage_id=$lineage" ;;
    propose) echo "doc_path=inspector/note.md content=Inspector-proposed-text reason=inspector" ;;
    promote) echo "lineage_id=$lineage reason=inspector" ;;
    checkpoint) echo "name=inspector" ;;
    *) echo "mcp_inspector: no call arguments for tool $1" >&2; return 1 ;;
  esac
}
mkdir -p "$out/calls"
calls="[]"
for t in $(jq -r '.tools[].name' "$list"); do
  a="$(tool_args "$t")"
  ok=false
  # shellcheck disable=SC2086
  if insp --method tools/call --tool-name "$t" ${a:+--tool-arg $a} >"$out/calls/$t.json" 2>"$out/calls/$t.err" &&
     jq -e '.isError == false and (.content | length) > 0' "$out/calls/$t.json" >/dev/null; then
    ok=true
  fi
  calls="$(jq -c --arg t "$t" --argjson ok "$ok" '. + [{tool: $t, ok: $ok}]' <<<"$calls")"
done

jq -n --arg insp "$INSPECTOR" --slurpfile s "$out/schemas.json" --argjson calls "$calls" --argjson schema_ok "$schema_ok" '
  {inspector: $insp,
   tools_listed: $s[0].tools,
   tools_expected: $s[0].expected,
   schemas_valid: $s[0].valid,
   schema_check_pass: ($schema_ok and $s[0].pass),
   calls_ok: ([$calls[] | select(.ok)] | length),
   calls: $calls}
  | .pass = (.schema_check_pass and .schemas_valid == .tools_expected and .calls_ok == .tools_expected)' \
  >"$out/inspector.json"
jq -r '"mcp_inspector: schemas \(.schemas_valid)/\(.tools_expected) valid, Inspector calls \(.calls_ok)/\(.tools_expected)"' "$out/inspector.json"
jq -e .pass "$out/inspector.json" >/dev/null
