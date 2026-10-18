#!/usr/bin/env bash
# D-E of P2-0.1 (GOAL P2-0.1 "Veri setleri"; ASSUMPTIONS Y0-9): two vaults
# written by the P2-0 build verified independently (`ce4cf04`, spec 2.0.0),
# with attestation logs, committed under research/data/p2_0_1_old_vaults/
# <name>/ as `vault/` (the Markdown files), `store/` (the vault's
# `.rootmark/`, no token, no index) and `expected.json` (what `ce4cf04`
# itself reports: documents, `events-hash`, the BLAKE3 of
# `attestations.jsonl`, `replay` and `blame` layers of every document).
# `eval.run_p2_0_1` opens them with the current build next to P2-0's three
# `v0.1.0` vaults (H-P2-0.1h, 5/5).
#   - attested: `rootmark serve --http --watch` (default policy
#     `unverified`, W = 2 s): a human save attested before the watcher sees
#     it, an agent's direct write (unverified), a save attested late (raised),
#     the unverified lineage promoted by the human, a rename attested with
#     `renamed_from`, a named checkpoint;
#   - bulk: the same server: three quick attested saves, an agent overwrite
#     of a human note (unverified, actor `unattested_fs`), an agent rename
#     (unverified new path), a bulk attestation (`attest --all`, late false),
#     a checkpoint.
#
#   scripts/gen_p2_0_1_old_vaults.sh <rootmark binary built from ce4cf04>
set -Eeuo pipefail
trap 'echo "gen_p2_0_1_old_vaults: failed at line $LINENO: $BASH_COMMAND" >&2' ERR
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
B="${1:?usage: gen_p2_0_1_old_vaults.sh <rootmark binary built from ce4cf04>}"
[ "$("$B" --spec-version)" = "2.0.0" ] || { echo "gen_p2_0_1_old_vaults: $B does not write spec 2.0.0" >&2; exit 1; }
out="$root/research/data/p2_0_1_old_vaults"
export ROOTMARK_MODELS="${ROOTMARK_MODELS:-$root/models}"
work="$(mktemp -d -t rootmark-p2-0-1-old.XXXXXX)"
trap 'kill "${pid:-}" 2>/dev/null || true; rm -rf "$work"' EXIT
rm -rf "$out"
mkdir -p "$out"
b3() { (cd "$root/research" && uv run --frozen python -c 'import sys, blake3; print(blake3.blake3(open(sys.argv[1], "rb").read()).hexdigest()[:32])' "$1"); }

