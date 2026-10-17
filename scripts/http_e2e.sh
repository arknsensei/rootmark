#!/usr/bin/env bash
# HTTP API and service tests (GOAL P1-D D-7, D-10; TASKS D2).
#
#   scripts/http_e2e.sh <out-dir>
#
# Vault: a copy of spec/fixtures/mini_vault ingested in place (`rootmark
# ingest-dir vault vault`), no policy.toml and no
# index (the server writes the default recipe and builds the index).
# `rootmark serve --vault vault --http 127.0.0.1:0 --watch` (cwd <out-dir>; port
# read from its stderr line "http listening on 127.0.0.1:PORT").
# D-7 (5 cases, curl):
#   1. no Authorization → 401; 2. wrong token → 401; 3. right token +
#   X-Consumer → 200 (`status`); 4. right token, no X-Consumer → 400 (and
#   empty X-Consumer → 400); 5. external interface refused: `--http
#   0.0.0.0:PORT` and `--http <first non-loopback IPv4>:PORT` exit 1 without
#   listening, and the running server's socket is bound to 127.0.0.1 only
#   (lsof).
# Token file: created with mode 0600; a token file with mode 0644 makes the
# server refuse to start.
# Watch (D-10, function): a human edit outside the server → the server
# ingests it (`blame`: new block canonical, author Human, source
# rootmark-serve-watch) within 1 s; `rootmark index status` → no stale document, hot
# tier non-empty. (Latency over many edits: server/tests/serve_watch.rs.)
# Hidden entries (`.obsidian/h.md`, `.dotfile.md`, `sub/.x/y.md`) and
# non-.md files are not ingested by the watcher.
# SIGTERM: exit 0, stderr "compact (shutdown)", hot tier gone (`index
# status` hot_records 0), and the edit still found by `rootmark search` (cold).
# Timer: with `[index] compact_every_s = 1`, a watched edit leaves the hot
# tier within 5 s while the server runs ("compact (timer)").
# Result: <out-dir>/http_e2e.json; exit 1 if any case fails.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: scripts/http_e2e.sh <out-dir>}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
cd "$root"
cargo build -q --release -p rootmark-server --bin rootmark
rootmark="$root/target/release/rootmark"
cd "$out"
[ ! -e vault ] || { echo "http_e2e: $out/vault exists" >&2; exit 1; }
mkdir vault
cp "$root"/spec/fixtures/mini_vault/*.md vault/
"$rootmark" ingest-dir vault vault >/dev/null

results="[]"
record() { # record <id> <label> <true|false> [detail]
  results="$(jq -c --arg id "$1" --arg l "$2" --argjson p "$3" --arg d "${4:-}" \
    '. + [{id: $id, case: $l, pass: $p, detail: $d}]' <<<"$results")"
  if [ "$3" = true ]; then echo "http_e2e: ok   $1 $2"; else echo "http_e2e: FAIL $1 $2 ${4:-}" >&2; fi
}

start() { # start <stderr-file> <args…>: background server, waits for the port
  local err="$1"; shift
  "$rootmark" serve --vault vault "$@" 2>"$err" &
  pid=$!
  for _ in $(seq 1 600); do
    if grep -q "http listening on" "$err" 2>/dev/null; then
      port="$(sed -n 's/.*http listening on 127\.0\.0\.1:\([0-9][0-9]*\).*/\1/p' "$err")"
      return 0
    fi
    kill -0 "$pid" 2>/dev/null || { echo "http_e2e: server exited:" >&2; cat "$err" >&2; return 1; }
    sleep 0.1
  done
  echo "http_e2e: server did not start" >&2; cat "$err" >&2; return 1
}

code() { curl -s -o "$out/body.json" -w '%{http_code}' "$@"; }

# a refused start writes nothing (no policy.toml, token or index yet)
rc=0
"$rootmark" serve --vault vault --http 0.0.0.0:0 2>refused.err || rc=$?
ok=false
if [ "$rc" != 0 ] && [ ! -e vault/.rootmark/policy.toml ] && [ ! -e vault/.rootmark/server.token ] && [ ! -e vault/.rootmark/index ]; then ok=true; fi
record R-1 "refused start (--http 0.0.0.0) writes no policy, token or index" "$ok" "exit $rc"

start serve.err --http 127.0.0.1:0 --watch
base="http://127.0.0.1:$port/v1"
tokfile=vault/.rootmark/server.token
tok="$(tr -d '\n' <"$tokfile")"
mode="$(stat -f '%Lp' "$tokfile" 2>/dev/null || stat -c '%a' "$tokfile")"
record T-1 "token file created, mode 600, 64 hex" \
  "$( [ "$mode" = 600 ] && [[ "$tok" =~ ^[0-9a-f]{64}$ ]] && echo true || echo false)" "mode $mode"
