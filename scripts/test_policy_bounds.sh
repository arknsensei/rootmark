#!/usr/bin/env bash
# GOAL P1-B B-6 (VERIFY V5 g): six policy-bound cases, each must fail with its
# own error and none may fall back to built-in values. Every case runs in a
# scratch vault; the output of every failing command is searched for the word
# "default" (case-insensitive) and for any JSON on stdout.
#
#   scripts/test_policy_bounds.sh <mdk> [<out.json>]
set -euo pipefail

mdk="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
out="${2:-}"
scratch="$(mktemp -d -t mdk-bounds.XXXXXX)"
trap 'rm -rf "$scratch"' EXIT
v="$scratch/vault"
mkdir -p "$v/.mdkernel"
printf '# D\n\nAgent text for the bound cases.\n' >"$scratch/d.md"
"$mdk" propose "$v" d.md "$scratch/d.md" --source agent >"$scratch/p.json"
l="$(jq -r '.layers[1].lineage_id' "$scratch/p.json")"
good='promotion = "human_only"

[version_trigger]
idle_ms = 1000
min_sim_change = 0.2

[journal]
retention_hours = 24
'
n=0
results="[]"
case_fail() { # case_fail <id> <label> <pattern> <command...>
  local id="$1" label="$2" pat="$3"; shift 3
  local rc=0
  "$@" >"$scratch/stdout" 2>"$scratch/stderr" || rc=$?
  if [ "$rc" -eq 0 ]; then
    echo "FAIL $id $label: exit 0" >&2; cat "$scratch/stdout" >&2; exit 1
  fi
  if ! grep -q -- "$pat" "$scratch/stderr"; then
    echo "FAIL $id $label: no '$pat' in: $(cat "$scratch/stderr")" >&2; exit 1
  fi
  if grep -qi default "$scratch/stdout" "$scratch/stderr" || [ -s "$scratch/stdout" ]; then
    echo "FAIL $id $label: fallback wording or output" >&2; cat "$scratch/stdout" "$scratch/stderr" >&2; exit 1
  fi
  echo "ok   $id $label (exit $rc: $(head -c 160 "$scratch/stderr" | tr '\n' ' '))"
  # the matched pattern, not the message (messages hold temp paths)
  results="$(jq -c --arg id "$id" --arg l "$label" --argjson rc "$rc" --arg e "$pat" \
    '. + [{id: $id, case: $l, exit: $rc, error_matches: $e, stdout_empty: true, default_word: false}]' <<<"$results")"
  n=$((n + 1))
}
policy() { printf '%s' "$1" >"$v/.mdkernel/policy.toml"; }
promote=("$mdk" promote "$v" "$l" --actor human:alice --reason ok)

policy "${good/retention_hours = 24/retention_hours = 23}"
case_fail B6-1 "retention < 24h" "kernel minimum is 24" "$mdk" policy show "$v"
policy "${good/idle_ms = 1000/idle_ms = -1}"
case_fail B6-2 "idle_ms < 0" "idle_ms = -1: must be >= 0" "$mdk" journal prune "$v" --now-ms 0
policy "${good/min_sim_change = 0.2/min_sim_change = 1.5}"
case_fail B6-3 "min_sim_change > 1" "must lie in \[0, 1\]" "$mdk" policy show "$v"
policy "${good/human_only/implicit}"
case_fail B6-4 "unknown promotion" "unknown promotion policy" "${promote[@]}"
rm "$v/.mdkernel/policy.toml"
case_fail B6-5 "promotion without a hook (no recipe: deny)" "denied by policy deny" "${promote[@]}"
policy "$good"
case_fail B6-6 "Promoted without actor" "actor id is empty" "$mdk" promote "$v" "$l" --actor human: --reason ok
# the vault holds no transition after all six
if grep -q '"Promoted"' "$v/.mdkernel/events.jsonl"; then
  echo "FAIL: a Promoted event was written" >&2; exit 1
fi
# and the valid recipe works
"${promote[@]}" >/dev/null
[ -z "$out" ] || jq -n --argjson r "$results" '{cases: $r, all_err: true, no_fallback: true}' >"$out"
echo "test_policy_bounds: OK ($n cases, all errors, no fallback)"
