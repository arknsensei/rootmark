#!/usr/bin/env bash
# P1-D.1 criteria (GOAL D1-1..D1-9, D1-11, D1-13; TASKS D1.6) into <out-dir>:
#   1. server/tests/p1d1_criteria.rs with ROOTMARK_P1D1_OUT=<out-dir> →
#      d1_1.json … d1_9.json (each written before its threshold is asserted;
#      a failing criterion is recorded, a missing file fails this script);
#   2. D1-11 old vaults: the `p1d-final` tree (git archive) runs its own P1-D
#      agent scene (MDK_SCENE_KEEP=1) and MCP e2e (agent replace_section with
#      deletions written without an actor); every document of both vaults
#      replays to the same snapshot_id under the p1d-final `mdk` and the
#      current `rootmark` (after `rootmark migrate --rename-store`: the old
#      vaults keep their metadata under .mdkernel/, K-GH.1), `events_hash` and
#      the events.jsonl bytes are unchanged after the current `rootmark`
#      opened them (replay, blame, history, checkpoint list, events-hash), and
#      the v3-final vaults pass
#      scripts/check_v3_compat.sh → d1_11.json, v3_compat*.json;
#   3. D1-13: scripts/contract_check.sh → d1_13.json;
#   4. p1d1.json: every criterion's `pass`.
# Needs the model files; builds the p1d-final tree (its lock, its ONNX Runtime).
# Exit 0 when every result file exists (criteria are recorded, not fatal);
# exit 1 when one is missing.
#
#   scripts/test_p1d1.sh <out-dir>
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: test_p1d1.sh <out-dir>}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
work="$(mktemp -d -t rootmark-p1d1.XXXXXX)"
trap 'rm -rf "$work"' EXIT
export ROOTMARK_MODELS="${ROOTMARK_MODELS:-${MDK_MODELS:-$here/models}}"
# the p1d-final binary reads the model directory under its own name
export MDK_MODELS="$ROOTMARK_MODELS"

cd "$here"
cargo build -q --release -p rootmark-server --bin rootmark --example mcp_pipeline
new="$here/target/release/rootmark"

# 1. D1-1..D1-9
ROOTMARK_P1D1_OUT="$out" cargo test -q --release -p rootmark-server --test p1d1_criteria -- --test-threads=1 \
  >"$out/p1d1_criteria.log" 2>&1 || echo "test_p1d1: p1d1_criteria exit $? (recorded)" >&2
for n in 1 2 3 4 5 6 7 8 9; do
  [ -f "$out/d1_$n.json" ] || { tail -40 "$out/p1d1_criteria.log" >&2; echo "test_p1d1: d1_$n.json missing" >&2; exit 1; }
done

# 2. D1-11
sha="$(git rev-parse --verify 'p1d-final^{commit}')"
mkdir -p "$work/p1d"
git archive "$sha" | tar -xf - -C "$work/p1d"
# the p1d-final scripts build into and run from their own tree (target/)
(cd "$work/p1d" && MDK_SCENE_KEEP=1 bash scripts/scene_agent.sh "$work/scene" >"$out/d1_11_scene.log" 2>&1) \
  || { tail -20 "$out/d1_11_scene.log" >&2; echo "test_p1d1: p1d-final scene failed" >&2; exit 1; }
(cd "$work/p1d" && bash scripts/mcp_e2e.sh "$work/e2e" >"$out/d1_11_e2e.log" 2>&1) \
  || echo "test_p1d1: p1d-final mcp_e2e exit $? (its own checks are not D1-11)" >&2
old="$work/p1d/target/release/mdk"
[ -x "$old" ] || { echo "test_p1d1: no p1d-final mdk" >&2; exit 1; }

