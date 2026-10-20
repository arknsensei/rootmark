#!/usr/bin/env bash
# D1-13 (GOAL P1-D.1; TASKS D1.5): docs/CLIENT_CONTRACT.md against the server
# and the tests.
#   1. the tools of the contract (§2 table, first column) are exactly the
#      tools a live `rootmark serve --mcp stdio` lists (`tools/list`);
#   2. the error codes of the contract (§3 table) are exactly
#      `server/src/error.rs` `Code::as_str`, with the same HTTP status
#      (`Code::http_status`);
#   3. every tool and every code is matched by a test of
#      server/tests/contract.rs: the tool by a real call
#      (`call_tool(&k, "<tool>", …)`), the code by a real call expecting it
#      (`expect_code(&k, …, Code::<Variant>);`); comments do not count;
#   4. (P1-E/E0, K-E0.2 Ö3/Ş4) over real HTTP against a live
#      `rootmark serve --http 127.0.0.1:0`: every §1 endpoint (`POST /v1/<tool>`
#      for each §2 tool, `GET /v1/status`) answers 200 with a JSON object
#      and no error; every §3 code is triggered by a real request and
#      answers the status of its §3 row with that `error.code`; the §1/§3
#      transport rules (no token → 401 `denied`, unknown path → 404
#      `not_found`, GET of a tool → 405 `invalid_arguments`). A contract
#      code with no HTTP trigger here is missing (`http-code:<code>`).
#      (P1-E.1/E1.1, K-E1.2, E1-8) `status` does not depend on the index:
#      with the index `ok`, `stale` (format aged), `damaged` (manifest not
#      JSON) and `missing` (index removed), `GET` and `POST /v1/status`
#      answer 200 with `index.state` of that name and every field the
#      contract's §2 `status` row lists under `index {…}`, while `search`
#      answers 503 `index_stale` in the three non-`ok` states (the 503
#      exception covers index calls, not `status`; `http.status_index`,
#      `missing` entries `http-status-index:<state>`). `internal` is now
#      triggered by an index file that does not match its manifest (an
#      unreadable manifest is `index_stale`, K-E1.2).
#      (P2-0/Z2, K-P2-0.5) the human channel's own endpoints of §2.6
#      (`| \`<name>\` |` rows under "### 2.6"): none is an MCP tool
#      (`tools/list`), and over HTTP `attest_session` answers 200 with
#      `session_id, key (64 hex), ttl_s, stage`; `attest` with a MAC made with
#      that key (HMAC-SHA256, `openssl`) answers 200 with every field of its
#      row; a wrong MAC answers 403 `denied` (`http.http_only`; a missing one
#      is `http-only:<name>`).
#      (P2-A/A2, freeze exception P2-A/F2) `migration_required` (an older
#      rootmark's index: format aged to `mdkernel-index/2`) is triggered
#      before `index_stale` (now: a format this rootmark does not know,
#      `rootmark-index/99`); in the aged (`stale`) state `search` answers 503
#      `migration_required`, in `damaged` and `missing` 503 `index_stale`;
#      the `status` row's `migrations {…}` group is among the checked fields.
#      (P2-0/Z0, P1-E.1 verification recommendation 2) in each of those
#      states both `status` answers also hold every field of the §2 `status`
#      row: every top-level name and every name of its `{…}` groups
#      (`http.status_fields`; a missing one is `http-status-fields:<state>`).
# Prints and writes (<out.json>, optional) {tools, codes, http, missing,
# pass}; exit 1 if anything is missing or differs.
#
#   scripts/contract_check.sh [<out.json>]   (ROOTMARK_CONTRACT: another contract
#   file; ROOTMARK_BIN: another `rootmark` binary to serve, e.g. a deliberately broken
#   build — `scripts/test_tamper_p1e.sh`)
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-}"
contract="${ROOTMARK_CONTRACT:-$here/docs/CLIENT_CONTRACT.md}"
work="$(mktemp -d -t rootmark-contract.XXXXXX)"
trap 'rm -rf "$work"' EXIT

(cd "$here" && cargo build -q --release -p rootmark-server --bin rootmark --example mcp_pipeline)
rootmark="${ROOTMARK_BIN:-$here/target/release/rootmark}"
pipe="$here/target/release/examples/mcp_pipeline"

