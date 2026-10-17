#!/usr/bin/env bash
# P1-D deliberate corruptions (TASKS D7: "tamper (D-11, token, X-Consumer,
# Inspector şema bozma)"). Each must be rejected; each group has an
# untouched control that must pass.
#
#   scripts/test_tamper_p1d.sh <tools_list.json> <out.json>
#
# <tools_list.json>: the `tools/list` result the official MCP Inspector
# printed (scripts/mcp_inspector.sh).
# 1. schemas: copies of the list with one corruption each — a tool's
#    inputSchema type misspelt, `additionalProperties: true`, a tool
#    removed, a `required` name that is not a property, a property type that
#    is not a JSON Schema type, a schema differing from the server's — must
#    fail `server/tests/mcp_schemas.rs`; the untouched list passes.
# 2. token: a token file with mode 644, a token file that is not 64 hex
#    characters → `rootmark serve --http` refuses to start; on a running server a
#    wrong token, another vault's token, a token without "Bearer " → 401;
#    the right token → 200 (control).
# 3. X-Consumer: missing, empty, whitespace → 400 (right token).
# 4. D-11 over MCP: `propose` with `../`, a path outside the vault, content
#    over 10 MB, `replace_section` with a missing heading → tool error, and
#    no file written; a valid `propose` succeeds (control).
# Nothing in the source tree is modified (temporary copies only). Exit 1 if
# any corruption is accepted or any control fails.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
list="$(cd "$(dirname "${1:?usage: test_tamper_p1d.sh <tools_list.json> <out.json>}")" && pwd)/$(basename "$1")"
out="${2:?usage: test_tamper_p1d.sh <tools_list.json> <out.json>}"
cd "$root"
cargo build -q --release -p rootmark-server --bin rootmark --example mcp_client
cargo test -q --release -p rootmark-server --test mcp_schemas --no-run 2>/dev/null
rootmark="$root/target/release/rootmark"
client="$root/target/release/examples/mcp_client"
work="$(mktemp -d -t rootmark-tamper-p1d.XXXXXX)"
pid=""
trap '[ -z "$pid" ] || kill "$pid" 2>/dev/null || true; rm -rf "$work"' EXIT

results="[]"
record() { # record <group> <case> <expected: rejected|accepted> <true|false>
  results="$(jq -c --arg g "$1" --arg c "$2" --arg e "$3" --argjson ok "$4" \
    '. + [{group: $g, case: $c, expected: $e, as_expected: $ok}]' <<<"$results")"
  if [ "$4" = true ]; then echo "tamper P1-D: ok   [$1] $2 ($3)"; else echo "tamper P1-D: WRONG [$1] $2 (expected $3)" >&2; fi
}

# 1. schemas
schema_case() { # schema_case <label> <jq filter on the list>
  jq "$2" "$list" >"$work/list.json"
  if ROOTMARK_TOOLS_LIST="$work/list.json" cargo test -q --release -p rootmark-server --test mcp_schemas >/dev/null 2>&1; then
    record schema "$1" rejected false
  else
    record schema "$1" rejected true
  fi
}
if ROOTMARK_TOOLS_LIST="$list" cargo test -q --release -p rootmark-server --test mcp_schemas >/dev/null 2>&1; then
  record schema "untouched Inspector list (control)" accepted true
else
  record schema "untouched Inspector list (control)" accepted false
fi
schema_case "inputSchema type misspelt" '.tools[0].inputSchema.type = "objekt"'
schema_case "additionalProperties true" '(.tools[] | select(.name == "search") | .inputSchema.additionalProperties) = true'
schema_case "a tool removed" 'del(.tools[] | select(.name == "promote"))'
schema_case "required names no property" '(.tools[] | select(.name == "blame") | .inputSchema.required) += ["nope"]'
schema_case "property type not a JSON Schema type" '(.tools[] | select(.name == "history") | .inputSchema.properties.lineage_id.type) = "uuid"'
schema_case "schema differs from the server's" '(.tools[] | select(.name == "search") | .inputSchema.properties.k.maximum) = 5'

# a vault for 2–4
v="$work/vault"
mkdir "$v"
cp "$root"/spec/fixtures/mini_vault/*.md "$v/"
"$rootmark" ingest-dir "$v" "$v" >/dev/null
start() {
  "$rootmark" serve --vault "$v" --http 127.0.0.1:0 2>"$work/serve.err" &
  pid=$!
  for _ in $(seq 1 600); do
    if grep -q "http listening on" "$work/serve.err" 2>/dev/null; then
      port="$(sed -n 's/.*http listening on 127\.0\.0\.1:\([0-9][0-9]*\).*/\1/p' "$work/serve.err")"
      return 0
    fi
    kill -0 "$pid" 2>/dev/null || { cat "$work/serve.err" >&2; return 1; }
    sleep 0.1
  done
  return 1
}
stop() { kill -TERM "$pid"; wait "$pid" || true; pid=""; }
code() { curl -s -o /dev/null -w '%{http_code}' "$@"; }

