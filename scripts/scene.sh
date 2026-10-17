#!/usr/bin/env bash
# The first product scene end to end (NORTH_STAR; GOAL P1-E E-1, E-3, E-6;
# TASKS E4): the agent side over MCP stdio, the human side over the HTTP
# contract (the plugin's channel), on a clean vault.
#
#   scripts/scene.sh [<out-dir>]      (default reports/rounds/P1-E/scene)
#
# Vault: spec/fixtures/mini_vault copied and ingested in a fresh temporary
# directory; the first `rootmark serve` writes the default policy (human_only,
# append_only), to which `[identity] human_id = "editor"` is added. One
# server process at a time (never two on the vault, contract §1): each
# agent session is its own `rootmark serve --mcp stdio` (server/examples/
# mcp_client.rs, client `scene-agent/1.0`), each human step block one
# `rootmark serve --http 127.0.0.1:0` (X-Consumer `scene-human`).
# Steps:
#    1. status            HTTP and MCP: spec version, policies, the human
#    2. checkpoint        HTTP {name: "before"}: the state before the agent
#    3. propose           MCP: agent/scene.md (new document, plain Markdown)
#    4. blame             HTTP: every block of it proposal, Agent, scene-agent/1.0
#    5. search            HTTP default: the proposal is not found
#    6. promote           HTTP with a reason: Promoted by human:editor (each
#                         block); one without a reason → 400, no event (E-5)
#    7. search            HTTP default: found, canonical
#    8. trace             HTTP: the answer of step 7 rested on its blocks
#    9. back to "before"  HTTP search at "before": not found; blame at
#                         "before": the agent document had no snapshot, the
#                         human espresso.md reads as it was
#   10. agent vs canonical MCP: replace_section of the human "Recipe" →
#                         denied; a heading-less append → policy_violation +
#                         hint; nothing written
#   11. unchanged         HTTP: blame of espresso.md and the default search
#                         equal to before step 10 (E-6)
# After every step the sha256 of every Markdown file is recorded (E-3: they
# change only at the propose step). <out-dir>/scene.json holds one record
# per step and `pass` (no time stamp, token or absolute path: two runs are
# byte-equal); <out-dir>/events.jsonl, traces.jsonl, proposals.jsonl are the
# vault's logs (E-8). Exit 1 unless every step passes. ROOTMARK_SCENE_KEEP=1
# keeps the vault in <out-dir>/scene_vault.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-$root/reports/rounds/P1-E/scene}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
cd "$root"
cargo build -q --release -p rootmark-server --bin rootmark --example mcp_client
rootmark="$root/target/release/rootmark"
client="$root/target/release/examples/mcp_client"
work="$(mktemp -d -t rootmark-scene-e.XXXXXX)"
serve_pid=""
cleanup() {
  rc=$?
  [ -n "$serve_pid" ] && kill "$serve_pid" 2>/dev/null || true
  if [ "$rc" != 0 ] && [ -s "$work/serve.err" ]; then echo "scene: server stderr:" >&2; tail -30 "$work/serve.err" >&2; fi
  rm -rf "$work"
}
trap cleanup EXIT
cd "$work"
mkdir vault rec
cp "$root"/spec/fixtures/mini_vault/*.md vault/
"$rootmark" ingest-dir vault vault >/dev/null

INIT='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"scene-agent","version":"1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}'
DOC="agent/scene.md"
TEXT='# Grind notes\n\nAn agent recommends a coarser grind for the kalimba roast.\n'
call() { printf '{"jsonrpc":"2.0","id":%d,"method":"tools/call","params":{"name":"%s","arguments":%s}}\n' "$1" "$2" "$3"; }
agent() { # agent <name> <calls…>: one MCP session → rec/<name>.jsonl
  local n="$1"; shift
  { printf '%s\n' "$INIT"; for c in "$@"; do printf '%s\n' "$c"; done; } >"$n.in"
  "$client" "$n.in" "rec/$n.jsonl" -- "$rootmark" serve --vault vault --mcp stdio 2>>serve.err
}
res() { jq -c --argjson id "$2" 'select(.request.id == $id) | .response.result' "rec/$1.jsonl"; }

http_up() {
  : >http.err
  "$rootmark" serve --vault vault --http 127.0.0.1:0 </dev/null >/dev/null 2>http.err &
  serve_pid=$!
  port=""
  for _ in $(seq 1 600); do
    port="$(sed -n 's/^rootmark serve: http listening on 127\.0\.0\.1:\([0-9]*\)$/\1/p' http.err)"
    [ -n "$port" ] && break
    kill -0 "$serve_pid" 2>/dev/null || break
    sleep 0.1
  done
  cat http.err >>serve.err
  [ -n "$port" ] || { echo "scene: rootmark serve --http did not start" >&2; exit 1; }
  token="$(tr -d '[:space:]' <vault/.rootmark/server.token)"
}
http_down() {
  kill "$serve_pid" 2>/dev/null || true
  wait "$serve_pid" 2>/dev/null || true
  serve_pid=""
}
# h <name> <tool> <json body>: POST /v1/<tool> → rec/<name>.json {status, body}
h() {
  local st
  st="$(curl -sS -o body.json -w '%{http_code}' -X POST -H "Authorization: Bearer $token" \
    -H 'X-Consumer: scene-human' -H 'Content-Type: application/json' --data-binary "$3" \
    "http://127.0.0.1:$port/v1/$2")"
  jq -c --argjson st "$st" '{status: $st, body: .}' body.json >"rec/$1.json"
}
hget() { jq -c "$2" "rec/$1.json"; }
md_sha() { # md_sha <step>: {path: sha256} of every Markdown file → rec/md_<step>.json
  (cd vault && find . -name '*.md' -not -path './.rootmark/*' | sed 's#^\./##' | LC_ALL=C sort | while read -r f; do
    printf '%s\t%s\n' "$f" "$(shasum -a 256 "$f" | cut -d' ' -f1)"; done) |
    jq -R -s 'split("\n") | map(select(. != "") | split("\t") | {(.[0]): .[1]}) | add // {}' >"rec/md_$1.json"
}
md_sha 0

# 1. status (the agent's first session writes the default policy)
c1="$(call 2 status '{}')"
agent s1 "$c1"
sed -i.bak 's/^# \[identity\]$/[identity]/; s/^# human_id = .*$/human_id = "editor"/' vault/.rootmark/policy.toml
rm -f vault/.rootmark/policy.toml.bak
grep -q '^human_id = "editor"$' vault/.rootmark/policy.toml || { echo "scene: could not set [identity] human_id" >&2; exit 1; }
http_up
h st1 status '{}'
md_sha 1
# 2. checkpoint "before"
h cp2 checkpoint '{"name":"before"}'
http_down
md_sha 2
# 3. the agent proposes
# arguments are built as JSON first (no brace expansion in nested quotes)
a3="$(jq -nc --arg d "$DOC" --arg t "$(printf "$TEXT")" '{doc_path: $d, content: $t, reason: "product scene"}')"
c3="$(call 2 propose "$a3")"
agent s3 "$c3"
md_sha 3
lineages="$(res s3 2 | jq -r '.structuredContent.blocks[].lineage_id')"
http_up
# 4. blame
h bl4 blame "{\"doc_path\":\"$DOC\"}"
md_sha 4
# 5. default search
h se5 search '{"query":"kalimba","mode":"lexical"}'
md_sha 5
# 6. promote, each block, with a reason; one without
ev_before6="$(wc -l <vault/.rootmark/events.jsonl | tr -d ' ')"
h pr6_noreason promote "{\"lineage_id\":\"$(echo "$lineages" | head -1)\",\"reason\":\"\"}"
ev_noreason6="$(wc -l <vault/.rootmark/events.jsonl | tr -d ' ')"
i=0
for l in $lineages; do
  i=$((i + 1))
  h "pr6_$i" promote "{\"lineage_id\":\"$l\",\"reason\":\"approved in the scene\"}"
done
n_promote=$i
jq -s -c '.' $(for j in $(seq 1 "$n_promote"); do echo "rec/pr6_$j.json"; done) >rec/pr6.json
md_sha 6
# 7. default search
h se7 search '{"query":"kalimba","mode":"lexical"}'
md_sha 7
# 8. trace
h tr8 trace '{"consumer":"scene-human"}'
md_sha 8
# 9. back to "before"
h se9 search '{"query":"kalimba","mode":"lexical","at":"before"}'
h bl9_doc blame "{\"doc_path\":\"$DOC\",\"at\":\"before\"}"
h bl9_esp blame '{"doc_path":"espresso.md","at":"before"}'
h bl9_esp_now blame '{"doc_path":"espresso.md"}'
md_sha 9
h bl10_before blame '{"doc_path":"espresso.md"}'
h se10_before search '{"query":"espresso recipe","mode":"lexical"}'
http_down
ev_before10="$(wc -l <vault/.rootmark/events.jsonl | tr -d ' ')"
# 10. the agent tries to change canonical content
c10a="$(call 2 propose '{"doc_path":"espresso.md","mode":"replace_section","heading":"Recipe","content":"Agents rewrite the recipe.","reason":"scene: change canonical"}')"
c10b="$(call 3 propose '{"doc_path":"espresso.md","mode":"append","content":"An agent line without a heading.","reason":"scene: heading-less append"}')"
agent s10 "$c10a" "$c10b"
ev_after10="$(wc -l <vault/.rootmark/events.jsonl | tr -d ' ')"
md_sha 10
# 11. unchanged
http_up
h bl11 blame '{"doc_path":"espresso.md"}'
h se11 search '{"query":"espresso recipe","mode":"lexical"}'
http_down
md_sha 11

# the Markdown files that changed at each step (E-3)
changed() { jq -n --slurpfile a "rec/md_$1.json" --slurpfile b "rec/md_$2.json" \
  '[($a[0] + $b[0]) | keys[] as $k | select($a[0][$k] != $b[0][$k]) | $k]'; }
md_changes="$(for s in $(seq 1 11); do changed $((s - 1)) "$s"; done | jq -s -c '.')"

jq -n --arg doc "$DOC" \
  --argjson s1 "$(res s1 2)" --argjson st1 "$(cat rec/st1.json)" --argjson cp2 "$(cat rec/cp2.json)" \
  --argjson s3 "$(res s3 2)" --argjson bl4 "$(cat rec/bl4.json)" --argjson se5 "$(cat rec/se5.json)" \
  --argjson nr6 "$(cat rec/pr6_noreason.json)" --argjson pr6 "$(cat rec/pr6.json)" \
  --argjson ev_before6 "$ev_before6" --argjson ev_noreason6 "$ev_noreason6" \
  --argjson se7 "$(cat rec/se7.json)" --argjson tr8 "$(cat rec/tr8.json)" \
  --argjson se9 "$(cat rec/se9.json)" --argjson bl9d "$(cat rec/bl9_doc.json)" \
  --argjson bl9e "$(cat rec/bl9_esp.json)" --argjson bl9n "$(cat rec/bl9_esp_now.json)" \
  --argjson bl10b "$(cat rec/bl10_before.json)" --argjson se10b "$(cat rec/se10_before.json)" \
  --argjson r10a "$(res s10 2)" --argjson r10b "$(res s10 3)" \
  --argjson ev_before10 "$ev_before10" --argjson ev_after10 "$ev_after10" \
  --argjson bl11 "$(cat rec/bl11.json)" --argjson se11 "$(cat rec/se11.json)" \
  --argjson md "$md_changes" '
  def on_doc($r): [$r.body.hits[] | select(.doc_path == $doc)];
  def md_only($i; $want): $md[$i - 1] == $want;
  [
    {step: 1, actor: "human+agent", tool: "status",
     shows: {spec_version: $st1.body.spec_version, promotion: $st1.body.policy.promotion, edit: $st1.body.policy.edit,
             human_id: $st1.body.server_policy.identity.human_id, mcp_spec_version: $s1.structuredContent.spec_version},
     pass: ($st1.status == 200 and $s1.isError == false and $st1.body.policy.promotion == "human_only"
            and $st1.body.policy.edit == "append_only" and $st1.body.server_policy.identity.human_id == "editor"
            and md_only(1; []))},
    {step: 2, actor: "human", tool: "checkpoint", args: "name before",
     shows: {name: $cp2.body.name, checkpoint_id: $cp2.body.checkpoint_id, event_seq: $cp2.body.event_seq},
     pass: ($cp2.status == 200 and $cp2.body.name == "before" and md_only(2; []))},
    {step: 3, actor: "agent", tool: "propose", args: $doc,
     shows: {blocks: [$s3.structuredContent.blocks[] | {layer, author_kind, source}], md_changed: $md[2]},
     pass: ($s3.isError == false and all($s3.structuredContent.blocks[]; .layer == "proposal" and .author_kind == "Agent")
            and md_only(3; [$doc]))},
    {step: 4, actor: "human", tool: "blame", args: $doc,
     shows: {blocks: [$bl4.body.blocks[] | {layer, author_kind, source}]},
     pass: ($bl4.status == 200 and ($bl4.body.blocks | length) > 0
            and all($bl4.body.blocks[]; .layer == "proposal" and .author_kind == "Agent" and .source == "scene-agent/1.0")
            and md_only(4; []))},
    {step: 5, actor: "human", tool: "search", args: "default",
     shows: {hits_on_doc: (on_doc($se5) | length)},
     pass: ($se5.status == 200 and (on_doc($se5) | length) == 0 and md_only(5; []))},
    {step: 6, actor: "human", tool: "promote", args: "each block, with a reason; one without",
     shows: {decisions: [$pr6[] | .body.decision], actors: [$pr6[] | .body.event.meta.actor],
             reasons: [$pr6[] | .body.event.meta.reason],
             without_reason: {status: $nr6.status, code: $nr6.body.error.code, events_written: ($ev_noreason6 - $ev_before6)}},
     pass: (all($pr6[]; .status == 200 and .body.decision == "promoted" and .body.event.kind == "Promoted"
                and .body.event.meta.actor == {id: "editor", kind: "human"} and .body.event.meta.reason == "approved in the scene")
            and $nr6.status == 400 and $nr6.body.error.code == "invalid_arguments" and $ev_noreason6 == $ev_before6
            and md_only(6; []))},
    {step: 7, actor: "human", tool: "search", args: "default, after the promotion",
     shows: {hits_on_doc: [on_doc($se7)[] | {layer}]},
     pass: ($se7.status == 200 and (on_doc($se7) | length) > 0 and all(on_doc($se7)[]; .layer == "canonical") and md_only(7; []))},
    {step: 8, actor: "human", tool: "trace",
     shows: {traces: ($tr8.body.traces | length),
             step7_trace_is_last: ($tr8.body.traces[-1].trace_id == $se7.body.trace_id),
             returned_from_doc: [$tr8.body.traces[-1].returned[] | select(.doc_path == $doc) | .lineage_id] | length},
     pass: ($tr8.status == 200 and $tr8.body.traces[-1].trace_id == $se7.body.trace_id
            and ([$tr8.body.traces[-1].returned[] | select(.doc_path == $doc)] | length) > 0 and md_only(8; []))},
    {step: 9, actor: "human", tool: "search+blame", args: "at before",
     shows: {checkpoint_id: $se9.body.checkpoint_id, same_as_step_2: ($se9.body.checkpoint_id == $cp2.body.checkpoint_id),
             hits_on_doc: (on_doc($se9) | length), doc_blame_then: $bl9d.body.error.code,
             espresso_then_equals_now: ($bl9e.body.blocks == $bl9n.body.blocks)},
     pass: ($se9.status == 200 and $se9.body.checkpoint_id == $cp2.body.checkpoint_id and (on_doc($se9) | length) == 0
            and $bl9d.status == 404 and $bl9d.body.error.code == "not_found"
            and $bl9e.status == 200 and ($bl9e.body.blocks | length) > 0 and $bl9e.body.blocks == $bl9n.body.blocks
            and md_only(9; []))},
    {step: 10, actor: "agent", tool: "propose", args: "replace_section Recipe; heading-less append",
     shows: {replace_section: $r10a.structuredContent.error.code, append: $r10b.structuredContent.error.code,
             hint: $r10b.structuredContent.error.hint, events_written: ($ev_after10 - $ev_before10)},
     pass: ($r10a.isError == true and $r10a.structuredContent.error.code == "denied"
            and $r10b.isError == true and $r10b.structuredContent.error.code == "policy_violation"
            and $r10b.structuredContent.error.hint == "start the appended text with a heading"
            and $ev_after10 == $ev_before10 and md_only(10; []))},
    {step: 11, actor: "human", tool: "blame+search", args: "espresso.md, after the denied attempt",
     shows: {blame_unchanged: ($bl11.body.blocks == $bl10b.body.blocks), search_unchanged: ($se11.body.hits == $se10b.body.hits)},
     pass: ($bl11.status == 200 and $bl11.body.blocks == $bl10b.body.blocks and $se11.body.hits == $se10b.body.hits
            and ($se11.body.hits | length) > 0 and md_only(11; []))}
  ] as $steps
  | {scene: "P1-E first product scene (agent over MCP, human over HTTP)", agent: "scene-agent/1.0",
     human: "human:editor", md_changed_per_step: $md,
     e3_md_changes_only_at_propose: ([range(0; 11) | select($md[.] != [])] == [2]),
     steps: $steps, passed: ([$steps[] | select(.pass)] | length), total: ($steps | length)}
  | .pass = (.passed == .total and .e3_md_changes_only_at_propose)' >"$out/scene.json"
cp vault/.rootmark/events.jsonl vault/.rootmark/traces.jsonl vault/.rootmark/proposals.jsonl "$out/"
if [ "${ROOTMARK_SCENE_KEEP:-0}" = 1 ]; then rm -rf "$out/scene_vault"; cp -R "$work/vault" "$out/scene_vault"; fi
jq -r '.steps[] | "scene: \(if .pass then "ok  " else "FAIL" end) \(.step) \(.actor) \(.tool)"' "$out/scene.json"
jq -r '"scene: \(.passed)/\(.total); E-3 Markdown changed only at the propose step: \(.e3_md_changes_only_at_propose)"' "$out/scene.json"
jq -e .pass "$out/scene.json" >/dev/null
