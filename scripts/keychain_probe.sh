#!/usr/bin/env bash
# D-D of P2-0 (GOAL P2-0; TASKS Z6; H-P2-0e): can a terminal process (the
# kind of process an agent such as Claude Code runs) read the secret that
# Obsidian's keychain item protects without a person approving it?
#
# The only keychain primitive an Obsidian plugin has is Electron's
# `safeStorage`, whose key is the login-keychain item "Obsidian Safe Storage"
# (its ACL trusts Obsidian.app). Stage 2 would keep the attestation secret
# encrypted with that key. This probe tries to read the item's secret from
# this (terminal) process `ATTEMPTS` times (default 20), each bounded by
# `TIMEOUT_S` (default 5) seconds: an access prompt that nobody answers, a
# refusal or a timeout is "not read"; a secret on stdout is "read". The
# secret is never printed, stored or compared — only whether one came back.
#
# Run it yourself, by hand, on the Mac where Obsidian has run once (the item
# exists); click "Deny" (or nothing) on every prompt: clicking "Allow" is
# exactly the approval this measures the absence of. Never click "Always
# Allow": it adds /usr/bin/security to the item's access list for good (undo
# it in Keychain Access → "Obsidian Safe Storage" → Access Control). A prompt
# of a killed attempt may stay on screen; dismiss it with "Deny".
#
#   scripts/keychain_probe.sh [<out.json>]
set -Eeuo pipefail
trap 'echo "keychain_probe: failed at line $LINENO: $BASH_COMMAND" >&2' ERR
out="${1:-/dev/stdout}"
n="${ATTEMPTS:-20}"
t="${TIMEOUT_S:-5}"
[ "$(uname -s)" = Darwin ] || { echo "keychain_probe: macOS only" >&2; exit 1; }
[[ "$n" =~ ^[1-9][0-9]*$ ]] || { echo "keychain_probe: ATTEMPTS must be a positive integer" >&2; exit 1; }
[[ "$t" =~ ^[1-9][0-9]*$ ]] || { echo "keychain_probe: TIMEOUT_S must be a positive integer (0 would wait forever)" >&2; exit 1; }
# the item must exist (attributes only, no secret, no prompt)
if ! err="$(security find-generic-password -s "Obsidian Safe Storage" 2>&1 >/dev/null)"; then
  echo "keychain_probe: cannot look up \"Obsidian Safe Storage\" (open Obsidian once; unlock the login keychain): $err" >&2
  exit 1
fi
read=0
denied=0
codes=()
for _ in $(seq 1 "$n"); do
  rc=0
  len="$(perl -e 'alarm shift; exec @ARGV' "$t" security find-generic-password -s "Obsidian Safe Storage" -w 2>/dev/null | wc -c | tr -d ' ')" || rc=$?
  codes+=("$rc")
  if [ "$rc" -eq 0 ] && [ "${len:-0}" -gt 1 ]; then read=$((read + 1)); else denied=$((denied + 1)); fi
done
printf '{"probe": "keychain", "item": "Obsidian Safe Storage", "attempts": %d, "timeout_s": %d, "read": %d, "not_read": %d, "exit_codes": [%s], "pass": %s}\n' \
  "$n" "$t" "$read" "$denied" "$(IFS=,; echo "${codes[*]}")" "$([ "$read" -eq 0 ] && [ "$denied" -eq "$n" ] && echo true || echo false)" >"$out"