serve() { # <vault>: recipe (human id, port, W 2 s), then the server in the background
  local v="$1"
  "$B" serve --vault "$v" --mcp stdio </dev/null >/dev/null 2>&1
  port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')"
  local p="$v/.rootmark/policy.toml"
  sed -i.bak 's/^# \[identity\]$/[identity]/; s/^# human_id = .*$/human_id = "editor"/' "$p"
  sed -i.bak "s/^\[server\]$/[server]\nhttp_port = $port/" "$p"
  rm -f "$p.bak"
  printf '\n[attestation]\nunattested = "unverified"\nwindow_s = 2.0\n' >>"$p"
  "$B" serve --vault "$v" --http 127.0.0.1 --watch 2>"$work/serve.err" &
  pid=$!
  for _ in $(seq 1 600); do grep -q "watching" "$work/serve.err" 2>/dev/null && break; sleep 0.1; done
  grep -q "watching" "$work/serve.err"
  tok="$(tr -d '\n' <"$v/.rootmark/server.token")"
}
h() { curl -sS -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: p2-0-1-old' -H 'Content-Type: application/json' --data-binary "$2" "http://127.0.0.1:$port/v1/$1"; }
settle() { sleep 2.6; } # past W: every pending change decided
stop() { kill -TERM "$pid"; wait "$pid" || true; pid=""; }

finish() { # <name> <vault>
  local name="$1" v="$2" d="$out/$1"
  mkdir -p "$d"
  docs="$(jq -r 'select(.doc_path != "" and .kind == "SnapshotCommitted") | .doc_path' "$v/.rootmark/events.jsonl" | LC_ALL=C sort -u)"
  {
    echo '{"vault": "'"$name"'", "written_by": "'"$("$B" --version)"' (ce4cf04)", "spec_version": "'"$("$B" --spec-version)"'",'
    echo '"events_hash": '"$("$B" events-hash "$v" | jq .events_hash)"','
    echo '"attestations_blake3": "'"$(b3 "$v/.rootmark/attestations.jsonl")"'",'
    echo '"attestation_lines": '"$(wc -l <"$v/.rootmark/attestations.jsonl" | tr -d ' ')"','
    echo '"docs": '"$(printf '%s\n' "$docs" | jq -R . | jq -s .)"','
    echo '"replay": {'
    first=true
    while read -r doc; do
      $first || echo ','; first=false
      echo "$(jq -n --arg d "$doc" '$d'): $("$B" replay "$v" "$doc")"
    done <<<"$docs"
    echo '}, "layers": {'
    first=true
    while read -r doc; do
      $first || echo ','; first=false
      echo "$(jq -n --arg d "$doc" '$d'): $("$B" blame "$v" "$doc" | jq -c '[(.blocks // .)[] | [.lineage_id, .layer]]')"
    done <<<"$docs"
    echo '}}'
  } | jq -S . >"$d/expected.json"
  rm -rf "$v/.rootmark/index" "$v/.rootmark/server.token"
  mv "$v/.rootmark" "$d/store"
  mkdir -p "$d/vault"
  (cd "$v" && find . -name '*.md' -print0 | tar --null -T - -cf - | tar -xf - -C "$d/vault")
}

# ---- attested
v="$work/attested"; mkdir -p "$v"
for f in roasting espresso filter glossary grinders; do cp "$root/spec/fixtures/mini_vault/$f.md" "$v/"; done
"$B" ingest-dir "$v" "$v" >/dev/null
serve "$v"
printf '\nA human paragraph saved and attested before the watcher looked.\n' >>"$v/roasting.md"
"$B" attest "$v" roasting.md >/dev/null
settle
printf '\nAn agent paragraph written straight to the file.\n' >>"$v/espresso.md"
settle
printf '\nA human paragraph attested only after the watcher saw it.\n' >>"$v/filter.md"
sleep 0.6
"$B" attest "$v" filter.md >/dev/null
settle
lin="$(h blame '{"doc_path":"espresso.md"}' | jq -r '[.blocks[] | select(.layer == "unverified")][0].lineage_id')"
[ -n "$lin" ] && [ "$lin" != null ]
h promote "{\"lineage_id\":\"$lin\",\"reason\":\"mine\"}" | jq -e '.lineage_id' >/dev/null
sid="$(h attest_session '{}')"
key="$(echo "$sid" | jq -r .key)"; sid="$(echo "$sid" | jq -r .session_id)"
mv "$v/glossary.md" "$v/terms.md"
hash="$(b3 "$v/terms.md")"
mac="$(printf 'rootmark-attest/1\n%s\n%s\n' terms.md "$hash" | openssl dgst -sha256 -mac HMAC -macopt "hexkey:$key" | awk '{print $NF}')"
h attest "{\"doc_path\":\"terms.md\",\"content_hash\":\"$hash\",\"session_id\":\"$sid\",\"mac\":\"$mac\",\"renamed_from\":\"glossary.md\"}" | jq -e '.attestation_id' >/dev/null
settle
h checkpoint '{"name":"p2-0-1-old"}' >/dev/null
stop
finish attested "$v"

# ---- bulk
v="$work/bulk"; mkdir -p "$v"
for f in maintenance origins safety staff; do cp "$root/spec/fixtures/mini_vault/$f.md" "$v/"; done
"$B" ingest-dir "$v" "$v" >/dev/null
serve "$v"
for i in 1 2 3; do
  printf '\nQuick human line %s.\n' "$i" >>"$v/staff.md"
  "$B" attest "$v" staff.md >/dev/null
  sleep 0.4
done
settle
printf '# Safety\n\nAn agent rewrote the whole note.\n' >"$v/safety.md"
settle
mv "$v/origins.md" "$v/agent-origins.md"
settle
"$B" attest "$v" --all >/dev/null
settle
h checkpoint '{}' >/dev/null
stop
finish bulk "$v"

echo "gen_p2_0_1_old_vaults: $(ls "$out" | tr '\n' ' ')written by $("$B" --version) (ce4cf04)" >&2