# 1. tools: contract §2 rows `| \`name\` | …` between "## 2." and "### 2.1"
#    (the tools table; P2-0: §2.6's HTTP-only endpoints are not tools)
contract_tools="$(sed -n '/^## 2\. /,/^### 2\.1 /p' "$contract" | grep -E '^\| `[a-z_]+` \|' | sed -E 's/^\| `([a-z_]+)`.*/\1/' | sort)"
# §2.6: the HTTP-only endpoints (P2-0)
http_only="$(sed -n '/^### 2\.6 /,/^## 3\. /p' "$contract" | grep -E '^\| `[a-z_]+` \|' | sed -E 's/^\| `([a-z_]+)`.*/\1/' | sort)"
mkdir -p "$work/vault"
printf '# A\n\nText.\n' >"$work/vault/a.md"
"$rootmark" ingest-dir "$work/vault" "$work/vault" >/dev/null
cat >"$work/list.jsonl" <<'J'
{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"contract-check","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
J
# pipelined: the server completes the initialize handshake before it reads
# tools/list
"$pipe" "$work/list.jsonl" "$work/list.out" -- "$rootmark" serve --vault "$work/vault" --mcp stdio 2>"$work/serve.err"
server_tools="$(jq -r 'select(.request.id == 1) | .response.result.tools[].name' "$work/list.out" | sort)"

# 2. codes: contract §3 rows `| \`code\` | <status> |`; error.rs mappings
contract_codes="$(sed -n '/^## 3\. /,/^## 4\. /p' "$contract" | grep -E '^\| `[a-z_]+` \| [0-9]{3} \|' | sed -E 's/^\| `([a-z_]+)` \| ([0-9]{3}) \|.*/\1 \2/' | sort)"
err="$here/server/src/error.rs"
# Code::Variant => "name"
names="$(grep -E '^[[:space:]]+Code::[A-Za-z]+ => "[a-z_]+",' "$err" | sed -E 's/^[[:space:]]+Code::([A-Za-z]+) => "([a-z_]+)",/\1 \2/')"
# http_status arms: `Code::A | Code::B => 400,`
statuses="$(sed -n '/pub fn http_status/,/^    }/p' "$err" | grep -E '=> [0-9]{3},' | while IFS= read -r line; do
  st="$(echo "$line" | sed -E 's/.*=> ([0-9]{3}),.*/\1/')"
  echo "$line" | grep -oE 'Code::[A-Za-z]+' | sed 's/Code:://' | while read -r v; do echo "$v $st"; done
done)"
server_codes="$(echo "$names" | while read -r v n; do
  st="$(echo "$statuses" | awk -v v="$v" '$1 == v {print $2}')"
  echo "$n $st"
done | sort)"

# 3. tests: server/tests/contract.rs, comments stripped, whitespace
#    collapsed; a tool counts only as `call_tool(&k, "<tool>"`, a code only as
#    the expectation of `expect_code(&k, …, Code::<Variant>);` (real calls,
#    not a mention)
tests="$here/server/tests/contract.rs"
flat="$(perl -0pe 's{/\*.*?\*/}{}gs; s{//[^\n]*}{}g; s{\s+}{ }g' "$tests")"
missing=()
while read -r t; do
  [ -n "$t" ] || continue
  if ! printf '%s' "$flat" | grep -qE "call_tool\( ?&k, \"$t\","; then missing+=("tool:$t"); fi
done <<<"$contract_tools"
while read -r c st; do
  [ -n "$c" ] || continue
  v="$(echo "$names" | awk -v n="$c" '$2 == n {print $1}')"
  if ! printf '%s' "$flat" | perl -ne 'BEGIN{$v=shift} $f.=$_; END{ exit(($f =~ /expect_code\( ?&k,[^;]*Code::$v\s*,?\s*\)\s*;/) ? 0 : 1) }' "$v"; then missing+=("code:$c"); fi
done <<<"$contract_codes"

# 4. HTTP: a live server on a vault of one human document
hv="$work/http_vault"
mkdir -p "$hv"
printf '# A\n\nSome text about espresso.\n' >"$hv/a.md"
"$rootmark" ingest-dir "$hv" "$hv" >/dev/null
cat >"$hv/.rootmark/policy.toml" <<'T'
[promotion]
policy = "human_only"

[edit]
policy = "append_only"

[version_trigger]
idle_ms = 60000
min_sim_change = 0.15

[journal]
retention_hours = 24

[server]
max_propose_bytes = 64

[index]
auto_rebuild = false

