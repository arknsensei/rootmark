#!/usr/bin/env bash
# GOAL P1-E E-7 in the verify chain (TASKS E7: "release build, en az yerel
# hedef"): the native release binary — `rootmark --version`, `--spec-version`,
# `rootmark conformance` (embedded fixtures, 34/34) — and its report equal to a
# fresh `make conformance` test by test (scripts/conformance_equiv.sh).
# The four-target build is scripts/release.sh (docs/RELEASE.md R4).
#
#   scripts/release_local.sh <out-dir>     → <out-dir>/release_local.json,
#                                            conformance_rootmark.json, conformance_equiv.json
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: release_local.sh <out-dir>}"
mkdir -p "$out"; out="$(cd "$out" && pwd)"
cd "$root"
cargo build -q --release --locked -p rootmark-server --bin rootmark
rootmark="$root/target/release/rootmark"
target="$(rustc -vV | sed -n 's/^host: //p')"
ver="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
spec="$(sed -n 's/^pub const SPEC_VERSION: &str = "\(.*\)";$/\1/p' server/src/tools.rs)"
v="$("$rootmark" --version)"; s="$("$rootmark" --spec-version)"
crc=0; ROOTMARK_MODELS="$root/models" "$rootmark" conformance >"$out/conformance_rootmark.json" 2>/dev/null || crc=$?
rm -f reports/conformance.json
mrc=0; make -s conformance >/dev/null 2>&1 || mrc=$?
erc=0; bash scripts/conformance_equiv.sh reports/conformance.json "$out/conformance_rootmark.json" "$out/conformance_equiv.json" >/dev/null || erc=$?
jq -n --arg t "$target" --arg v "$v" --arg s "$s" --arg ver "$ver" --arg spec "$spec" \
  --argjson crc "$crc" --argjson mrc "$mrc" --argjson erc "$erc" --slurpfile r "$out/conformance_rootmark.json" '
  {target: $t, version: $v, spec_version: $s, version_ok: ($v == "rootmark " + $ver), spec_version_ok: ($s == $spec),
   conformance: {level_claimed: $r[0].level_claimed, passed: $r[0].passed, total: $r[0].total, exit: $crc},
   make_conformance_exit: $mrc, equal_to_make_conformance: ($erc == 0)}
  | .pass = (.version_ok and .spec_version_ok and .conformance.level_claimed == 3 and .conformance.passed == 34
             and .conformance.total == 34 and .conformance.exit == 0 and .make_conformance_exit == 0 and .equal_to_make_conformance)' >"$out/release_local.json"
jq -c '{target, version, spec_version, conformance, equal_to_make_conformance, pass}' "$out/release_local.json"
jq -e .pass "$out/release_local.json" >/dev/null
