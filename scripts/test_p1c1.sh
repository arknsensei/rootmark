#!/usr/bin/env bash
# P1-C.1 deliberate tests through the CLI (GOAL C1-4..C1-6; K-C1.1, K-C1.4,
# K-C1.5). Each case must behave as stated or the script exits 1.
#
#   scripts/test_p1c1.sh [<out.json>]
#
# Vault: spec/fixtures/mini_vault + one `propose`d agent document,
# checkpoint t0, `rootmark index build`.
# 1. C1-4 read-only search: a copy of `.rootmark/index/` is taken; searches
#    in every mode, with and without --include-proposal, at t0 and at the
#    head, and `index status` → `diff -r` with the copy (hidden files, the
#    tantivy `.tantivy-meta.lock` included) shows no difference and no
#    `*.lock` exists. Control: a `.tantivy-meta.lock` placed by hand IS
#    reported by the same `diff -r` (the comparison sees lock files).
# 2. C1-5 empty query: "", " ", tab/newline, U+3000 → `rootmark search` exits
#    non-zero with "empty query", for every mode; `traces.jsonl`,
#    `events.jsonl` and `checkpoints/` unchanged. Control: a non-empty
#    query writes one trace line.
# 3. C1-6 manifest: removing each of lexical.{variant,idf,k1,b,tokenizer},
#    removing `lexical`, or changing k1 → `rootmark index status` and
#    `rootmark search` exit non-zero; the restored manifest opens.
# Nothing in the source tree is modified. With <out.json>, a summary is
# written there.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
out="${1:-}"
work="$(mktemp -d -t rootmark-p1c1.XXXXXX)"
trap 'rm -rf "$work"' EXIT
cargo build -q --release -p rootmark-server --bin rootmark -p rootmark-index --bin rootmark-index
rootmark="$root/target/release/rootmark"
n=0 ok=0
check() { # check <label> <0|1 = expected success|failure> <cmd...>
  local label="$1" want="$2"; shift 2
  n=$((n + 1))
  local rc=0
  "$@" >"$work/out.$n" 2>&1 || rc=$?
  if { [ "$want" = 0 ] && [ "$rc" = 0 ]; } || { [ "$want" = 1 ] && [ "$rc" != 0 ]; }; then
    ok=$((ok + 1))
    echo "p1c1: ok: $label$( [ "$rc" = 0 ] || echo " (exit $rc)") $(grep -m1 -iE 'error' "$work/out.$n" | cut -c1-140 || true)"
  else
    echo "p1c1: WRONG: $label (exit $rc)" >&2
    cat "$work/out.$n" >&2
  fi
}

v="$work/vault"
"$rootmark" ingest-dir "$v" spec/fixtures/mini_vault >/dev/null
cat >"$work/agent.md" <<'EOF'
# Agent espresso notes

## Grind

The espresso grind should be finer when shots run fast; dial in with small steps.
EOF
"$rootmark" propose "$v" agent/notes.md "$work/agent.md" --source claude-code >/dev/null
"$rootmark" checkpoint create "$v" --name t0 >/dev/null
"$rootmark" index build "$v" >/dev/null
idx="$v/.rootmark/index"

# ---- 1. C1-4
cp -R "$idx" "$work/index.copy"
for mode in flat lexical hybrid; do
  for flag in "" "--include-proposal"; do
    check "search $mode ${flag:-default} at t0" 0 "$rootmark" search "$v" "espresso grind" --mode "$mode" --at t0 $flag
    check "search $mode ${flag:-default} at head" 0 "$rootmark" search "$v" "espresso grind" --mode "$mode" $flag
  done
done
check "index status" 0 "$rootmark" index status "$v"
check "C1-4 diff -r index after searches: equal" 0 diff -r "$idx" "$work/index.copy"
locks="$(find "$idx" -name '*.lock' | wc -l | tr -d ' ')"
check "C1-4 no lock file in the index ($locks)" 0 test "$locks" = 0
cp -R "$idx" "$work/index.lock"
: >"$work/index.lock/lexical/.tantivy-meta.lock"
check "C1-4 control: diff -r reports a lock file" 1 diff -r "$idx" "$work/index.lock"

# ---- 2. C1-5
s="$v/.rootmark"
before="$(cat "$s/traces.jsonl" "$s/events.jsonl" | shasum -a 256)/$(ls "$s/checkpoints" | wc -l)"
for q in "" " " "$(printf '\t\n ')" "$(printf '\343\200\200')"; do
  for mode in flat lexical hybrid; do
    check "C1-5 empty query $(printf %q "$q") $mode" 1 "$rootmark" search "$v" "$q" --mode "$mode"
    grep -q "empty query" "$work/out.$n" || { echo "p1c1: WRONG: no 'empty query' error" >&2; ok=$((ok - 1)); }
  done
done
after="$(cat "$s/traces.jsonl" "$s/events.jsonl" | shasum -a 256)/$(ls "$s/checkpoints" | wc -l)"
check "C1-5 traces, events, checkpoints unchanged" 0 test "$before" = "$after"
t0="$(wc -l <"$s/traces.jsonl" | tr -d " ")"
check "C1-5 control: a query searches" 0 "$rootmark" search "$v" "espresso" --at t0
check "C1-5 control: one trace line" 0 test "$(wc -l <"$s/traces.jsonl" | tr -d " ")" = "$((t0 + 1))"

# ---- 3. C1-6
m="$idx/manifest.json"
cp "$m" "$work/manifest.orig"
# P1-D.1/F14 (K-D1.7): the record also declares the document length (`dl`)
check "C1-6 manifest holds lexical.{variant,idf,k1,b,tokenizer,dl}" 0 \
  jq -e '.lexical | keys == ["b","dl","idf","k1","tokenizer","variant"]' "$m"
for f in variant idf k1 b tokenizer; do
  jq "del(.lexical.$f)" "$work/manifest.orig" >"$m"
  check "C1-6 lexical.$f removed: index status" 1 "$rootmark" index status "$v"
  check "C1-6 lexical.$f removed: search" 1 "$rootmark" search "$v" espresso --at t0
done
jq 'del(.lexical)' "$work/manifest.orig" >"$m"
check "C1-6 lexical removed: index status" 1 "$rootmark" index status "$v"
jq '.lexical.k1 = 1.5' "$work/manifest.orig" >"$m"
check "C1-6 lexical.k1 changed: index status" 1 "$rootmark" index status "$v"
cp "$work/manifest.orig" "$m"
check "C1-6 restored manifest: index status" 0 "$rootmark" index status "$v"

echo "p1c1: $ok/$n cases as expected"
if [ -n "$out" ]; then
  jq -n --argjson ok "$ok" --argjson n "$n" '{cases: $n, as_expected: $ok, pass: ($ok == $n)}' >"$out"
fi
[ "$ok" -eq "$n" ] || exit 1