[identity]
human_id = "contract-http"
T
"$rootmark" serve --vault "$hv" --http 127.0.0.1:0 </dev/null >/dev/null 2>"$work/http.err" &
serve_pid=$!
trap 'kill "$serve_pid" 2>/dev/null || true; rm -rf "$work"' EXIT
port=""
for _ in $(seq 1 600); do
  # any binary name: ROOTMARK_BIN may be an older build (`mdk serve: …`, K-GH.1)
  port="$(sed -n 's/^[a-z]* serve: http listening on 127\.0\.0\.1:\([0-9]*\)$/\1/p' "$work/http.err")"
  [ -n "$port" ] && break
  kill -0 "$serve_pid" 2>/dev/null || break
  sleep 0.1
done
if [ -z "$port" ]; then
  cat "$work/http.err" >&2
  echo "contract_check: rootmark serve --http did not start" >&2
  exit 1
fi
token="$(tr -d '[:space:]' <"$hv/.rootmark/server.token")"
# req <method> <path> <body> [token] → "<status> <compact JSON body>"
req() {
  local t="${4-$token}" st
  st="$(curl -sS -o "$work/body" -w '%{http_code}' -X "$1" \
    ${t:+-H "Authorization: Bearer $t"} -H 'X-Consumer: contract-http' \
    -H 'Content-Type: application/json' --data-binary "$3" "http://127.0.0.1:$port$2")"
  echo "$st $(jq -c . "$work/body" 2>/dev/null || echo null)"
}
endpoints="[]"
add_endpoint() { # <method> <path> <status> <json>
  local ok=false
  if [ "$3" = 200 ] && echo "$4" | jq -e 'type == "object" and (has("error") | not)' >/dev/null; then ok=true; fi
  $ok || missing+=("http-endpoint:$1 $2")
  endpoints="$(echo "$endpoints" | jq -c --arg m "$1" --arg p "$2" --argjson st "$3" --argjson ok "$ok" \
    '. + [{endpoint: ($m + " " + $p), status: $st, ok: $ok}]')"
}
lineage="$(req POST /v1/search '{"query":"espresso"}' | cut -d' ' -f2- | jq -r '.hits[0].lineage_ids[0] // empty')"
proposed="$(req POST /v1/propose '{"doc_path":"agent/p.md","content":"# P\n\nAgent text.","reason":"contract-http"}' | cut -d' ' -f2- | jq -r '.blocks[-1].lineage_id // empty')"
tool_args() { # the §2 tool → a valid body
  case "$1" in
    status | checkpoints) echo '{}' ;;
    search) echo '{"query":"espresso"}' ;;
    read_block) echo "{\"doc_path\":\"a.md\",\"lineage_id\":\"$lineage\"}" ;;
    history) echo "{\"lineage_id\":\"$lineage\"}" ;;
    blame) echo '{"doc_path":"a.md"}' ;;
    propose) echo '{"doc_path":"agent/q.md","content":"# Q\n\nMore.","reason":"contract-http"}' ;;
    promote) echo "{\"lineage_id\":\"$proposed\",\"reason\":\"contract-http\"}" ;;
    checkpoint) echo '{"name":"contract-http"}' ;;
    trace) echo '{"consumer":"contract-http"}' ;;
    *) echo "" ;;
  esac
}
while read -r t; do
  [ -n "$t" ] || continue
  body="$(tool_args "$t")"
  if [ -z "$body" ]; then missing+=("http-endpoint:POST /v1/$t (no request in this script)"); continue; fi
  r="$(req POST "/v1/$t" "$body")"
  add_endpoint POST "/v1/$t" "${r%% *}" "${r#* }"
done <<<"$contract_tools"
r="$(req GET /v1/status '')"
add_endpoint GET /v1/status "${r%% *}" "${r#* }"
# §2.6 (P2-0): HTTP-only endpoints, never MCP tools
http_only_json="[]"
add_http_only() { # <name> <ok> <detail>
  $2 || missing+=("http-only:$1 ($3)")
  http_only_json="$(echo "$http_only_json" | jq -c --arg n "$1" --argjson ok "$2" --arg d "$3" '. + [{endpoint: $n, ok: $ok, detail: $d}]')"
}
[ -n "$http_only" ] || missing+=("contract:§2.6 (no HTTP-only endpoint rows)")
for n in $http_only; do
  if printf '%s\n' "$server_tools" | grep -qx "$n"; then add_http_only "$n" false "listed by MCP tools/list"; fi
