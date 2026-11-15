#!/usr/bin/env bash
# D-1/R2 (and R6): the product README's quick start blocks, run as written.
# The ```sh blocks under "## Quick start" are extracted verbatim from the
# given README and each is run with bash in a fresh, synthetic HOME (the
# vault $HOME/notes is a copy of spec/fixtures/mini_vault: no personal
# content). A block holding `rootmark serve` is started in the background,
# passes when the server says it listens, and is stopped with SIGTERM.
# Writes a record (block headings, exit codes; no host value) and exits 1
# unless every selected block passed.
#
#   readme_quickstart.sh <README.md> <record.json> <block heading>...
#   e.g. … "Your first vault"   (before the release: a local rootmark on PATH)
#        … "Binary (macOS on Apple silicon, Linux x86_64)" "Homebrew …" "Docker …" "Your first vault"
set -uo pipefail
readme="$1"; rec="$2"; shift 2
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
work="$(mktemp -d -t rootmark-readme.XXXXXX)"; trap 'rm -rf "$work"' EXIT
home="$work/home"; mkdir -p "$home"
cp -R "$here/spec/fixtures/mini_vault" "$home/notes"
find "$home/notes" -name .rootmark -prune -exec rm -rf {} +
# extract: heading (### …) → its ```sh block, verbatim
mkdir -p "$work/blocks"
awk -v out="$work/blocks" '
  /^## / { inqs = ($0 ~ /^## Quick start/) }
  inqs && /^### / { h = substr($0, 5); n++; next }
  inqs && /^```sh$/ && h != "" { inb = 1; f = out "/" n ".sh"; print h > (out "/" n ".heading"); next }
  inb && /^```$/ { inb = 0; next }
  inb { print > f }
' "$readme"
rows=()
for want in "$@"; do
  hf="$(grep -lxF "$want" "$work"/blocks/*.heading 2>/dev/null | head -1)"
  if [ -z "$hf" ]; then rows+=("$(jq -n --arg h "$want" '{block: $h, found: false, pass: false}')"); continue; fi
  b="${hf%.heading}.sh"
  log="$work/$(basename "$b").log"
  if grep -q 'rootmark serve' "$b"; then
    # its own process group (job control), so the server — a grandchild — is stopped too
    set -m
    (cd "$home" && HOME="$home" bash -e "$b" >"$log" 2>&1) &
    pid=$!; set +m; ok=false
    for _ in $(seq 1 120); do grep -q 'http listening' "$log" && { ok=true; break; }; sleep 0.5; done
    kill -TERM -- "-$pid" 2>/dev/null; wait "$pid" 2>/dev/null
    for _ in $(seq 1 40); do pgrep -g "$pid" >/dev/null 2>&1 || break; sleep 0.5; done
    if pgrep -g "$pid" >/dev/null 2>&1; then echo "readme_quickstart: the serve block did not stop" >&2; kill -KILL -- "-$pid" 2>/dev/null; ok=false; fi
    rc=$($ok && echo 0 || echo 1)
  else
    (cd "$home" && HOME="$home" bash -e "$b" >"$log" 2>&1); rc=$?
  fi
  rows+=("$(jq -n --arg h "$want" --argjson rc "$rc" --argjson lines "$(wc -l <"$b")" \
    --arg tail "$(tail -3 "$log" | sed -e "s#$home#<home>#g" -e "s#$work#<work>#g")" \
    '{block: $h, found: true, lines: $lines, exit: $rc, tail: $tail, pass: ($rc == 0)}')")
done
printf '%s\n' "${rows[@]}" | jq -s '{what: "D-1 README quick start blocks run as written (reports/rounds/D-1/r2/readme_quickstart.sh)", vault: "copy of spec/fixtures/mini_vault", blocks: ., pass: all(.[]; .pass)}' >"$rec"
cat "$rec"
jq -e .pass "$rec" >/dev/null
