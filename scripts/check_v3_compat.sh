#!/usr/bin/env bash
# B-5 (GOAL P1-B; VERIFY V3): a vault written by the **v3-final** `mdk` binary
# (events without layers) opens with the current core: `replay` gives the same
# snapshot_id for every document, `events_hash` and the events.jsonl bytes do
# not change, and `blame` derives every layer from `author_kind`.
#
#   scripts/check_v3_compat.sh [<out.json> [<mixed-out.json>]]
#
# Builds `git archive v3-final core` (like scripts/gen_fixtures.sh) into a
# temp dir; the current `mdk` is `target/release/mdk` of this tree (built here).
#
# P1-B.1 (GOAL B1-1, K-B1.1; freeze exception P1-B.1/F4): then the
# mixed-author scenarios of scripts/v3final-mixed-gen/scenarios.json are
# written afresh with the v3-final **library** API (scripts/gen_v3final_mixed.sh
# into a temp dir; the committed fixtures are not used) and each opens with the
# current `mdk`: every document's `replay` snapshot_id equals v3-final's,
# `events_hash` and the events.jsonl bytes do not change, and `blame` gives
# every block's [author_kind, layer, layer_source] as `expect_blame` states.
# The result goes to <mixed-out.json> (stdout and <out.json> keep the P1-B
# output byte for byte).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-}"
mixed_out="${2:-}"
sha="$(git -C "$here" rev-parse --verify 'v3-final^{commit}')"
work="$(mktemp -d -t mdk-v3compat.XXXXXX)"
trap 'rm -rf "$work"' EXIT

git -C "$here" archive "$sha" Cargo.lock rust-toolchain.toml .cargo core | tar -xf - -C "$work"
printf '[workspace]\nresolver = "2"\nmembers = ["core"]\n' >"$work/Cargo.toml"
(cd "$work" && cargo build --offline --release -q -p mdkernel-core --bin mdk)
old="$work/target/release/mdk"
(cd "$here" && cargo build --release -q -p mdkernel-core --bin mdk)
new="$here/target/release/mdk"

vault="$work/vault"
mkdir -p "$vault"
"$old" ingest-dir "$vault" "$here/spec/fixtures/mini_vault" >"$work/ingest.json"
"$old" ingest-dir "$vault" "$here/core/tests/fixtures/v3_final/edit" >>"$work/ingest.json"
log="$vault/.mdkernel/events.jsonl"
before_bytes="$(shasum -a 256 "$log" | cut -d' ' -f1)"
old_hash="$("$old" events-hash "$vault" | jq -r .events_hash)"

fail=0
docs=0
while IFS= read -r doc; do
  docs=$((docs + 1))
  a="$("$old" replay "$vault" "$doc" | jq -r .snapshot_id)"
  b="$("$new" replay "$vault" "$doc" | jq -r .snapshot_id)"
  if [ "$a" != "$b" ] || [ -z "$a" ] || [ "$a" = null ]; then
    echo "check_v3_compat: $doc replay snapshot_id v3-final $a != current $b" >&2
    fail=1
  fi
  if "$new" blame "$vault" "$doc" >"$work/blame.json" 2>"$work/blame.err"; then
    bad="$(jq '[.blocks[] | select(.layer != "canonical" or .author_kind != "Human")] | length' "$work/blame.json")"
    n="$(jq '.blocks | length' "$work/blame.json")"
    if [ "$bad" != 0 ] || [ "$n" = 0 ]; then
      echo "check_v3_compat: $doc blame: $bad of $n blocks without the derived layer" >&2
      fail=1
    fi
  else
    echo "check_v3_compat: $doc blame failed: $(cat "$work/blame.err")" >&2
    fail=1
  fi
done < <(jq -r '.ingested[].doc_path' "$work/ingest.json" | sort -u)
new_hash="$("$new" events-hash "$vault" | jq -r .events_hash)"
after_bytes="$(shasum -a 256 "$log" | cut -d' ' -f1)"
[ "$old_hash" = "$new_hash" ] || { echo "check_v3_compat: events_hash $old_hash != $new_hash" >&2; fail=1; }
[ "$before_bytes" = "$after_bytes" ] || { echo "check_v3_compat: events.jsonl bytes changed" >&2; fail=1; }

result="$(jq -n --arg sha "$sha" --arg h "$old_hash" --arg h2 "$new_hash" --argjson docs "$docs" \
  --argjson ok "$([ "$fail" -eq 0 ] && echo true || echo false)" \
  '{v3_final_sha: $sha, docs: $docs, events_hash_v3_final: $h, events_hash_current: $h2, replay_equal: $ok, pass: $ok}')"