done
if printf '%s\n' "$http_only" | grep -qx attest_session && printf '%s\n' "$http_only" | grep -qx attest; then
  r="$(req POST /v1/attest_session '{}')"
  sid="$(echo "${r#* }" | jq -r '.session_id // empty')"; key="$(echo "${r#* }" | jq -r '.key // empty')"
  ok=false
  [ "${r%% *}" = 200 ] && [ -n "$sid" ] && [[ "$key" =~ ^[0-9a-f]{64}$ ]] \
    && echo "${r#* }" | jq -e '(.ttl_s | type == "number") and (.stage | type == "number")' >/dev/null && ok=true
  add_http_only attest_session "$ok" "status ${r%% *}"
  h=00000000000000000000000000000000
  # a server without a session (an older build) gives no key: the MAC is
  # then made with a zero key, so `attest` is still asked and recorded
  [[ "$key" =~ ^[0-9a-f]{64}$ ]] || key="$(printf '0%.0s' $(seq 1 64))"
  mac="$(printf 'rootmark-attest/1\na.md\n%s\n' "$h" | openssl dgst -sha256 -mac HMAC -macopt "hexkey:$key" -r | cut -d' ' -f1)"
  r="$(req POST /v1/attest "{\"doc_path\":\"a.md\",\"content_hash\":\"$h\",\"session_id\":\"$sid\",\"mac\":\"$mac\"}")"
  ok=false
  [ "${r%% *}" = 200 ] && echo "${r#* }" | jq -e '(has("error") | not) and has("attestation_id") and has("doc_path") and has("content_hash") and has("at_seq") and has("recorded") and has("raised")' >/dev/null && ok=true
  add_http_only attest "$ok" "status ${r%% *}"
  r="$(req POST /v1/attest "{\"doc_path\":\"a.md\",\"content_hash\":\"$h\",\"session_id\":\"$sid\",\"mac\":\"$(printf '0%.0s' $(seq 1 64))\"}")"
  ok=false; [ "${r%% *}" = 403 ] && [ "$(echo "${r#* }" | jq -r '.error.code // empty')" = denied ] && ok=true
  add_http_only "attest (wrong mac → 403 denied)" "$ok" "status ${r%% *}"
else
  missing+=("contract:§2.6 attest_session/attest rows")
fi
# §3: each code from a real request (index_stale and internal last: they
# age, then damage the index)
code_req() {
  case "$1" in
    invalid_arguments) req POST /v1/search '{"query":"x","mode":"bogus"}' ;;
    empty_query) req POST /v1/search '{"query":" "}' ;;
    path_outside_vault) req POST /v1/propose '{"doc_path":"../x.md","content":"x","reason":"r"}' ;;
    denied) req POST /v1/propose '{"doc_path":"a.md","mode":"replace_section","heading":"A","content":"x","reason":"r"}' ;;
    not_found) req POST /v1/history '{"lineage_id":"0123"}' ;;
    exists) req POST /v1/propose '{"doc_path":"a.md","content":"x","reason":"r"}' ;;
    too_large) req POST /v1/propose "{\"doc_path\":\"big.md\",\"content\":\"$(printf 'x%.0s' $(seq 1 100))\",\"reason\":\"r\"}" ;;
    heading_not_found) req POST /v1/propose '{"doc_path":"a.md","mode":"replace_section","heading":"Nope","content":"x","reason":"r"}' ;;
    policy_violation) req POST /v1/propose '{"doc_path":"a.md","mode":"append","content":"x","reason":"r"}' ;;
    migration_required)
      m="$hv/.rootmark/index/manifest.json"
      # the current index, for `index_stale`, `internal` and the status step below
      rm -rf "$work/index.bak" && cp -R "$hv/.rootmark/index" "$work/index.bak"
      jq -c '.format = "mdkernel-index/2"' "$m" >"$work/m" && cp "$work/m" "$m"
      req POST /v1/search '{"query":"espresso"}' ;;
    index_stale)
      # a format this rootmark does not know (P2-A/F2)
      m="$hv/.rootmark/index/manifest.json"
      rm -rf "$hv/.rootmark/index" && cp -R "$work/index.bak" "$hv/.rootmark/index"
      jq -c '.format = "rootmark-index/99"' "$m" >"$work/m" && cp "$work/m" "$m"
      req POST /v1/search '{"query":"espresso"}' ;;
    internal)
      # an index file that does not match its manifest (blake3)
      rm -rf "$hv/.rootmark/index" && cp -R "$work/index.bak" "$hv/.rootmark/index"
      printf '\n' >>"$hv/.rootmark/index/records.jsonl"
      req POST /v1/search '{"query":"espresso"}' ;;
    *) echo "" ;;
  esac
}
http_codes="[]"
order="$(echo "$contract_codes" | awk '$1 != "index_stale" && $1 != "internal" && $1 != "migration_required"'; echo "$contract_codes" | awk '$1 == "migration_required"'; echo "$contract_codes" | awk '$1 == "index_stale"'; echo "$contract_codes" | awk '$1 == "internal"')"
while read -r c st; do
  [ -n "$c" ] || continue
  r="$(code_req "$c")"
  if [ -z "$r" ]; then missing+=("http-code:$c"); continue; fi
  got="${r%% *}"; gcode="$(echo "${r#* }" | jq -r '.error.code // empty')"
  ok=false; [ "$got" = "$st" ] && [ "$gcode" = "$c" ] && ok=true
  $ok || missing+=("http-code:$c (status $got, code ${gcode:-none}; contract $st)")
  http_codes="$(echo "$http_codes" | jq -c --arg c "$c" --argjson want "$st" --argjson got "$got" --arg gc "$gcode" --argjson ok "$ok" \
    '. + [{code: $c, contract_status: $want, status: $got, error_code: $gc, ok: $ok}]')"
done <<<"$order"
# status in every index state (K-E1.2, E1-8): the `index {…}` fields of the
# contract's §2 `status` row
index_fields="$(sed -n '/^## 2\. /,/^## 3\. /p' "$contract" | grep -E '^\| `status` \|' | grep -oE 'index \{[^}]*\}' | head -1 \
  | sed -E 's/^index \{//; s/\}$//' | tr ',' '\n' | tr -d ' ' | grep -v '^$' | jq -R . | jq -sc .)"
[ "$index_fields" != "[]" ] || missing+=("contract:status.index fields (no \`index {…}\` in the §2 status row)")
# (P2-0/Z0; P1-E.1 verification, recommendation 2) every field of the §2
# `status` row: each top-level name and each name listed in its `{…}` group
# (`…` is not a field) is a path the response must hold (`null` values
# allowed: `last_checkpoint`, `embedder`)
status_paths="$(sed -n '/^## 2\. /,/^## 3\. /p' "$contract" | grep -E '^\| `status` \|' | head -1 | awk -F'|' '{print $5}' | tr -d '`' \
  | jq -Rc '[scan("([a-z_]+)( \\{([^}]*)\\})?") | .[0] as $n
      | if .[2] == null then [[$n]]
        else [.[2] | split(",")[] | gsub("^\\s+|\\s+$"; "") | select(. != "" and . != "…") | [$n, .]] end] | add // []')"
