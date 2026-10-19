#!/usr/bin/env bash
# X4 (SPEC_EDITS "Sürüm", X3 review R8): a vault written by the current
# build (spec 2.0.2) with carried `unverified` and `proposal` blocks, opened
# by the 21b16ea build (spec 2.0.1): events-hash, replay and blame of every
# document must succeed and agree. Information, not a hypothesis.
set -euo pipefail
NEW=/Users/arkns/Works/mdkernel/.claude/worktrees/p2-0.2-round/target/release/rootmark
OLD=/Users/arkns/.claude/jobs/93286294/tmp/b21/target/release/rootmark
export ROOTMARK_MODELS=/Users/arkns/Works/mdkernel/.claude/worktrees/p2-0.2-round/models
w="$(mktemp -d -t rootmark-reverse.XXXXXX)"
trap 'kill "${pid:-}" 2>/dev/null || true' EXIT
v="$w/vault"; mkdir -p "$v"
printf '# B\n\nHuman one about the roast.\n\nHuman two, see [[b]].\n' >"$v/b.md"
printf '# A\n\nHuman text, see [b](b.md).\n' >"$v/a.md"
"$NEW" ingest-dir "$v" "$v" >/dev/null
"$NEW" serve --vault "$v" --mcp stdio </dev/null >/dev/null 2>&1
port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')"
p="$v/.rootmark/policy.toml"
sed -i.bak 's/^# \[identity\]$/[identity]/; s/^# human_id = .*$/human_id = "editor"/' "$p"
sed -i.bak "s/^\[server\]$/[server]\nhttp_port = $port/" "$p"; rm -f "$p.bak"
printf '\n[attestation]\nunattested = "unverified"\nwindow_s = 1.0\n' >>"$p"
"$NEW" serve --vault "$v" --http 127.0.0.1 --watch 2>"$w/serve.err" &
pid=$!
for _ in $(seq 1 600); do grep -q "watching" "$w/serve.err" 2>/dev/null && break; sleep 0.1; done
# an agent paragraph written to b.md (unverified)
printf '\nAgent paragraph written to the file.\n' >>"$v/b.md"; sleep 2
# an agent proposal (CLI propose: a new note of proposals)
printf '# Plan\n\nAn agent plan.\n' >"$w/plan.txt"
"$NEW" propose --source agent-x "$v" plan.md "$w/plan.txt" >/dev/null; sleep 1
# the human renames both (mv + attest --renamed-from): layers carried
mv "$v/b.md" "$v/bee.md"; "$NEW" attest "$v" bee.md --renamed-from b.md >/dev/null; sleep 2
mv "$v/plan.md" "$v/plan2.md"; "$NEW" attest "$v" plan2.md --renamed-from plan.md >/dev/null; sleep 2
kill -TERM "$pid"; wait "$pid" || true; pid=""
for d in bee.md plan2.md; do
  echo "== $d (current build)"; "$NEW" blame "$v" "$d" | jq -c '[(.blocks // .)[] | .layer]'
done
echo "== opened by 21b16ea (spec $("$OLD" --spec-version))"
a="$("$NEW" events-hash "$v" | jq -r .events_hash)"; b="$("$OLD" events-hash "$v" | jq -r .events_hash)"
echo "events_hash current=$a old=$b equal=$([ "$a" = "$b" ] && echo yes || echo no)"
for d in a.md b.md bee.md plan.md plan2.md; do
  x="$("$NEW" replay "$v" "$d" | jq -cS .)"; y="$("$OLD" replay "$v" "$d" | jq -cS .)"
  l1="$("$NEW" blame "$v" "$d" | jq -c '[(.blocks // .)[] | [.lineage_id, .layer]]')"
  l2="$("$OLD" blame "$v" "$d" | jq -c '[(.blocks // .)[] | [.lineage_id, .layer]]')"
  echo "$d replay_equal=$([ "$x" = "$y" ] && echo yes || echo no) layers_equal=$([ "$l1" = "$l2" ] && echo yes || echo no)"
done
rm -rf "$w"
