#!/usr/bin/env bash
# The agent side of the first product scene (NORTH_STAR; GOAL P1-D, TASKS D6),
# over MCP stdio, on a clean vault.
#
#   scripts/scene_agent.sh [<out-dir>]      (default reports/rounds/P1-D)
#
# Vault: a copy of spec/fixtures/mini_vault ingested in place, in a fresh
# temporary directory; `mdk serve --vault vault --mcp stdio` (cwd = that
# directory) writes the default policy (human_only) and builds the index.
# Steps, as an MCP client `scene-agent/1.0` (server/examples/mcp_client.rs):
#   1. status
#   2. propose      agent/scene.md (new document, plain Markdown)
#   3. search       default: the proposal is not found
#   4. search       include_proposal: found, labelled proposal
#   5. promote      denied (policy human_only), no event
#   6. [CLI]        mdk promote --actor human:editor, every block of the document
#   7. search       default: found, canonical
#   8. trace        the scene's consumption traces (consumer scene-agent/1.0)
#   9. checkpoint   of the current state: the one the search of step 7 ran
#                   at (a state has one checkpoint; spec §8 same state → same id)
# The vault is kept in <out-dir>/scene_vault only if MDK_SCENE_KEEP=1.
# <out-dir>/scene.json: one record per step (what it showed, pass) and
# `pass`; it holds no time stamp, no token and no absolute path, so two runs
# are byte-equal. Exit 1 unless every step passes.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-$root/reports/rounds/P1-D}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
cd "$root"
cargo build -q --release -p mdkernel-server --bin mdk --example mcp_client
mdk="$root/target/release/mdk"
client="$root/target/release/examples/mcp_client"
work="$(mktemp -d -t mdk-scene.XXXXXX)"
trap 'rm -rf "$work"' EXIT
cd "$work"
mkdir vault
cp "$root"/spec/fixtures/mini_vault/*.md vault/
"$mdk" ingest-dir vault vault >/dev/null

INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"scene-agent","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}'
DOC="agent/scene.md"
TEXT='# Grind notes\n\nAn agent recommends a coarser grind for the kalimba roast.\n'
call() { printf '{"jsonrpc":"2.0","id":%d,"method":"tools/call","params":{"name":"%s","arguments":%s}}\n' "$1" "$2" "$3"; }
serve() { # serve <session> <transcript>
  "$client" "$1" "$2" -- "$mdk" serve --vault vault --mcp stdio 2>>serve.err
}
res() { jq -c --argjson id "$2" 'select(.request.id == $id) | .response.result' "$1"; }

# session A: status, propose, the two searches
{
  echo "$INIT"
  call 2 status '{}'
  call 3 propose "{\"doc_path\":\"$DOC\",\"content\":\"$TEXT\",\"reason\":\"product scene\"}"
  call 4 search '{"query":"kalimba","mode":"lexical"}'
  call 5 search '{"query":"kalimba","mode":"lexical","include_proposal":true}'
} >a.jsonl
serve a.jsonl ta.jsonl
lineages="$(res ta.jsonl 3 | jq -r '.structuredContent.blocks[].lineage_id')"
para="$(res ta.jsonl 3 | jq -r '.structuredContent.blocks[-1].lineage_id')"

# session A2: the agent asks for the promotion
{
  echo "$INIT"
  call 2 promote "{\"lineage_id\":\"$para\",\"reason\":\"the agent asks for approval\"}"
} >a2.jsonl
n_events_before="$(wc -l <vault/.mdkernel/events.jsonl | tr -d ' ')"
serve a2.jsonl ta2.jsonl
n_events_after="$(wc -l <vault/.mdkernel/events.jsonl | tr -d ' ')"

# the human approves through the CLI
: >cli.jsonl
for l in $lineages; do
  "$mdk" promote vault "$l" --actor human:editor --reason "approved in the scene" >>cli.jsonl
done

# session B: search, trace, checkpoint
{
  echo "$INIT"
  call 2 search '{"query":"kalimba","mode":"lexical"}'
  call 3 trace '{"consumer":"scene-agent/1.0"}'
  call 4 checkpoint '{}'
} >b.jsonl
serve b.jsonl tb.jsonl

plain=false
if [ "$(printf "$TEXT")" = "$(cat "vault/$DOC")" ] && [ "$(grep -cE 'lineage|version_id|mdkernel' "vault/$DOC" || true)" = 0 ]; then plain=true; fi

jq -n --slurpfile cli <(jq -c . cli.jsonl) \
  --argjson status "$(res ta.jsonl 2)" --argjson propose "$(res ta.jsonl 3)" \
  --argjson s_def "$(res ta.jsonl 4)" --argjson s_inc "$(res ta.jsonl 5)" \
  --argjson promote "$(res ta2.jsonl 2)" --argjson s_after "$(res tb.jsonl 2)" \
  --argjson trace "$(res tb.jsonl 3)" --argjson cp "$(res tb.jsonl 4)" \
  --arg doc "$DOC" --argjson plain "$plain" \
  --argjson ev_before "$n_events_before" --argjson ev_after "$n_events_after" '
  def hits_on($r): [$r.structuredContent.hits[] | select(.doc_path == $doc)];
  [
    {step: 1, tool: "status",
     shows: {spec_version: $status.structuredContent.spec_version, level_claimed: $status.structuredContent.level_claimed,
             policy: $status.structuredContent.policy.promotion, embedder: $status.structuredContent.embedder},
     pass: ($status.isError == false and $status.structuredContent.policy.promotion == "human_only")},
    {step: 2, tool: "propose",
     shows: {doc_path: $doc, plain_markdown: $plain,
             blocks: [$propose.structuredContent.blocks[] | {layer, author_kind, source}]},
     pass: ($propose.isError == false and $plain and all($propose.structuredContent.blocks[]; .layer == "proposal" and .author_kind == "Agent"))},
    {step: 3, tool: "search", args: "default",
     shows: {hits_on_doc: (hits_on($s_def) | length)},
     pass: ($s_def.isError == false and (hits_on($s_def) | length) == 0)},
    {step: 4, tool: "search", args: "include_proposal",
     shows: {hits_on_doc: [hits_on($s_inc)[] | {layer}]},
     pass: ((hits_on($s_inc) | length) > 0 and all(hits_on($s_inc)[]; .layer == "proposal"))},
    {step: 5, tool: "promote",
     shows: {decision: $promote.structuredContent.decision, policy: $promote.structuredContent.policy,
             events_written: ($ev_after - $ev_before)},
     pass: ($promote.isError == false and $promote.structuredContent.decision == "denied"
            and $promote.structuredContent.policy == "human_only" and $ev_after == $ev_before)},
    {step: 6, tool: "cli promote --actor human:editor",
     shows: [$cli[] | {kind: .event.kind, layer, actor: .event.meta.actor}],
     pass: (($cli | length) > 0 and all($cli[]; .event.kind == "Promoted" and .layer == "canonical"))},
    {step: 7, tool: "search", args: "default, after the human approval",
     shows: {hits_on_doc: [hits_on($s_after)[] | {layer}]},
     pass: ((hits_on($s_after) | length) > 0 and all(hits_on($s_after)[]; .layer == "canonical"))},
    {step: 8, tool: "trace",
     shows: {traces: ($trace.structuredContent.traces | length),
             returned_from_doc: [$trace.structuredContent.traces[] | [.returned[] | select(.doc_path == $doc)] | length]},
     pass: (($trace.structuredContent.traces | length) == 3
            and all($trace.structuredContent.traces[]; .consumer == "scene-agent/1.0")
            and ($trace.structuredContent.traces[-1].returned | any(.doc_path == $doc)))},
    {step: 9, tool: "checkpoint",
     shows: {checkpoint_id: $cp.structuredContent.checkpoint_id, event_seq: $cp.structuredContent.event_seq,
             same_as_search_step_7: ($cp.structuredContent.checkpoint_id == $s_after.structuredContent.checkpoint_id)},
     pass: ($cp.isError == false and $cp.structuredContent.checkpoint_id == $s_after.structuredContent.checkpoint_id
            and ($cp.structuredContent.docs | has($doc)))}
  ] as $steps
  | {scene: "P1-D agent side (NORTH_STAR first product scene)", consumer: "scene-agent/1.0",
     steps: $steps, passed: ([$steps[] | select(.pass)] | length), total: ($steps | length)}
  | .pass = (.passed == .total)' >"$out/scene.json"
if [ "${MDK_SCENE_KEEP:-0}" = 1 ]; then rm -rf "$out/scene_vault"; cp -R "$work" "$out/scene_vault"; fi
jq -r '.steps[] | "scene: \(if .pass then "ok  " else "FAIL" end) \(.step) \(.tool)"' "$out/scene.json"
jq -r '"scene: \(.passed)/\(.total) (\(.consumer))"' "$out/scene.json"
jq -e .pass "$out/scene.json" >/dev/null
