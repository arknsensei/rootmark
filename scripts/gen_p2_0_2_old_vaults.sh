#!/usr/bin/env bash
# D-E of P2-0.2 (GOAL P2-0.2 "Veri setleri"; ASSUMPTIONS X0-11): one vault
# written by the build of P2-0.1's official RUN (`21b16ea`, spec 2.0.1) whose
# attestation log holds `partial: true` and `matched: false` lines, committed
# under research/data/p2_0_2_old_vaults/<name>/ as `vault/` (the Markdown
# files), `store/` (the vault's `.rootmark/`, no token, no index) and
# `expected.json` (what `21b16ea` itself reports: documents, `events-hash`,
# the BLAKE3 of `attestations.jsonl`, `replay` and `blame` layers of every
# document) — the layout of P2-0.1's `gen_p2_0_1_old_vaults.sh`.
# `eval.run_p2_0_2` opens it with the current build next to P2-0's three
# `v0.1.0` and P2-0.1's two `ce4cf04` vaults (H-P2-0.2h, 6/6).
#   - partial: `rootmark serve --http --watch` (default policy `unverified`,
#     W = 2 s): a human save attested with its bytes (`content`) and an
#     agent's paragraph appended to the same note before the watcher looked
#     (one observation: a partial match, `partial: true`); an attestation of
#     bytes the file never held (`matched: false`); a save attested late
#     (raised); a rename attested with `renamed_from` (`rootmark attest`); a
#     named checkpoint.
#
#   scripts/gen_p2_0_2_old_vaults.sh <rootmark binary built from 21b16ea>
set -Eeuo pipefail
trap 'echo "gen_p2_0_2_old_vaults: failed at line $LINENO: $BASH_COMMAND" >&2' ERR
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
B="${1:?usage: gen_p2_0_2_old_vaults.sh <rootmark binary built from 21b16ea>}"
[ "$("$B" --spec-version)" = "2.0.1" ] || { echo "gen_p2_0_2_old_vaults: $B does not write spec 2.0.1" >&2; exit 1; }
out="$root/research/data/p2_0_2_old_vaults"
export ROOTMARK_MODELS="${ROOTMARK_MODELS:-$root/models}"
work="$(mktemp -d -t rootmark-p2-0-2-old.XXXXXX)"
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
h() { curl -sS -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: p2-0-2-old' -H 'Content-Type: application/json' --data-binary "$2" "http://127.0.0.1:$port/v1/$1"; }
settle() { sleep 2.6; } # past W: every pending change decided
stop() { kill -TERM "$pid"; wait "$pid" || true; pid=""; }
# token_attest <doc> <hash> [<content file>]: the plugin's protocol (session, MAC)
token_attest() {
  local doc="$1" hash="$2" body s key sid mac
  s="$(h attest_session '{}')"
  key="$(echo "$s" | jq -r .key)"; sid="$(echo "$s" | jq -r .session_id)"
  mac="$(printf 'rootmark-attest/1\n%s\n%s\n' "$doc" "$hash" | openssl dgst -sha256 -mac HMAC -macopt "hexkey:$key" | awk '{print $NF}')"
  body="$(jq -n --arg d "$doc" --arg c "$hash" --arg s "$sid" --arg m "$mac" '{doc_path: $d, content_hash: $c, session_id: $s, mac: $m}')"
  if [ -n "${3:-}" ]; then body="$(echo "$body" | jq --rawfile t "$3" '. + {content: $t}')"; fi
  h attest "$body"
}

finish() { # <name> <vault>
  local name="$1" v="$2" d="$out/$1"
  mkdir -p "$d"
  docs="$(jq -r 'select(.doc_path != "" and .kind == "SnapshotCommitted") | .doc_path' "$v/.rootmark/events.jsonl" | LC_ALL=C sort -u)"
  {
    echo '{"vault": "'"$name"'", "written_by": "'"$("$B" --version)"' (21b16ea)", "spec_version": "'"$("$B" --spec-version)"'",'
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

# ---- partial
v="$work/partial"; mkdir -p "$v"
for f in roasting espresso filter glossary staff; do cp "$root/spec/fixtures/mini_vault/$f.md" "$v/"; done
"$B" ingest-dir "$v" "$v" >/dev/null
serve "$v"
# the human's save and at once the agent's paragraph (one observation),
# then the human's attestation with its bytes: a partial match (early or
# late, both write `partial: true`)
cp "$v/staff.md" "$work/saved.md"
printf '\nA human paragraph saved and attested with its bytes.\n' >>"$work/saved.md"
saved_hash="$(b3 "$work/saved.md")"
cp "$work/saved.md" "$v/staff.md"
printf '\nAn agent paragraph appended before the watcher looked.\n' >>"$v/staff.md"
token_attest staff.md "$saved_hash" "$work/saved.md" >/dev/null
settle
grep -q '"partial":true' "$v/.rootmark/attestations.jsonl" \
  || { echo "gen_p2_0_2_old_vaults: no partial match (the watcher saw the two writes apart); run again" >&2; exit 1; }
# bytes the file never held: recorded, marked matched false
printf 'never written\n' >"$work/never.md"
token_attest glossary.md "$(b3 "$work/never.md")" >/dev/null
grep -q '"matched":false' "$v/.rootmark/attestations.jsonl"
settle
# a save attested only after the watcher saw it (late raise)
printf '\nA human paragraph attested only after the watcher saw it.\n' >>"$v/filter.md"
sleep 0.6
"$B" attest "$v" filter.md >/dev/null
settle
# a rename (`mv` + `rootmark attest --renamed-from`)
mv "$v/espresso.md" "$v/coffee-espresso.md"
"$B" attest "$v" coffee-espresso.md --renamed-from espresso.md >/dev/null
settle
h checkpoint '{"name":"p2-0-2-old"}' >/dev/null
stop
finish partial "$v"

echo "gen_p2_0_2_old_vaults: $(ls "$out" | tr '\n' ' ')written by $("$B" --version) (21b16ea)" >&2
