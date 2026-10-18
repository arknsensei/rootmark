#!/usr/bin/env bash
# D-E of P2-0 (GOAL P2-0 "Veri setleri"; TASKS Z5): three vaults written by
# `rootmark 0.1.0` (spec 1.0.3) — the binary built from the `v0.1.0` tag —
# committed under research/data/p2_0_old_vaults/<name>/ as `vault/` (the
# Markdown files), `store/` (the vault's `.rootmark/`: `.rootmark/` dirs of
# fixtures are not committed; no token, no index) and `expected.json`
# (what 0.1.0 itself reports: the documents, `events-hash`, `replay` and
# `blame` layers of every document). `eval.run_p2_0` opens them with the
# current build (H-P2-0f).
#   - scene: the product scene through the CLI — the mini vault ingested
#     (human), an agent proposal promoted by the human, a named checkpoint,
#     a second proposal appended under its own heading, a checkpoint;
#   - e2e: `rootmark serve --http --watch` — a human edit outside the server
#     (the watcher), an HTTP proposal, a promotion, a search (trace), a named
#     checkpoint, the shutdown compaction;
#   - mixed: the v3-final-format log of core/tests/fixtures/v3_final_mixed
#     (no layer information) extended by 0.1.0 (a human document, an agent
#     proposal, a checkpoint): both formats in one log.
#
#   scripts/gen_p2_0_old_vaults.sh <rootmark-0.1.0 binary>
set -Eeuo pipefail
trap 'echo "gen_p2_0_old_vaults: failed at line $LINENO: $BASH_COMMAND" >&2' ERR
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
B="${1:?usage: gen_p2_0_old_vaults.sh <rootmark-0.1.0 binary>}"
[ "$("$B" --version)" = "rootmark 0.1.0" ] || { echo "gen_p2_0_old_vaults: $B is not rootmark 0.1.0" >&2; exit 1; }
out="$root/research/data/p2_0_old_vaults"
# the embedder model of this tree (a binary built elsewhere looks in its own tree)
export ROOTMARK_MODELS="${ROOTMARK_MODELS:-$root/models}"
work="$(mktemp -d -t rootmark-p2-0-old.XXXXXX)"
trap 'kill "${pid:-}" 2>/dev/null || true; rm -rf "$work"' EXIT
rm -rf "$out"
mkdir -p "$out"

recipe() { # <vault> [port]: the default recipe, human id `editor`
  "$B" serve --vault "$1" --mcp stdio </dev/null >/dev/null 2>&1
  local p="$1/.rootmark/policy.toml"
  sed -i.bak 's/^# \[identity\]$/[identity]/; s/^# human_id = .*$/human_id = "editor"/' "$p"
  [ -z "${2:-}" ] || sed -i.bak "s/^\[server\]$/[server]\nhttp_port = $2/" "$p"
  rm -f "$p.bak"
}

finish() { # <name> <vault>: expected.json from 0.1.0, then store/
  local name="$1" v="$2" d="$out/$1"
  mkdir -p "$d"
  docs="$(jq -r 'select(.doc_path != "" and .kind == "SnapshotCommitted") | .doc_path' "$v/.rootmark/events.jsonl" | LC_ALL=C sort -u)"
  {
    echo '{"vault": "'"$name"'", "written_by": "'"$("$B" --version)"'", "spec_version": "'"$("$B" --spec-version)"'",'
    echo '"events_hash": '"$("$B" events-hash "$v" | jq .events_hash)"','
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

# ---- scene
v="$work/scene"; mkdir -p "$v"; cp "$root"/spec/fixtures/mini_vault/*.md "$v/"
"$B" ingest-dir "$v" "$v" >/dev/null
recipe "$v"
printf '# Plan\n\nAn agent recommends a coarser grind for the kalimba roast.\n' >"$work/plan.md"
"$B" propose "$v" agent/plan.md "$work/plan.md" --source scene-agent/1.0 >/dev/null
lin="$("$B" blame "$v" agent/plan.md | jq -r '(.blocks // .)[-1].lineage_id')"
"$B" promote "$v" "$lin" --actor human:editor --reason "approved in the scene" >/dev/null
"$B" checkpoint create "$v" --name before >/dev/null
printf '## Agent note\n\nAn agent adds a note about the cooling tray.\n' >"$work/note.md"
"$B" propose "$v" roasting.md "$work/note.md" --source scene-agent/1.0 --mode append >/dev/null
"$B" checkpoint create "$v" >/dev/null
finish scene "$v"

# ---- e2e (a served vault)
v="$work/e2e"; mkdir -p "$v"; cp "$root"/spec/fixtures/mini_vault/*.md "$v/"
"$B" ingest-dir "$v" "$v" >/dev/null
port="$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')"
recipe "$v" "$port"
"$B" serve --vault "$v" --http 127.0.0.1 --watch 2>"$work/serve.err" &
pid=$!
for _ in $(seq 1 600); do grep -q "watching" "$work/serve.err" 2>/dev/null && break; sleep 0.1; done
tok="$(tr -d '\n' <"$v/.rootmark/server.token")"
h() { curl -sS -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: p2-0-old' -H 'Content-Type: application/json' --data-binary "$2" "http://127.0.0.1:$port/v1/$1"; }
printf '\nA human paragraph added outside the server about the drum speed.\n' >>"$v/roasting.md"
for _ in $(seq 1 50); do sleep 0.1; [ "$(h blame '{"doc_path":"roasting.md"}' | jq '.blocks | length')" -gt 0 ] && grep -q "watch: roasting.md ingested" "$work/serve.err" && break; done
lin="$(h propose '{"doc_path":"agent/e2e.md","content":"# E2E\n\nAn agent drafted this over HTTP.","reason":"p2-0 old vault"}' | jq -r '.blocks[-1].lineage_id')"
h promote "{\"lineage_id\":\"$lin\",\"reason\":\"ok\"}" >/dev/null
h search '{"query":"drum speed"}' >/dev/null
h checkpoint '{"name":"e2e"}' >/dev/null
kill -TERM "$pid"; wait "$pid" || true; pid=""
finish e2e "$v"

# ---- mixed (a v3-final log extended by 0.1.0)
v="$work/mixed"; mkdir -p "$v/.rootmark"
cp -R "$root/core/tests/fixtures/v3_final_mixed/mixed_vault/store/." "$v/.rootmark/"
printf '# Notes\n\nA human note written with 0.1.0 on a v3-final log.\n' >"$v/notes.md"
"$B" ingest "$v" "$v/notes.md" >/dev/null
recipe "$v"
printf '# Agent\n\nAn agent proposal on the mixed log.\n' >"$work/agent.md"
"$B" propose "$v" agent/mixed.md "$work/agent.md" --source mixed-agent/1.0 >/dev/null
"$B" checkpoint create "$v" >/dev/null
finish mixed "$v"

echo "gen_p2_0_old_vaults: $(ls "$out" | tr '\n' ' ')written by $("$B" --version)" >&2