record T-2 "default recipe written (promotion human_only)" \
  "$(grep -q 'policy = "human_only"' vault/.rootmark/policy.toml && echo true || echo false)"

c="$(code -H 'X-Consumer: http-e2e' "$base/status")"
record D7-1 "no token → 401" "$( [ "$c" = 401 ] && echo true || echo false)" "$c"
c="$(code -H "Authorization: Bearer ${tok%?}0" -H 'X-Consumer: http-e2e' "$base/status")"
[ "${tok: -1}" != 0 ] || c="$(code -H "Authorization: Bearer ${tok%?}1" -H 'X-Consumer: http-e2e' "$base/status")"
record D7-2 "wrong token → 401" "$( [ "$c" = 401 ] && echo true || echo false)" "$c"
c="$(code -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' "$base/status")"
ok=false
if [ "$c" = 200 ] && jq -e '.spec_version == "1.0.2" and .policy.promotion == "human_only" and .index == "present"' body.json >/dev/null; then ok=true; fi
record D7-3 "right token → 200 (status: human_only, index built)" "$ok" "$c"
c1="$(code -H "Authorization: Bearer $tok" "$base/status")"
c2="$(code -H "Authorization: Bearer $tok" -H 'X-Consumer: ' "$base/status")"
c3="$(code -X POST -H "Authorization: Bearer $tok" -d '{}' "$base/checkpoints")"
record D7-4 "no / empty X-Consumer → 400" \
  "$( [ "$c1" = 400 ] && [ "$c2" = 400 ] && [ "$c3" = 400 ] && echo true || echo false)" "$c1 $c2 $c3"

# 5. external interface: refused before listening; the running socket is loopback only
ext_ok=true
detail=""
lan="$(ifconfig 2>/dev/null | awk '/inet / && $2 != "127.0.0.1" {print $2; exit}')"
# (the detail names the address classes, never the port or this machine's
# LAN address, so two runs record the same text)
for a in "0.0.0.0:0" ${lan:+"$lan:0"} "[::]:0" "[::1]:0"; do
  rc=0
  "$rootmark" serve --vault vault --http "$a" 2>ext.err >/dev/null || rc=$?
  if [ "$rc" = 0 ] || ! grep -q "127.0.0.1 only" ext.err || grep -q "listening" ext.err; then ext_ok=false; fi
  case "$a" in "$lan:0") cls="lan" ;; *) cls="${a%:0}" ;; esac
  detail="$detail $cls:exit$rc"
done
socks="$(lsof -nP -a -p "$pid" -iTCP -sTCP:LISTEN 2>/dev/null | awk 'NR > 1 {print $9}' | sort -u | tr '\n' ' ')"
listen="other"
if [ "$socks" = "127.0.0.1:$port " ]; then listen="127.0.0.1 only"; else ext_ok=false; fi
record D7-5 "external interface refused; listening only on 127.0.0.1" "$ext_ok" "$detail; listen: $listen"

# other routes and tool errors
c="$(code -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' "$base/nope")"
c2="$(code -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' -d '[1]' "$base/blame")"
c3="$(code -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' -d '{"doc_path":"nope.md"}' "$base/blame")"
e3="$(jq -r .error body.json)"
c4="$(code -X GET -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' "$base/blame")"
record H-1 "unknown tool 404, non-object body 400, tool error 422 with message, GET on a tool 405" \
  "$( [ "$c" = 404 ] && [ "$c2" = 400 ] && [ "$c3" = 422 ] && [ "$c4" = 405 ] && [[ "$e3" == *"no snapshot"* ]] && echo true || echo false)" "$c $c2 $c3 $c4"

# watch: a human edit outside the server
n_before="$(code -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' -d '{"doc_path":"espresso.md"}' "$base/blame" >/dev/null; jq '.blocks | length' body.json)"
printf '\nA watched human paragraph about zanzibar crema.\n' >>vault/espresso.md
seen=false
for _ in $(seq 1 20); do
  sleep 0.1
  code -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' -d '{"doc_path":"espresso.md"}' "$base/blame" >/dev/null
  if [ "$(jq '.blocks | length' body.json)" -gt "$n_before" ]; then seen=true; break; fi
done
ok=false
if [ "$seen" = true ] && jq -e '.blocks[-1] | .layer == "canonical" and .author_kind == "Human" and .source == "rootmark-serve-watch"' body.json >/dev/null; then ok=true; fi
record W-1 "external human edit ingested within 2 s: canonical, Human, rootmark-serve-watch" "$ok"
"$rootmark" index status vault >status_hot.json
record W-2 "index: no stale document, hot tier non-empty" \
  "$(jq -e '.stale_docs == [] and .hot_records > 0' status_hot.json >/dev/null && echo true || echo false)"

