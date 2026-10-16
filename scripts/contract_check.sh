#!/usr/bin/env bash
# D1-13 (GOAL P1-D.1; TASKS D1.5): docs/CLIENT_CONTRACT.md against the server
# and the tests.
#   1. the tools of the contract (§2 table, first column) are exactly the
#      tools a live `mdk serve --mcp stdio` lists (`tools/list`);
#   2. the error codes of the contract (§3 table) are exactly
#      `server/src/error.rs` `Code::as_str`, with the same HTTP status
#      (`Code::http_status`);
#   3. every tool and every code is matched by a test of
#      server/tests/contract.rs: the tool by a real call
#      (`call_tool(&k, "<tool>", …)`), the code by a real call expecting it
#      (`expect_code(&k, …, Code::<Variant>);`); comments do not count.
# Prints and writes (<out.json>, optional) {tools, codes, missing, pass};
# exit 1 if anything is missing or differs.
#
#   scripts/contract_check.sh [<out.json>]   (MDK_CONTRACT: another contract file)
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-}"
contract="${MDK_CONTRACT:-$here/docs/CLIENT_CONTRACT.md}"
work="$(mktemp -d -t mdk-contract.XXXXXX)"
trap 'rm -rf "$work"' EXIT

(cd "$here" && cargo build -q --release -p mdkernel-server --bin mdk --example mcp_pipeline)
mdk="$here/target/release/mdk"
pipe="$here/target/release/examples/mcp_pipeline"

# 1. tools: contract §2 rows `| \`name\` | …` between "## 2." and "## 3."
contract_tools="$(sed -n '/^## 2\. /,/^## 3\. /p' "$contract" | grep -E '^\| `[a-z_]+` \|' | sed -E 's/^\| `([a-z_]+)`.*/\1/' | sort)"
mkdir -p "$work/vault"
printf '# A\n\nText.\n' >"$work/vault/a.md"
"$mdk" ingest-dir "$work/vault" "$work/vault" >/dev/null
cat >"$work/list.jsonl" <<'J'
{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"contract-check","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
J
# pipelined: the server completes the initialize handshake before it reads
# tools/list
"$pipe" "$work/list.jsonl" "$work/list.out" -- "$mdk" serve --vault "$work/vault" --mcp stdio 2>"$work/serve.err"
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

tools_equal=false; [ "$contract_tools" = "$server_tools" ] && tools_equal=true
codes_equal=false; [ "$contract_codes" = "$server_codes" ] && codes_equal=true
pass=false
if $tools_equal && $codes_equal && [ "${#missing[@]}" -eq 0 ] && [ -n "$contract_tools" ] && [ -n "$contract_codes" ]; then pass=true; fi
result="$(jq -n \
  --arg ct "$contract_tools" --arg st "$server_tools" \
  --arg cc "$contract_codes" --arg sc "$server_codes" \
  --argjson te "$tools_equal" --argjson ce "$codes_equal" --argjson pass "$pass" \
  --args '{
    tools: {contract: ($ct | split("\n")), server: ($st | split("\n")), equal: $te},
    codes: {contract: ($cc | split("\n")), server: ($sc | split("\n")), equal: $ce},
    missing: $ARGS.positional, pass: $pass}' "${missing[@]+"${missing[@]}"}")"
[ -z "$out" ] || echo "$result" >"$out"
echo "$result"
if [ "$pass" != true ]; then
  echo "contract_check: FAILED" >&2
  exit 1
fi
echo "contract_check: OK ($(echo "$contract_tools" | wc -l | tr -d ' ') tools, $(echo "$contract_codes" | wc -l | tr -d ' ') codes, each matched by a test)" >&2
