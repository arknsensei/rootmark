#!/usr/bin/env bash
# MCP end-to-end over stdio JSON-RPC (GOAL P1-D D-2, D-8; TASKS D1, D3–D5).
#
#   scripts/mcp_e2e.sh <out-dir>
#
# Builds `mdk` and the lock-step driver (`server/examples/mcp_client.rs`),
# then runs the same session twice, in <out-dir>/run1 and <out-dir>/run2:
#   1. vault: spec/fixtures/mini_vault (`mdk ingest-dir`), checkpoint `t0`,
#      one agent document (`mdk propose`), one CLI read (one trace),
#      checkpoint `t1` — all through the CLI, before the server starts;
#   2. `mdk serve --vault vault --mcp stdio` (cwd = the run directory, so
#      the vault path is the same relative path in both runs; it writes the
#      default policy recipe and builds the index at start), driven by
#      `session.jsonl`: initialize (clientInfo mcp-e2e/1.0), tools/list,
#      then for every tool its calls, valid and deliberately invalid;
#   3. per tool, every check on its responses (jq) must hold; `search` is
#      compared byte for byte with its CLI twin (`mdk search --consumer
#      mcp-e2e/1.0` on a copy of the vault taken before the session);
#      D-5: 100 `search` + 100 `read_block` calls → exactly 200 more trace
#      lines, consumer `mcp-e2e/1.0` (the clientInfo of the handshake).
# Then the two runs are compared: transcripts byte for byte (they carry no
# time stamp and no token), and the vault's events.jsonl and traces.jsonl
# byte for byte. Result: <out-dir>/mcp_e2e.json; exit 1 if any tool fails
# or the runs differ.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: scripts/mcp_e2e.sh <out-dir>}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
cd "$root"
cargo build -q --release -p mdkernel-server --bin mdk --example mcp_client
mdk="$root/target/release/mdk"
client="$root/target/release/examples/mcp_client"
TOOLS="status search read_block history blame checkpoints trace"

prepare() { # prepare <run-dir>: the vault, through the CLI
  local d="$1"
  mkdir "$d"
  (
    cd "$d"
    "$mdk" ingest-dir vault "$root/spec/fixtures/mini_vault" >prep_ingest.json
    "$mdk" checkpoint create vault --name t0 >prep_t0.json
    printf '# Agent notes\n\nAn agent suggests a finer espresso grind for light roasts.\n' >agent_note.md
    "$mdk" propose vault agent/notes.md agent_note.md --source mcp-e2e-prep >prep_propose.json
    rm agent_note.md
    lineage="$("$mdk" blame vault espresso.md | jq -r '.blocks[0].lineage_id')"
    "$mdk" read vault --at t0 --consumer cli-prep "$lineage" >prep_read.json
    "$mdk" checkpoint create vault --name t1 >prep_t1.json
    "$mdk" blame vault agent/notes.md | jq -r '.blocks[-1].lineage_id' >agent_lineage.txt
    "$mdk" index build vault >prep_index.json
    echo "$lineage" >lineage.txt
    # the same state for the CLI twin of the first MCP search (byte equality)
    cp -R vault vault_cli
  )
}

