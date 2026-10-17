#!/usr/bin/env bash
# P1-E/E5 (GOAL E-7): `mdk conformance` is the `make conformance`
# equivalent of a binary. Compares the two reports test by test — id, level,
# result, actual and expected hash — and the manifest hash, level claimed,
# passed and total. Exit 1 on any difference.
#
#   scripts/conformance_equiv.sh <make-report.json> <mdk-report.json> [<out.json>]
set -euo pipefail
a="${1:?make conformance report}"; b="${2:?mdk conformance report}"; out="${3:-}"
res="$(jq -n --slurpfile a "$a" --slurpfile b "$b" '
  def t: [.tests[] | {id, level, result, actual_hash, expected_hash}] | sort_by(.id);
  {tests_equal: (($a[0] | t) == ($b[0] | t)),
   manifest_hash_equal: ($a[0].fixtures_manifest_hash == $b[0].fixtures_manifest_hash),
   level_claimed: [$a[0].level_claimed, $b[0].level_claimed],
   passed: [$a[0].passed, $b[0].passed], total: [$a[0].total, $b[0].total],
   spec_version: [$a[0].spec_version, $b[0].spec_version],
   differing_tests: [($a[0] | t) as $x | ($b[0] | t) as $y | range(0; ([$x, $y] | map(length) | max)) | select($x[.] != $y[.]) | ($x[.].id // $y[.].id)]}
  | .equal = (.tests_equal and .manifest_hash_equal and .level_claimed[0] == .level_claimed[1]
              and .passed[0] == .passed[1] and .total[0] == .total[1] and .spec_version[0] == .spec_version[1])')"
[ -z "$out" ] || echo "$res" >"$out"
echo "$res"
jq -e .equal <<<"$res" >/dev/null