vaults=()
[ -d "$work/scene/scene_vault/vault" ] && vaults+=("scene:$work/scene/scene_vault/vault")
[ -d "$work/e2e/run1/vault" ] && vaults+=("e2e:$work/e2e/run1/vault")
results=()
for entry in "${vaults[@]}"; do
  name="${entry%%:*}"; v="${entry#*:}"
  # the p1d-final binary reads the vault as it wrote it (.mdkernel/) …
  log="$v/.mdkernel/events.jsonl"
  bytes_before="$(shasum -a 256 "$log" | cut -d' ' -f1)"
  h_old="$("$old" events-hash "$v" | jq -r .events_hash)"
  jq -r 'select(.kind == "SnapshotCommitted") | .doc_path' "$log" | sort -u >"$work/docs_$name"
  : >"$work/old_ids_$name"
  while IFS= read -r doc; do
    "$old" replay "$v" "$doc" | jq -r .snapshot_id >>"$work/old_ids_$name"
  done <"$work/docs_$name"
  # … and the current one after renaming it to .rootmark/ (K-GH.1)
  "$new" migrate --rename-store "$v" >"$work/migrate_$name.json" \
    || { echo "test_p1d1: rootmark migrate --rename-store failed on the $name vault" >&2; exit 1; }
  log="$v/.rootmark/events.jsonl"
  docs=0; equal=0; blame_ok=0
  while IFS= read -r doc && IFS= read -r a <&3; do
    docs=$((docs + 1))
    b="$("$new" replay "$v" "$doc" | jq -r .snapshot_id)"
    [ -n "$a" ] && [ "$a" = "$b" ] && equal=$((equal + 1))
    "$new" blame "$v" "$doc" >/dev/null 2>&1 && blame_ok=$((blame_ok + 1))
  done <"$work/docs_$name" 3<"$work/old_ids_$name"
  # an agent deletion written before K-D1.2 reads without an actor
  dels="$(jq -c 'select(.kind == "BlockDeleted")' "$log" | wc -l | tr -d ' ')"
  del_l="$(jq -r 'select(.kind == "BlockDeleted") | .lineage_ids[0]' "$log" | head -1)"
  hist_ok=true
  if [ -n "$del_l" ]; then
    "$new" history "$v" "$del_l" >"$work/hist.json" 2>/dev/null \
      && jq -e '[.events[] | select(.kind == "BlockDeleted")] | length > 0 and all(.meta.actor == null)' "$work/hist.json" >/dev/null \
      || hist_ok=false
  fi
  "$new" checkpoint list "$v" >/dev/null
  h_new="$("$new" events-hash "$v" | jq -r .events_hash)"
  bytes_after="$(shasum -a 256 "$log" | cut -d' ' -f1)"
  # the old index (format mdkernel-index/2) is refused, not misread (K-D1.7)
  idx="$("$new" index status "$v" 2>&1 >/dev/null || true)"
  idx_refused=false; echo "$idx" | grep -q "mdkernel-index/3" && idx_refused=true
  results+=("$(jq -n --arg n "$name" --argjson docs "$docs" --argjson eq "$equal" --argjson bl "$blame_ok" \
    --arg ho "$h_old" --arg hn "$h_new" --argjson dels "$dels" --argjson hist "$hist_ok" \
    --argjson bytes "$([ "$bytes_before" = "$bytes_after" ] && echo true || echo false)" \
    --argjson idx "$idx_refused" \
    '{vault: $n, docs: $docs, replay_equal: $eq, blame_ok: $bl, events_hash_p1d_final: $ho,
      events_hash_current: $hn, events_hash_equal: ($ho == $hn), events_bytes_unchanged: $bytes,
      block_deleted_events: $dels, deletion_history_without_actor: $hist,
      old_index_refused_with_rebuild_hint: $idx,
      pass: ($docs > 0 and $eq == $docs and $bl == $docs and $ho == $hn and $bytes and $hist
             and ($n != "e2e" or $dels > 0))}')")
done
v3ok=true
bash scripts/check_v3_compat.sh "$out/v3_compat.json" "$out/v3_compat_mixed.json" >"$out/v3_compat.log" 2>&1 || v3ok=false
printf '%s\n' "${results[@]}" | jq -s --argjson v3 "$v3ok" --arg sha "$sha" \
  '{criterion: "D1-11", p1d_final: $sha, vaults: ., v3_final_compat: $v3,
    pass: (length == 2 and all(.pass) and $v3)}' >"$out/d1_11.json"

# 3. D1-13
bash scripts/contract_check.sh "$out/d1_13.json" >"$out/d1_13.log" 2>&1 || echo "test_p1d1: contract_check exit $? (recorded)" >&2
[ -f "$out/d1_13.json" ] || { cat "$out/d1_13.log" >&2; exit 1; }

# 4. summary
jq -s 'map({key: (.criterion // "D1-13"), value: .pass}) | from_entries | {criteria: ., pass: (to_entries | all(.value))}' \
  "$out"/d1_1.json "$out"/d1_2.json "$out"/d1_3.json "$out"/d1_4.json "$out"/d1_5.json \
  "$out"/d1_6.json "$out"/d1_7.json "$out"/d1_8.json "$out"/d1_9.json "$out"/d1_11.json "$out"/d1_13.json \
  >"$out/p1d1.json"
jq -c . "$out/p1d1.json"