[ "$(echo "$status_paths" | jq 'map(select(length == 1)) | length')" -gt 0 ] || missing+=("contract:status fields (no field list in the §2 status row)")
status_index="[]"
index_state_check() { # <state>: status GET/POST 200 with that state; search 503 unless ok
  local want="$1" g p sr ok=true sst scode absent
  g="$(req GET /v1/status '')"; p="$(req POST /v1/status '{}')"
  for r in "$g" "$p"; do
    [ "${r%% *}" = 200 ] || ok=false
    echo "${r#* }" | jq -e --arg s "$want" \
      '(has("error") | not) and (.index | type == "object") and .index.state == $s' >/dev/null 2>&1 || ok=false
    echo "${r#* }" | jq -e --argjson f "$index_fields" '.index as $i | all($f[]; . as $k | $i | has($k))' >/dev/null 2>&1 || ok=false
    absent="$(echo "${r#* }" | jq -c --argjson ps "$status_paths" \
      '. as $r | [$ps[] | select(. as $p | ($r | getpath($p[:-1]) | type == "object" and has($p[-1])) | not) | join(".")]' 2>/dev/null || echo '["?"]')"
    if [ "$absent" != "[]" ]; then ok=false; missing+=("http-status-fields:$want $absent"); fi
  done
  sr="$(req POST /v1/search '{"query":"espresso"}')"
  sst="${sr%% *}"; scode="$(echo "${sr#* }" | jq -r '.error.code // empty')"
  if [ "$want" = ok ]; then
    [ "$sst" = 200 ] || ok=false
  elif [ "$want" = stale ]; then
    # an older rootmark's index (P2-A/F2)
    { [ "$sst" = 503 ] && [ "$scode" = migration_required ]; } || ok=false
  else
    { [ "$sst" = 503 ] && [ "$scode" = index_stale ]; } || ok=false
  fi
  $ok || missing+=("http-status-index:$want")
  status_index="$(echo "$status_index" | jq -c --arg s "$want" --argjson gs "${g%% *}" --argjson ps "${p%% *}" \
    --argjson gi "$(echo "${g#* }" | jq -c '.index // null')" --argjson ss "$sst" --arg sc "$scode" --argjson ok "$ok" \
    '. + [{state: $s, status_get: $gs, status_post: $ps, index: $gi, search_status: $ss, search_code: $sc, ok: $ok}]')"
}
if [ -d "$work/index.bak" ]; then
  rm -rf "$hv/.rootmark/index" && cp -R "$work/index.bak" "$hv/.rootmark/index"
  index_state_check ok
  m="$hv/.rootmark/index/manifest.json"
  jq -c '.format = "mdkernel-index/2"' "$m" >"$work/m" && cp "$work/m" "$m"
  index_state_check stale
  printf '{ not json' >"$m"
  index_state_check damaged
  rm -rf "$hv/.rootmark/index"
  index_state_check missing
else
  missing+=("http-status-index:no index to vary (index_stale step did not run)")
fi
# §1/§3 transport rules
transport="[]"
add_transport() { # <case> <want status> <want code> <response>
  local got="${4%% *}" gc ok=false
  gc="$(echo "${4#* }" | jq -r '.error.code // empty')"
  [ "$got" = "$2" ] && [ "$gc" = "$3" ] && ok=true
  $ok || missing+=("http-transport:$1")
  transport="$(echo "$transport" | jq -c --arg n "$1" --argjson st "$got" --arg gc "$gc" --argjson ok "$ok" \
    '. + [{case: $n, status: $st, error_code: $gc, ok: $ok}]')"
}
add_transport "no token" 401 denied "$(req POST /v1/status '{}' '')"
add_transport "unknown path" 404 not_found "$(req POST /v1/nope '{}')"
add_transport "GET of a tool" 405 invalid_arguments "$(req GET /v1/search '')"
kill "$serve_pid" 2>/dev/null || true
wait "$serve_pid" 2>/dev/null || true
http_pass="$(jq -n --argjson e "$endpoints" --argjson c "$http_codes" --argjson t "$transport" --argjson si "$status_index" \
  '($e | length) > 0 and ($c | length) > 0 and ($si | length) == 4 and all($e[]; .ok) and all($c[]; .ok) and all($t[]; .ok) and all($si[]; .ok)')"
http_json="$(jq -n --argjson e "$endpoints" --argjson c "$http_codes" --argjson t "$transport" --argjson si "$status_index" \
  --argjson f "$index_fields" --argjson sf "$status_paths" --argjson p "$http_pass" \
  --argjson ho "$http_only_json" \
  '{endpoints: $e, codes: $c, transport: $t, status_index: $si, status_index_fields: $f, status_fields: ($sf | map(join("."))), http_only: $ho, pass: $p}')"

tools_equal=false; [ "$contract_tools" = "$server_tools" ] && tools_equal=true
codes_equal=false; [ "$contract_codes" = "$server_codes" ] && codes_equal=true
pass=false
if $tools_equal && $codes_equal && [ "$http_pass" = true ] && [ "${#missing[@]}" -eq 0 ] && [ -n "$contract_tools" ] && [ -n "$contract_codes" ]; then pass=true; fi
result="$(jq -n \
  --arg ct "$contract_tools" --arg st "$server_tools" \
  --arg cc "$contract_codes" --arg sc "$server_codes" \
  --argjson te "$tools_equal" --argjson ce "$codes_equal" --argjson pass "$pass" --argjson http "$http_json" \
  --args '{
    tools: {contract: ($ct | split("\n")), server: ($st | split("\n")), equal: $te},
    codes: {contract: ($cc | split("\n")), server: ($sc | split("\n")), equal: $ce},
    http: $http,
    missing: $ARGS.positional, pass: $pass}' "${missing[@]+"${missing[@]}"}")"
[ -z "$out" ] || echo "$result" >"$out"
echo "$result"
if [ "$pass" != true ]; then
  echo "contract_check: FAILED" >&2
  exit 1
fi
echo "contract_check: OK ($(echo "$contract_tools" | wc -l | tr -d ' ') tools, $(echo "$contract_codes" | wc -l | tr -d ' ') codes, each matched by a test and by a real HTTP call)" >&2