session() { # session <lineage> <agent-lineage>: the JSON-RPC messages, one per line
  local l="$1" al="$2" id=0
  req() { id=$((id + 1)); printf '{"jsonrpc":"2.0","id":%d,"method":"%s","params":%s}\n' "$id" "$1" "$2"; }
  call() { req tools/call "{\"name\":\"$1\",\"arguments\":$2}"; }
  req initialize '{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"mcp-e2e","version":"1.0"}}'
  printf '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
  req tools/list '{}'
  call status '{}'                                           # 3
  call status '{"verbose":true}'                             # 4 unknown argument
  call history "{\"lineage_id\":\"$l\"}"                     # 5
  call history '{"lineage_id":"00000000000000000000000000000000"}' # 6 unknown lineage
  call blame '{"doc_path":"espresso.md"}'                    # 7
  call blame '{"doc_path":"agent/notes.md","at":"t1"}'       # 8
  call blame '{"doc_path":"agent/notes.md","at":"t0"}'       # 9 not yet at t0
  call blame '{}'                                            # 10 missing doc_path
  call checkpoints '{}'                                      # 11
  call trace '{}'                                            # 12
  call trace '{"consumer":"cli-prep","since_seq":0}'         # 13
  call trace '{"since_seq":1}'                               # 14
  call trace '{"since_seq":-1}'                              # 15 out of range
  # D3: the first trace-recording call (at_seq 1, as its CLI twin on vault_cli)
  call search '{"query":"espresso grind"}'                   # 16
  call search '{"query":"espresso grind","include_proposal":true}' # 17
  call search '{"query":" \t "}'                            # 18 whitespace only
  call search '{"query":"espresso","mode":"fuzzy"}'          # 19 unknown mode
  call search '{"query":"espresso","k":0}'                   # 20 k out of range
  call read_block "{\"doc_path\":\"espresso.md\",\"lineage_id\":\"$l\"}"          # 21
  call read_block "{\"doc_path\":\"filter.md\",\"lineage_id\":\"$l\"}"            # 22 not in that doc
  call read_block "{\"doc_path\":\"agent/notes.md\",\"lineage_id\":\"$al\",\"at\":\"t0\"}" # 23 not yet at t0
  call read_block "{\"doc_path\":\"agent/notes.md\",\"lineage_id\":\"$al\"}"      # 24 proposal
  call trace '{"consumer":"mcp-e2e/1.0"}'                    # 25
  # D-5: 100 searches and 100 reads, one trace each
  local i
  for i in $(seq 1 100); do
    call search "{\"query\":\"espresso load $i\",\"k\":3}"
    call read_block "{\"doc_path\":\"espresso.md\",\"lineage_id\":\"$l\"}"
  done                                                        # 26..225
  call trace '{"consumer":"mcp-e2e/1.0","since_seq":5}'      # 226
}

run() { # run <run-dir>
  local d="$1"
  prepare "$d"
  session "$(cat "$d/lineage.txt")" "$(cat "$d/agent_lineage.txt")" >"$d/session.jsonl"
  (cd "$d" && "$client" session.jsonl transcript.jsonl -- "$mdk" serve --vault vault --mcp stdio 2>server.err)
  # the CLI twin of call 16, on the copy taken before the session
  (cd "$d" && "$mdk" search vault_cli "espresso grind" --consumer mcp-e2e/1.0 >cli_search.json)
}

run "$out/run1"
run "$out/run2"

t="$out/run1/transcript.jsonl"
r() { jq -c --argjson id "$1" 'select(.request.id == $id) | .response' "$t"; }
sc() { r "$1" | jq -c '.result.structuredContent'; }
declare -a results=()
n_ok=0 n=0
check() { # check <tool> <label> <jq-boolean over the response> <id>
  local tool="$1" label="$2" expr="$3" id="$4"
  local v
  v="$(r "$id" | jq -e "$expr" 2>/dev/null || true)"
  if [ "$v" = "true" ]; then
    echo "mcp_e2e: ok   [$tool] $label"
    results+=("{\"tool\":\"$tool\",\"check\":\"$label\",\"pass\":true}")
  else
    echo "mcp_e2e: FAIL [$tool] $label: $(r "$id" | cut -c1-300)" >&2
    results+=("{\"tool\":\"$tool\",\"check\":\"$label\",\"pass\":false}")
  fi
}
lin="$(cat "$out/run1/lineage.txt")"
ok_result='(.result.isError == false) and (.result.structuredContent | type == "object")'
tool_err='.result.isError == true and (.result.content[0].text | startswith("error: "))'

check handshake "initialize names server mdk, tools capability" \
  '.result.serverInfo.name == "mdk" and (.result.capabilities.tools | type == "object")' 1
check handshake "tools/list has every tool of this round" \
  "[.result.tools[].name] == ($(printf '%s\n' $TOOLS | jq -R . | jq -sc .))" 2

