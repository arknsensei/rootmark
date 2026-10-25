#!/usr/bin/env bash
# P2-B/B4 (GOAL P2-B B-7, B-8): both clients' gates and the contract check of
# every HTTP call they make, recorded.
#   - clients/obsidian and clients/vscode: `npm ci`, `tsc --noEmit`, `eslint .`,
#     `vitest run` (every test file under the contract checker; each file's
#     record into <out>/contract/), the VS Code bundle (`npm run build`);
#   - the two e2e runs against a real `rootmark serve`: the Obsidian plugin's
#     command layer aligning >= 20 D-NOTES notes by location
#     (clients/obsidian/e2e/p2_b), the VS Code command layer
#     (clients/vscode/e2e); each writes its contract report;
#   - <out>/client_checks.json: per client the gates' exits and test counts,
#     the e2e results, and the contract summary over every run: calls; B-8's
#     count = the clients' requests outside the contract in every run + the
#     real server's answers outside the contract in the e2e runs (must be 0);
#     listed apart, never hidden: the unit tests' fake-server answers outside
#     the contract (negative tests of the client's refusal, older samples)
#     and the deliberate calls of negative tests.
# No time, port, token or absolute path in the record. Exit 0 when every
# gate passes and B-8's count is 0, 3 otherwise (the record is written).
#
#   scripts/p2b_client_checks.sh <out-dir> [<rootmark>]
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: p2b_client_checks.sh <out-dir> [<rootmark>]}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
rm -rf "$out/contract" && mkdir -p "$out/contract"
rootmark="${2:-$root/target/release/rootmark}"
[ -x "$rootmark" ] || { echo "p2b_client_checks: no rootmark binary at $rootmark" >&2; exit 2; }
log="$(mktemp -t rootmark-p2b-clients.XXXXXX)"
trap 'rm -f "$log" "$log".*' EXIT

gate() { # gate <client> <name> <cmd…> → exit code
  local c="$1" n="$2"; shift 2
  if (cd "$root/clients/$c" && "$@") >>"$log" 2>&1; then echo 0; else echo $?; fi
}
vitest_counts() { # vitest_counts <client> → "files tests passed failed"
  (cd "$root/clients/$1" && ROOTMARK_CONTRACT_LOG_DIR="$out/contract" npx vitest run --reporter=json --outputFile="$log.$1.json" >>"$log" 2>&1) || true
  jq -r '"\(.numTotalTestSuites) \(.numTotalTests) \(.numPassedTests) \(.numFailedTests)"' "$log.$1.json" 2>/dev/null || echo "0 0 0 1"
}

ci_o="$(gate obsidian ci npm ci --no-audit --no-fund)"
tsc_o="$(gate obsidian tsc npx tsc --noEmit)"
es_o="$(gate obsidian eslint npx eslint .)"
ci_v="$(gate vscode ci npm ci --no-audit --no-fund)"
tsc_v="$(gate vscode tsc npx tsc --noEmit)"
es_v="$(gate vscode eslint npx eslint .)"
read -r of ot op ofail <<<"$(vitest_counts obsidian)"
read -r vf vt vp vfail <<<"$(vitest_counts vscode)"
build="$(gate vscode build npm run -s build)"
requires="$(grep -oE 'require\("[^"]+"\)' "$root/clients/vscode/dist/extension.cjs" 2>/dev/null | sort -u | sed 's/require("\(.*\)")/\1/' | tr '\n' ' ' || true)"

# the e2e runs (their own contract reports)
e2e_o=0
(cd "$root/clients/obsidian" && node e2e/p2_b/build.mjs && node e2e/p2_b/.build/run.mjs --rootmark "$rootmark" --repo "$root" --out "$out") >>"$log" 2>&1 || e2e_o=$?
e2e_v=0
(cd "$root/clients/vscode" && node e2e/build.mjs && node e2e/.build/run.mjs --rootmark "$rootmark" --out "$out") >>"$log" 2>&1 || e2e_v=$?
for f in obsidian_e2e vscode_e2e; do
  [ -f "$out/$f.json" ] || { tail -40 "$log" >&2; echo "p2b_client_checks: $f.json missing" >&2; exit 3; }