# hidden entries and non-Markdown files are not watched (the rule of
# `ingest-dir`, core::files::markdown_files); a visible edit afterwards
# marks the end of the batch
mkdir -p vault/.obsidian vault/sub/.x
printf '# Hidden\n\nObsidian settings note.\n' >vault/.obsidian/h.md
printf '# Dot\n\nA dotfile note.\n' >vault/.dotfile.md
printf '# Deep\n\nA note in a hidden directory.\n' >vault/sub/.x/y.md
printf 'not markdown\n' >vault/notes.txt
n_f="$(code -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' -d '{"doc_path":"filter.md"}' "$base/blame" >/dev/null; jq '.blocks | length' body.json)"
printf '\nA visible marker paragraph about kiwano filters.\n' >>vault/filter.md
for _ in $(seq 1 20); do
  sleep 0.1
  code -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' -d '{"doc_path":"filter.md"}' "$base/blame" >/dev/null
  [ "$(jq '.blocks | length' body.json)" -gt "$n_f" ] && break
done
c1="$(code -X POST -H "Authorization: Bearer $tok" -H 'X-Consumer: http-e2e' -d '{"doc_path":".dotfile.md"}' "$base/blame")"
hidden_docs="$(jq -r '.doc_path' vault/.rootmark/events.jsonl | sort -u | grep -E '(^|/)\.|\.txt$' || true)"
record W-3 "hidden .md files and non-.md files are not ingested" \
  "$( [ "$c1" = 422 ] && [ -z "$hidden_docs" ] && echo true || echo false)" "blame .dotfile.md $c1; logged: $hidden_docs"

# SIGTERM: compact, exit 0
kill -TERM "$pid"
rc=0
wait "$pid" || rc=$?
"$rootmark" index status vault >status_after.json
"$rootmark" search vault zanzibar --mode lexical --consumer http-e2e >search_after.json
ok=false
if [ "$rc" = 0 ] && grep -q "compact (shutdown)" serve.err && grep -q "stopped" serve.err &&
   jq -e '.hot_records == 0 and .hot_snapshots == 0 and .stale_docs == []' status_after.json >/dev/null &&
   [ ! -e vault/.rootmark/index/hot ] &&
   jq -e '.hits | length > 0 and (.[0].doc_path == "espresso.md") and .[0].layer == "canonical"' search_after.json >/dev/null; then
  ok=true
fi
record S-1 "SIGTERM → compact, exit 0, hot tier empty, edit found in the cold index" "$ok" "exit $rc"

# a token file readable by others: the server refuses to start
chmod 644 "$tokfile"
rc=0
"$rootmark" serve --vault vault --http 127.0.0.1:0 2>badtok.err || rc=$?
record T-3 "token file mode 644 → refused" \
  "$( [ "$rc" != 0 ] && grep -q "must be 600" badtok.err && echo true || echo false)" "exit $rc"
chmod 600 "$tokfile"

# compaction timer: [index] compact_every_s = 1 → the hot tier is compacted
# while the server runs
sed -i.bak 's/^compact_every_s = .*/compact_every_s = 1/' vault/.rootmark/policy.toml && rm vault/.rootmark/policy.toml.bak
"$rootmark" serve --vault vault --watch 2>timer.err &
tpid=$!
for _ in $(seq 1 600); do grep -q "rootmark serve: watching" timer.err 2>/dev/null && break; sleep 0.1; done
printf '\nA second watched paragraph about okapi roasting.\n' >>vault/filter.md
ok=false
for _ in $(seq 1 50); do
  sleep 0.1
  if grep -q "compact (timer): .* [1-9][0-9]* moved from the hot tier" timer.err; then ok=true; break; fi
done
"$rootmark" index status vault >status_timer.json
jq -e '.hot_records == 0 and .stale_docs == []' status_timer.json >/dev/null || ok=false
kill -TERM "$tpid"; wait "$tpid" || ok=false
record C-1 "compaction timer (compact_every_s = 1) empties the hot tier while serving" "$ok"

jq -n --argjson r "$results" '
  {cases: $r,
   d7_passed: ([$r[] | select((.id | startswith("D7-")) and .pass)] | length),
   d7_total: ([$r[] | select(.id | startswith("D7-"))] | length),
   passed: ([$r[] | select(.pass)] | length),
   total: ($r | length)}
  | .pass = (.passed == .total)' >http_e2e.json
jq -r '"http_e2e: D-7 \(.d7_passed)/\(.d7_total), all \(.passed)/\(.total)"' http_e2e.json
jq -e .pass http_e2e.json >/dev/null