[ -z "$out" ] || echo "$result" >"$out"
echo "$result"
if [ "$fail" -ne 0 ]; then
  echo "check_v3_compat: FAILED" >&2
  exit 1
fi
echo "check_v3_compat: OK ($docs documents)" >&2

# --- P1-B.1: mixed-author vaults written by the v3-final library API
scen="$here/scripts/v3final-mixed-gen/scenarios.json"
gen="$work/mixed"
bash "$here/scripts/gen_v3final_mixed.sh" "$gen" >/dev/null
mfail=0
mres="[]"
while IFS= read -r name; do
  sfail=0
  mv_="$work/mv-$name"
  mkdir -p "$mv_"
  cp -R "$gen/$name/store" "$mv_/.mdkernel"
  mlog="$mv_/.mdkernel/events.jsonl"
  mb="$(shasum -a 256 "$mlog" | cut -d' ' -f1)"
  mh_old="$("$old" events-hash "$mv_" | jq -r .events_hash)"
  ndocs=0
  ok_blame=0
  while IFS= read -r doc; do
    ndocs=$((ndocs + 1))
    a="$("$old" replay "$mv_" "$doc" | jq -r .snapshot_id)"
    if ! b="$("$new" replay "$mv_" "$doc" 2>"$work/m.err" | jq -r .snapshot_id)" || [ "$a" != "$b" ] || [ -z "$a" ] || [ "$a" = null ]; then
      echo "check_v3_compat: $name/$doc replay snapshot_id v3-final $a != current ${b:-} $(cat "$work/m.err")" >&2
      sfail=1
    fi
    if "$new" blame "$mv_" "$doc" >"$work/m.json" 2>"$work/m.err"; then
      got="$(jq -c '[.blocks[] | [.author_kind, .layer, .layer_source]]' "$work/m.json")"
      want="$(jq -c --arg n "$name" --arg d "$doc" '.scenarios[] | select(.name == $n) | .expect_blame[$d]' "$scen")"
      if [ "$got" = "$want" ]; then
        ok_blame=$((ok_blame + 1))
      else
        echo "check_v3_compat: $name/$doc blame $got != expected $want" >&2
        sfail=1
      fi
    else
      echo "check_v3_compat: $name/$doc blame failed: $(cat "$work/m.err")" >&2
      sfail=1
    fi
  done < <(jq -r --arg n "$name" '.scenarios[] | select(.name == $n) | .expect_blame | keys[]' "$scen")
  # every document of the vault has expectations
  nv="$(jq '.replay_snapshot_ids | length' "$gen/$name/expected.json")"
  [ "$nv" = "$ndocs" ] || { echo "check_v3_compat: $name: $nv documents, $ndocs expected" >&2; sfail=1; }
  mh_new="$("$new" events-hash "$mv_" 2>"$work/m.err" | jq -r .events_hash)" || mh_new=""
  [ "$mh_old" = "$mh_new" ] || { echo "check_v3_compat: $name events_hash $mh_old != ${mh_new:-error}" >&2; sfail=1; }
  [ "$mb" = "$(shasum -a 256 "$mlog" | cut -d' ' -f1)" ] || { echo "check_v3_compat: $name events.jsonl bytes changed" >&2; sfail=1; }
  [ "$sfail" -eq 0 ] || mfail=1
  mres="$(jq -c --arg n "$name" --argjson docs "$ndocs" --argjson blame "$ok_blame" --arg h "$mh_old" --arg h2 "$mh_new" \
    --argjson ok "$([ "$sfail" -eq 0 ] && echo true || echo false)" \
    '. + [{scenario: $n, docs: $docs, blame_equal: $blame, events_hash_v3_final: $h, events_hash_current: $h2, pass: $ok}]' <<<"$mres")"
done < <(jq -r '.scenarios[].name' "$scen")
mresult="$(jq -n --arg sha "$sha" --argjson s "$mres" \
  '{v3_final_sha: $sha, scenarios: $s, passed: ([$s[] | select(.pass)] | length), total: ($s | length), pass: ([$s[] | select(.pass | not)] | length == 0)}')"
[ -z "$mixed_out" ] || echo "$mresult" >"$mixed_out"
if [ "$mfail" -ne 0 ]; then
  echo "check_v3_compat: mixed-author vaults FAILED" >&2
  exit 1
fi
echo "check_v3_compat: OK (mixed-author vaults $(jq -r '"\(.passed)/\(.total)"' <<<"$mresult"))" >&2