# 2. token
start
stop
tokf="$v/.rootmark/server.token"
tok="$(tr -d '\n' <"$tokf")"
cp "$tokf" "$work/token.good"
chmod 644 "$tokf"
rc=0; "$rootmark" serve --vault "$v" --http 127.0.0.1:0 2>"$work/t1.err" || rc=$?
record token "token file mode 644 → start refused" rejected "$( [ "$rc" != 0 ] && grep -q "must be 600" "$work/t1.err" && echo true || echo false)"
chmod 600 "$tokf"
printf 'not-a-token\n' >"$tokf"
rc=0; "$rootmark" serve --vault "$v" --http 127.0.0.1:0 2>"$work/t2.err" || rc=$?
record token "token file not 64 hex → start refused" rejected "$( [ "$rc" != 0 ] && grep -q "is not a token" "$work/t2.err" && echo true || echo false)"
cp "$work/token.good" "$tokf"
chmod 600 "$tokf"
other="$(printf '%064d' 0 | tr 0 a)"
start
base="http://127.0.0.1:$port/v1/status"
c="$(code -H "Authorization: Bearer $tok" -H 'X-Consumer: tamper' "$base")"
record token "right token (control)" accepted "$( [ "$c" = 200 ] && echo true || echo false)"
c="$(code -H "Authorization: Bearer ${tok:0:63}x" -H 'X-Consumer: tamper' "$base")"
record token "wrong token → 401" rejected "$( [ "$c" = 401 ] && echo true || echo false)"
c="$(code -H "Authorization: Bearer $other" -H 'X-Consumer: tamper' "$base")"
record token "another vault's token → 401" rejected "$( [ "$c" = 401 ] && echo true || echo false)"
c="$(code -H "Authorization: $tok" -H 'X-Consumer: tamper' "$base")"
record token "token without Bearer → 401" rejected "$( [ "$c" = 401 ] && echo true || echo false)"

# 3. X-Consumer
c="$(code -H "Authorization: Bearer $tok" "$base")"
record x-consumer "missing → 400" rejected "$( [ "$c" = 400 ] && echo true || echo false)"
c="$(code -H "Authorization: Bearer $tok" -H 'X-Consumer:' "$base")"
record x-consumer "empty → 400" rejected "$( [ "$c" = 400 ] && echo true || echo false)"
c="$(code -H "Authorization: Bearer $tok" -H 'X-Consumer:    ' "$base")"
record x-consumer "whitespace → 400" rejected "$( [ "$c" = 400 ] && echo true || echo false)"
stop

# 4. D-11 over MCP
{
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"tamper","version":"1.0"}}}'
  echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"propose","arguments":{"doc_path":"../escape.md","content":"x","reason":"t"}}}'
  printf '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"propose","arguments":{"doc_path":"%s","content":"x","reason":"t"}}}\n' "$work/outside.md"
  printf '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"propose","arguments":{"doc_path":"big.md","reason":"t","content":"%s"}}}\n' "$(head -c 10500001 /dev/zero | tr '\0' 'b')"
  echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"propose","arguments":{"doc_path":"espresso.md","content":"x","reason":"t","mode":"replace_section","heading":"No Such Heading"}}}'
  echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"propose","arguments":{"doc_path":"tamper/ok.md","content":"control","reason":"t"}}}'
} >"$work/d11.jsonl"
before="$(shasum -a 256 "$v/espresso.md" | cut -d' ' -f1)"
"$client" "$work/d11.jsonl" "$work/d11_t.jsonl" -- "$rootmark" serve --vault "$v" --mcp stdio 2>>"$work/serve.err"
err() { jq -e --argjson id "$1" 'select(.request.id == $id) | .response.result.isError == true' "$work/d11_t.jsonl" >/dev/null && echo true || echo false; }
e2="$(err 2)"; [ ! -e "$work/escape.md" ] || e2=false
record d-11 "propose ../escape.md → error, nothing written" rejected "$e2"
e3="$(err 3)"; [ ! -e "$work/outside.md" ] || e3=false
record d-11 "propose outside the vault → error, nothing written" rejected "$e3"
e4="$(err 4)"; [ ! -e "$v/big.md" ] || e4=false
record d-11 "propose over 10 MB → error, nothing written" rejected "$e4"
e5="$(err 5)"; [ "$(shasum -a 256 "$v/espresso.md" | cut -d' ' -f1)" = "$before" ] || e5=false
record d-11 "replace_section with a missing heading → error, file unchanged" rejected "$e5"
ok6="$(jq -e 'select(.request.id == 6) | .response.result.isError == false' "$work/d11_t.jsonl" >/dev/null && [ -f "$v/tamper/ok.md" ] && echo true || echo false)"
record d-11 "valid propose (control)" accepted "$ok6"

jq -n --argjson r "$results" '
  {cases: $r,
   rejected: ([$r[] | select(.expected == "rejected" and .as_expected)] | length),
   rejected_total: ([$r[] | select(.expected == "rejected")] | length),
   controls_ok: ([$r[] | select(.expected == "accepted" and .as_expected)] | length),
   controls_total: ([$r[] | select(.expected == "accepted")] | length)}
  | .pass = (.rejected == .rejected_total and .controls_ok == .controls_total)' >"$out"
jq -r '"tamper P1-D: \(.rejected)/\(.rejected_total) corruptions rejected, \(.controls_ok)/\(.controls_total) controls ok"' "$out"
jq -e .pass "$out" >/dev/null
