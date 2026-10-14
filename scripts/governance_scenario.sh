#!/usr/bin/env bash
# Deterministic governance scenario (P1-B GOAL B-4; VERIFY V1): drives every
# governance command of `mdk` on a fresh vault built from the mini_vault
# fixture and leaves the artefacts the pair comparison checks byte for byte.
#
#   scripts/governance_scenario.sh <mdk> <out-dir>
#
# <out-dir>/vault/.mdkernel/{events.jsonl, traces.jsonl, checkpoints/},
# <out-dir>/governance.json (summary: hashes, layers, checkpoints, traces).
# Every time is given explicitly (--now-ms); nothing depends on the clock,
# the machine or the directory (doc paths are vault-relative).
set -euo pipefail

mdk="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
out="${2:?usage: governance_scenario.sh <mdk> <out-dir>}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
[ ! -e "$out/vault" ] || { echo "error: $out/vault exists" >&2; exit 1; }
v="$out/vault"
mkdir -p "$v/.mdkernel"
cat >"$v/.mdkernel/policy.toml" <<'TOML'
promotion = "human_only"

[version_trigger]
idle_ms = 60000
min_sim_change = 0.15

[journal]
retention_hours = 24
TOML

j() { "$mdk" "$@"; }

# human corpus
cp -R "$here/spec/fixtures/mini_vault/." "$v/"
j ingest-dir "$v" "$v" >/dev/null
cp0="$(j checkpoint create "$v" --name baseline | jq -r .checkpoint_id)"

# agent proposals: a new note and an edit of a human document
work="$out/work"
mkdir -p "$work"
cat >"$work/agent-note.md" <<'MD'
# Agent notes

The agent suggests weighing every espresso dose to a tenth of a gram.

The agent suggests logging grinder settings next to each roast batch.

The agent suggests a weekly backflush with detergent for the group heads.
MD
j propose "$v" agents/notes.md "$work/agent-note.md" --source claude-code >"$work/propose1.json"
sed 's/clean, sweet cups/clean, sweet and balanced cups/' "$v/espresso.md" >"$work/espresso.md"
if cmp -s "$work/espresso.md" "$v/espresso.md"; then
  printf '\nThe agent adds that shots should be timed from the first drop.\n' >>"$work/espresso.md"
fi
j propose "$v" espresso.md "$work/espresso.md" --source claude-code >"$work/propose2.json"
cp1="$(j checkpoint create "$v" --name proposed | jq -r .checkpoint_id)"

# governance: refused promotion (agent actor), promote, demote, re-promote, prune
L=()
while IFS= read -r l; do L+=("$l"); done < <(jq -r '.layers[] | select(.layer == "proposal") | .lineage_id' "$work/propose1.json")
[ "${#L[@]}" -ge 4 ] || { echo "error: expected ≥ 4 proposal lineages" >&2; exit 1; }
if j promote "$v" "${L[1]}" --actor agent:claude --reason self >/dev/null 2>&1; then
  echo "error: agent promotion was not refused" >&2
  exit 1
fi
j promote "$v" "${L[1]}" --actor human:reviewer --reason "checked against the scale log" >/dev/null
j promote "$v" "${L[2]}" --actor human:reviewer --reason "matches the cleaning plan" >/dev/null
j demote "$v" "${L[2]}" --actor human:reviewer --reason "needs the grinder manual first" >/dev/null
j prune "$v" "${L[3]}" --actor human:reviewer --reason "duplicate of maintenance.md" >/dev/null
cp2="$(j checkpoint create "$v" --name reviewed | jq -r .checkpoint_id)"

# consumption: the same reads at three checkpoints, one repeated
j read "$v" --at baseline --consumer claude-code "$(j blame "$v" espresso.md --at baseline | jq -r '.blocks[1].lineage_id')" >/dev/null
for cp in proposed reviewed reviewed; do
  j read "$v" --at "$cp" --consumer claude-code "${L[1]}" "${L[2]}" "${L[3]}" >/dev/null
done

# journal with explicit times: distance, then idle, then pruning
cp "$v/glossary.md" "$work/glossary.md"
t=1760400000000
j journal append "$v" glossary.md --now-ms "$t" --author human --source editor >/dev/null
printf '\nCrema: the foam on top of an espresso shot.\n' >>"$v/glossary.md"
j journal append "$v" glossary.md --now-ms $((t + 1000)) --author human --source editor >/dev/null
printf '\nTamp: to press ground coffee into the basket.\n' >>"$v/glossary.md"
j journal append "$v" glossary.md --now-ms $((t + 2000)) --author human --source editor >/dev/null
j journal tick "$v" glossary.md --now-ms $((t + 70000)) --author human --source editor >/dev/null
j journal prune "$v" --now-ms $((t + 30 * 3600000)) >"$work/journal_prune.json"
cp3="$(j checkpoint create "$v" --name final | jq -r .checkpoint_id)"
# same state again: same id, nothing written
again="$(j checkpoint create "$v" --name final | jq -r .checkpoint_id)"
[ "$again" = "$cp3" ] || { echo "error: checkpoint not deterministic" >&2; exit 1; }

b3() { python3 -c 'import sys,hashlib;print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())' "$1"; }
m="$v/.mdkernel"
layers="$(j blame "$v" agents/notes.md | jq -c '[.blocks[] | {layer, inactive, seq_created, seq_last}]')"
layers_then="$(j blame "$v" agents/notes.md --at proposed | jq -c '[.blocks[] | {layer, inactive}]')"
jq -n \
  --arg eh "$(j events-hash "$v" | jq -r .events_hash)" \
  --arg traces "$(b3 "$m/traces.jsonl")" \
  --argjson ntraces "$(wc -l <"$m/traces.jsonl" | tr -d ' ')" \
  --argjson cps "$(j checkpoint list "$v" | jq '[.checkpoints[] | {checkpoint_id, name, event_seq}]')" \
  --argjson layers "$layers" --argjson layers_then "$layers_then" \
  --argjson prune "$(cat "$work/journal_prune.json")" \
  --argjson journal "$(j journal list "$v" glossary.md | jq '[.records[] | {seq, content_hash, bytes_len}]')" \
  --argjson events "$(wc -l <"$m/events.jsonl" | tr -d ' ')" \
  '{events_hash: $eh, events: $events, traces_sha256: $traces, traces: $ntraces,
    checkpoints: $cps, agent_note_layers: $layers, agent_note_layers_at_proposed: $layers_then,
    journal_prune: $prune, journal: $journal}' >"$out/governance.json"
rm -rf "$work"
echo "governance scenario: $(jq -c '{events_hash, traces, checkpoints: (.checkpoints | length)}' "$out/governance.json")" >&2