done

# every run's contract record: the unit-test files and the two e2e runs
summary="$(jq -n --slurpfile u <(jq -s . "$out"/contract/*.json) --slurpfile e <(jq -s '[.[] | .contract]' "$out/obsidian_e2e.json" "$out/vscode_e2e.json") '
  ($u[0] + $e[0]) as $all
  | {
    runs: ($all | length),
    calls: ($all | map(.calls) | add),
    request_violations: ($all | map(.request_violations) | add),
    response_violations_e2e: ($e[0] | map(.response_violations) | add),
    response_violations_unit_fake: ($u[0] | map(.response_violations) | add),
    deliberate_violations: ($all | map(.deliberate_violations) | add)
  }')"

jq -n \
  --argjson ci_o "$ci_o" --argjson tsc_o "$tsc_o" --argjson es_o "$es_o" \
  --argjson ci_v "$ci_v" --argjson tsc_v "$tsc_v" --argjson es_v "$es_v" \
  --argjson of "$of" --argjson ot "$ot" --argjson op "$op" --argjson ofail "$ofail" \
  --argjson vf "$vf" --argjson vt "$vt" --argjson vp "$vp" --argjson vfail "$vfail" \
  --argjson build "$build" --arg req "$requires" \
  --argjson e2e_o "$e2e_o" --argjson e2e_v "$e2e_v" \
  --slurpfile eo "$out/obsidian_e2e.json" --slurpfile ev "$out/vscode_e2e.json" \
  --argjson contract "$summary" '
  ($req | split(" ") | map(select(. != ""))) as $r
  | {
    obsidian: {npm_ci: $ci_o, tsc: $tsc_o, eslint: $es_o,
      vitest: {files: $of, tests: $ot, passed: $op, failed: $ofail},
      e2e: {exit: $e2e_o, notes: $eo[0].notes, aligned_by_location: $eo[0].aligned_by_location, stale: $eo[0].stale, pass: $eo[0].pass}},
    vscode: {npm_ci: $ci_v, tsc: $tsc_v, eslint: $es_v,
      vitest: {files: $vf, tests: $vt, passed: $vp, failed: $vfail},
      build: $build, bundle_requires: $r,
      e2e: {exit: $e2e_v, steps: $ev[0].steps, pass: $ev[0].pass}},
    contract: $contract,
    "B-7": (($ci_o + $tsc_o + $es_o + $ofail + $ci_v + $tsc_v + $es_v + $vfail + $build) == 0
            and $ot > 0 and $vt > 0 and $eo[0].pass and $eo[0].notes >= 20
            and ($r - ["vscode", "node:fs/promises", "node:path"] | length) == 0),
    "B-8": {outside_contract: (($contract.request_violations | length) + ($contract.response_violations_e2e | length)),
            pass: ((($contract.request_violations | length) + ($contract.response_violations_e2e | length)) == 0 and $ev[0].pass)}
  }
  | .pass = (."B-7" and ."B-8".pass)' >"$out/client_checks.json"
jq -r '"p2b-client-checks: B-7 \(."B-7") (obsidian \(.obsidian.vitest.passed)/\(.obsidian.vitest.tests) tests, e2e \(.obsidian.e2e.aligned_by_location)/\(.obsidian.e2e.notes) by location; vscode \(.vscode.vitest.passed)/\(.vscode.vitest.tests) tests, build \(.vscode.build)); B-8 \(.contract.calls) calls in \(.contract.runs) runs, \(."B-8".outside_contract) outside the contract (apart: \(.contract.deliberate_violations | length) deliberate calls, \(.contract.response_violations_unit_fake | length) fake-server answers in unit tests)"' "$out/client_checks.json"
[ "$(jq -r .pass "$out/client_checks.json")" = "true" ] || { tail -60 "$log" >&2; exit 3; }
