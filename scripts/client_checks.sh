#!/usr/bin/env bash
# GOAL P1-E E-10 (TASKS E7): the Obsidian client's gates, recorded —
# `tsc --noEmit` (strict tsconfig), `eslint .`, `vitest run` (counts), the
# bundle (`npm run build` → main.js < 500 KB), no runtime dependency
# besides the Obsidian API (package.json has no dependencies; main.js
# requires only the externals obsidian, electron and CodeMirror).
#
#   scripts/client_checks.sh <out.json>
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: client_checks.sh <out.json>}"
mkdir -p "$(dirname "$out")"
out="$(cd "$(dirname "$out")" && pwd)/$(basename "$out")"
cd "$root/clients/obsidian"
log="$(mktemp -t rootmark-client.XXXXXX)"; trap 'rm -f "$log" vitest.json' EXIT
npm ci --no-audit --no-fund >"$log" 2>&1 || { cat "$log" >&2; exit 1; }
strict="$(jq -r '.compilerOptions.strict' tsconfig.json)"
tsc=0; npx tsc --noEmit >"$log" 2>&1 || tsc=$?
eslint=0; npx eslint . >>"$log" 2>&1 || eslint=$?
vitest=0; npx vitest run --reporter=json --outputFile=vitest.json >>"$log" 2>&1 || vitest=$?
build=0; npm run -s build >>"$log" 2>&1 || build=$?
size="$(wc -c <main.js 2>/dev/null | tr -d ' ' || echo 0)"
# the bundle's run-time requires: only the app-provided externals
requires="$(grep -oE 'require\("[^"]+"\)' main.js 2>/dev/null | sort -u | sed 's/require("\(.*\)")/\1/' | tr '\n' ' ')"
jq -n --argjson tsc "$tsc" --argjson eslint "$eslint" --argjson vitest "$vitest" --argjson build "$build" \
  --arg strict "$strict" --argjson size "${size:-0}" --arg req "$requires" \
  --slurpfile v vitest.json --slurpfile pkg package.json '
  ($req | split(" ") | map(select(. != ""))) as $r
  | {tsc_strict: ($strict == "true"), tsc_exit: $tsc, eslint_exit: $eslint, vitest_exit: $vitest, build_exit: $build,
     vitest: {files: $v[0].numTotalTestSuites, tests: $v[0].numTotalTests, passed: $v[0].numPassedTests, failed: $v[0].numFailedTests},
     bundle_bytes: $size, bundle_below_500kb: ($size > 0 and $size < 512000),
     runtime_dependencies: (($pkg[0].dependencies // {}) | keys),
     bundle_requires: $r,
     bundle_requires_only_app_externals: all($r[]; IN("obsidian", "electron", "@codemirror/state", "@codemirror/view"))}
  | .pass = (.tsc_strict and .tsc_exit == 0 and .eslint_exit == 0 and .vitest_exit == 0 and .build_exit == 0
             and .vitest.failed == 0 and .vitest.tests > 0 and .bundle_below_500kb
             and (.runtime_dependencies | length) == 0 and .bundle_requires_only_app_externals)' >"$out"
jq -c '{tests: .vitest.tests, bundle_bytes, runtime_dependencies, bundle_requires, pass}' "$out"
jq -e .pass "$out" >/dev/null || { tail -30 "$log" >&2; exit 1; }