check status "valid call" "$ok_result" 3
check status "versions, level, relative vault path" \
  '.result.structuredContent | .spec_version == "1.0.2" and .level_claimed == 3 and .vault == "vault" and (.mdk_version | type == "string")' 3
check status "last checkpoint t1, default policy human_only, index built at start" \
  '.result.structuredContent | .last_checkpoint.name == "t1" and .policy.promotion == "human_only" and .index == "present" and (.embedder | startswith("minilm-l6-v2/")) and .server_policy.server.max_propose_bytes == 1048576' 3
check status "unknown argument is an error" "$tool_err" 4

check history "valid call" "$ok_result" 5
check history "lineage, layer, events" \
  ".result.structuredContent | .lineage_id == \"$lin\" and .layer == \"canonical\" and (.events | length) >= 1 and all(.events[]; .lineage_ids | index(\"$lin\"))" 5
check history "unknown lineage is an error" "$tool_err" 6

check blame "valid call (head)" "$ok_result" 7
check blame "every block: lineage, version, layer, author" \
  '.result.structuredContent | .doc_path == "espresso.md" and (.blocks | length) > 0 and all(.blocks[]; (.lineage_id | length) == 32 and (.version_id | length) == 32 and .layer == "canonical" and .author_kind == "Human")' 7
check blame "agent document as of t1: proposal blocks" \
  '.result.structuredContent | .at == "t1" and (.blocks | length) > 0 and all(.blocks[]; .layer == "proposal" and .author_kind == "Agent")' 8
check blame "document absent at t0 is an error" "$tool_err" 9
check blame "missing doc_path is an error" "$tool_err" 10

check checkpoints "valid call" "$ok_result" 11
check checkpoints "t0 then t1, with event offsets" \
  '[.result.structuredContent.checkpoints[] | .name] == ["t0", "t1"] and (.result.structuredContent.checkpoints | .[0].event_seq < .[1].event_seq)' 11

check trace "valid call" "$ok_result" 12
check trace "the CLI read's trace" \
  ".result.structuredContent.traces | length == 1 and .[0].consumer == \"cli-prep\" and .[0].at_seq == 0 and .[0].returned[0].lineage_id == \"$lin\"" 12
check trace "consumer filter" '.result.structuredContent.traces | length == 1' 13
check trace "since_seq past the end: empty" '.result.structuredContent.traces == []' 14
check trace "negative since_seq is an error" "$tool_err" 15

alin="$(cat "$out/run1/agent_lineage.txt")"
check search "valid call (hybrid default)" "$ok_result" 16
check search "canonical hits only, agent document absent, trace by mcp-e2e/1.0" \
  '.result.structuredContent | (.hits | length) > 0 and all(.hits[]; .layer == "canonical" and .doc_path != "agent/notes.md") and .mode == "hybrid" and .include_proposal == false and (.trace_id | length) == 32' 16
check search "byte-equal to the CLI twin (mdk search --consumer mcp-e2e/1.0, same state)" \
  ".result.structuredContent == $(jq -c . "$out/run1/cli_search.json")" 16
check search "include_proposal: canonical prefix unchanged, agent proposal labelled after it" \
  "(.result.structuredContent.hits | map(select(.layer == \"canonical\"))) == $(r 16 | jq -c .result.structuredContent.hits) and any(.result.structuredContent.hits[]; .doc_path == \"agent/notes.md\" and .layer == \"proposal\") and (.result.structuredContent.hits | (map(.layer) | index(\"proposal\")) >= ([.[] | select(.layer == \"canonical\")] | length))" 17
check search "whitespace-only query is an error" "$tool_err and (.result.content[0].text | test(\"empty query\"))" 18
check search "unknown mode is an error" "$tool_err" 19
check search "k = 0 is an error" "$tool_err" 20
check read_block "valid call" "$ok_result" 21
check read_block "block version, canonical, human provenance, trace by mcp-e2e/1.0" \
  ".result.structuredContent | .block.doc_path == \"espresso.md\" and .block.block.lineage_id == \"$lin\" and .block.block.layer == \"canonical\" and .block.block.provenance.author_kind == \"Human\" and (.block.block.version_id | length) == 32 and .trace.consumer == \"mcp-e2e/1.0\" and .trace.returned[0].lineage_id == \"$lin\"" 21
