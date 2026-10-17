#!/usr/bin/env bash
# P1-E.1 deliberate cases of the real-Obsidian e2e (E1.2; E1-3, E1-9;
# K-E1.3); each must be rejected, each control must pass:
#   without an Obsidian binary scripts/plugin_e2e_real.sh reports a skip
#     (`skipped: true`, `pass: null`, exit 0), never a pass (control);
#   with one, a plugin bundle whose status bar does not name the index state
#     → the E1-9 step fails while the 11 steps still pass. Without an
#     Obsidian binary this case is not run and `real_cases_run` is false.
# Writes <out.json>: {cases, rejected, rejected_total, controls_ok,
# controls_total, real_cases_run, pass}; exit 1 unless pass.
#
#   scripts/test_tamper_p1e1_e2e.sh <out.json>
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: test_tamper_p1e1_e2e.sh <out.json>}"
work="$(mktemp -d -t mdk-tamper-p1e1-e2e.XXXXXX)"
trap 'rm -rf "$work"' EXIT
export MDK_MODELS="${MDK_MODELS:-$here/models}"
cd "$here"
cases=()
add() { # add <group> <case> <expected rejected|accepted> <rc> [<detail JSON>]
  local got=accepted
  [ "$4" != 0 ] && got=rejected
  cases+=("$(jq -n --arg g "$1" --arg c "$2" --arg e "$3" --arg got "$got" --argjson d "${5:-null}" \
    '{group: $g, case: $c, expected: $e, got: $got, detail: $d,
      as_expected: ($e == $got and ($d == null or $d.as_expected))}')")
}
run() { local rc=0; "$@" >/dev/null 2>&1 || rc=$?; echo "$rc"; }

rc="$(MDK_OBSIDIAN=/nonexistent/Obsidian run bash scripts/plugin_e2e_real.sh "$work/skip")"
detail="$(jq -c '{skipped, pass, as_expected: (.skipped == true and .pass == null)}' "$work/skip/plugin_e2e_real.json")"
add real-e2e "no Obsidian binary: skip reported, not a pass" accepted "$rc" "$detail"

obs="${MDK_OBSIDIAN:-/Applications/Obsidian.app/Contents/MacOS/Obsidian}"
real=false
if [ -x "$obs" ]; then
  real=true
  (cd clients/obsidian && { [ -d node_modules ] || npm ci --no-audit --no-fund >/dev/null; } && npm run -s build)
  pd="$work/plugin"
  mkdir -p "$pd"
  cp clients/obsidian/manifest.json clients/obsidian/styles.css "$pd/"
  # the status-bar label of a non-ok index removed from the bundle
  perl -pe 's/` \\xB7 \$\{ix\.label\}`/""/g' clients/obsidian/main.js >"$pd/main.js"
  cmp -s "$pd/main.js" clients/obsidian/main.js && { echo "test_tamper_p1e1_e2e: index label not found in main.js" >&2; exit 1; }
  rc="$(MDK_E2E_PLUGIN_DIR="$pd" run bash scripts/plugin_e2e_real.sh "$work/tamper")"
  detail="$(jq -c '{passed, e1_9: .e1_9_connection.pass,
    as_expected: (.passed == 11 and (.e1_9_connection.pass | not))}' "$work/tamper/plugin_e2e_real.json")"
  add real-e2e "plugin bundle without the index state in the status bar" rejected "$rc" "$detail"
else
  echo "test_tamper_p1e1_e2e: no Obsidian binary at $obs; the bundle case is not run" >&2
fi

printf '%s\n' "${cases[@]}" | jq -s --argjson real "$real" '{cases: .,
  rejected: ([.[] | select(.expected == "rejected" and .as_expected)] | length),
  rejected_total: ([.[] | select(.expected == "rejected")] | length),
  controls_ok: ([.[] | select(.expected == "accepted" and .as_expected)] | length),
  controls_total: ([.[] | select(.expected == "accepted")] | length),
  real_cases_run: $real}
  | . + {pass: (.rejected == .rejected_total and .controls_ok == .controls_total)}' >"$out"
jq -r '"test_tamper_p1e1_e2e: rejected \(.rejected)/\(.rejected_total), controls \(.controls_ok)/\(.controls_total), real cases run: \(.real_cases_run)"' "$out"
jq -e .pass "$out" >/dev/null
