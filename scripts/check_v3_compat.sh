#!/usr/bin/env bash
# B-5 (GOAL P1-B; VERIFY V3): a vault written by the **v3-final** `mdk` binary
# (events without layers) opens with the current core: `replay` gives the same
# snapshot_id for every document, `events_hash` and the events.jsonl bytes do
# not change, and `blame` derives every layer from `author_kind`.
#
#   scripts/check_v3_compat.sh [<out.json>]
#
# Builds `git archive v3-final core` (like scripts/gen_fixtures.sh) into a
# temp dir; the current `mdk` is `target/release/mdk` of this tree (built here).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-}"
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