check read_block "lineage of another document is an error" "$tool_err" 22
check read_block "document absent at t0 is an error" "$tool_err" 23
check read_block "agent block: proposal, agent provenance" \
  ".result.structuredContent.block.block | .lineage_id == \"$alin\" and .layer == \"proposal\" and .provenance.author_kind == \"Agent\"" 24
check trace "consumer filter: the 4 MCP traces, at_seq 1..4, failed calls recorded nothing" \
  '[.result.structuredContent.traces[] | .at_seq] == [1, 2, 3, 4] and all(.result.structuredContent.traces[]; .consumer == "mcp-e2e/1.0")' 25
check trace "D-5: 200 calls → 200 traces, consumer mcp-e2e/1.0" \
  '(.result.structuredContent.traces | length) == 200 and all(.result.structuredContent.traces[]; .consumer == "mcp-e2e/1.0") and ([.result.structuredContent.traces[] | .at_seq] == [range(5; 205)])' 226
ok_load=true
for i in $(seq 26 225); do
  r "$i" | jq -e '.result.isError == false' >/dev/null || ok_load=false
done
tl="$out/run1/vault/.mdkernel/traces.jsonl"
n_lines="$(wc -l <"$tl" | tr -d ' ')"
n_mcp="$(jq -r 'select(.consumer == "mcp-e2e/1.0") | .trace_id' "$tl" | wc -l | tr -d ' ')"
d5=false
if [ "$ok_load" = true ] && [ "$n_lines" = 205 ] && [ "$n_mcp" = 204 ]; then d5=true; fi
results+=("{\"tool\":\"trace\",\"check\":\"D-5 traces.jsonl: 205 lines, 204 by mcp-e2e/1.0 (4 + 200), every load call ok\",\"pass\":$d5}")
echo "mcp_e2e: $( [ "$d5" = true ] && echo 'ok  ' || echo FAIL) [trace] D-5 traces.jsonl: $n_lines lines, $n_mcp by mcp-e2e/1.0"

# pass per tool: every check of that tool holds
per_tool="$(printf '%s\n' "${results[@]}" | jq -sc 'group_by(.tool) | map({key: .[0].tool, value: {checks: length, passed: (map(select(.pass)) | length)}}) | from_entries')"

# determinism: run1 ↔ run2
same=true
for f in transcript.jsonl vault/.mdkernel/events.jsonl vault/.mdkernel/traces.jsonl; do
  if ! cmp -s "$out/run1/$f" "$out/run2/$f"; then
    echo "mcp_e2e: runs differ: $f" >&2
    same=false
  fi
done

tools_json="$(printf '%s\n' $TOOLS | jq -R . | jq -sc .)"
jq -n --argjson per "$per_tool" --argjson tools "$tools_json" --argjson same "$same" \
  --argjson checks "$(printf '%s\n' "${results[@]}" | jq -sc .)" '
  {tools: $tools,
   per_tool: $per,
   tools_passed: ([$tools[] | select($per[.] != null and $per[.].checks == $per[.].passed)] | length),
   tools_total: ($tools | length),
   handshake_pass: ($per.handshake.checks == $per.handshake.passed),
   runs_equal: $same,
   d5: {calls: 200, traces_file_lines: '"$n_lines"', mcp_consumer_traces: '"$n_mcp"', pass: '"$d5"'},
   checks: $checks}
  | .pass = (.tools_passed == .tools_total and .handshake_pass and .runs_equal)' >"$out/mcp_e2e.json"
jq -r '"mcp_e2e: tools \(.tools_passed)/\(.tools_total), handshake \(.handshake_pass), runs equal \(.runs_equal)"' "$out/mcp_e2e.json"
jq -e .pass "$out/mcp_e2e.json" >/dev/null
