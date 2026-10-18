#!/usr/bin/env bash
# One release target's check record (K-E1.1; GOAL P1-E.1 E1-5), the same
# locally (scripts/release.sh) and in CI (.github/workflows/release.yml):
#
#   scripts/release_target.sh check <target> <required true|false> <out.json> <binary> [<command>…]
#       runs `<binary> --version` (or, when given, `<command…> --version`:
#       the binary run elsewhere, e.g. in a container), `--spec-version` and
#       `conformance` (Levels 1–3 on the embedded fixtures; ROOTMARK_MODELS) and
#       writes {target, required, built: true, sha256 (of the binary),
#       version, spec_version, conformance {level_claimed, passed, total,
#       fixtures_manifest} | {error}, conformance_exit, ok}; ok = version
#       `rootmark <ver>`, spec version of the tree, 37/37 at level 3 (34/34 before spec 2.0.0).
#   scripts/release_target.sh skipped <target> <required> <out.json> <reason>
#       {target, required, built: false, skipped: true, reason, ok: false}
#   scripts/release_target.sh assemble <dist> <commit> <dirty true|false>
#       <dist>/release.json from <dist>/bin/*.json: {version, spec_version,
#       commit, worktree_dirty, required_targets, targets, built, ok,
#       required_ok}; required_ok = every required target's record ok and
#       at least one required target. Exit 1 unless required_ok.
# The versions are read from the tree (Cargo.toml, server/src/tools.rs).
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ver="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$root/Cargo.toml" | head -1)"
spec="$(sed -n 's/^pub const SPEC_VERSION: &str = "\(.*\)";$/\1/p' "$root/server/src/tools.rs")"
[ -n "$ver" ] && [ -n "$spec" ] || { echo "release_target: cannot read versions" >&2; exit 1; }
sha() { if command -v shasum >/dev/null; then shasum -a 256 "$1"; else sha256sum "$1"; fi | cut -d' ' -f1; }

case "${1:-}" in
  check)
    t="$2"; req="$3"; out="$4"; bin="$5"; shift 5
    [ -f "$bin" ] || { echo "release_target: no binary $bin" >&2; exit 1; }
    [ "$#" -gt 0 ] || set -- "$bin"
    v="$("$@" --version 2>&1 | tr -d '\r' || true)"
    s="$("$@" --spec-version 2>&1 | tr -d '\r' || true)"
    rc=0
    c="$("$@" conformance 2>/dev/null | tr -d '\r')" || rc=$?
    jq -n --arg t "$t" --argjson req "$req" --arg h "$(sha "$bin")" --arg v "$v" --arg s "$s" --argjson rc "$rc" --arg c "$c" --arg ver "$ver" --arg spec "$spec" '
      ($c | try fromjson catch null) as $r
      | {target: $t, required: $req, built: true, sha256: $h, version: $v, spec_version: $s,
         conformance: (if $r == null then {error: ($c | .[0:400])} else {level_claimed: $r.level_claimed, passed: $r.passed, total: $r.total, fixtures_manifest: $r.fixtures_manifest} end),
         conformance_exit: $rc,
         ok: ($v == ("rootmark " + $ver) and $s == $spec and $r != null and $r.level_claimed == 3 and $r.passed == 37 and $r.total == 37)}' >"$out"
    jq -r '"release_target: \(if .ok then "ok  " else "FAIL" end) \(.target) \(.version) spec \(.spec_version) conformance \(.conformance.passed // "-")/\(.conformance.total // "-")"' "$out"
    ;;
  skipped)
    jq -n --arg t "$2" --argjson req "$3" --arg r "$5" '{target: $t, required: $req, built: false, skipped: true, reason: $r, ok: false}' >"$4"
    echo "release_target: skipped $2 ($5)"
    ;;
  assemble)
    dist="$2"; commit="$3"; dirty="$4"
    jq -s --arg ver "$ver" --arg spec "$spec" --arg c "$commit" --argjson d "$dirty" '
      sort_by(.target)
      | {version: $ver, spec_version: $spec, commit: $c, worktree_dirty: $d,
         required_targets: [.[] | select(.required) | .target],
         targets: .,
         built: ([.[] | select(.built)] | length), ok: ([.[] | select(.ok)] | length),
         required_ok: (([.[] | select(.required)] | length) > 0 and all(.[] | select(.required); .ok))}' \
      "$dist"/bin/*.json >"$dist/release.json"
    jq -r '.targets[] | "release: \(if .ok then "ok  " elif .skipped then "skip" else "FAIL" end) \(.target)\(if .required then " (required)" else " (CI info)" end)\(if .built then "" else ": \(.reason)" end)"' "$dist/release.json"
    jq -r '"release: required \(.required_targets | join(", ")): \(if .required_ok then "ok" else "FAIL" end); \(.ok)/\(.targets | length) targets ok"' "$dist/release.json"
    jq -e .required_ok "$dist/release.json" >/dev/null
    ;;
  *)
    echo "usage: release_target.sh check|skipped|assemble …" >&2
    exit 2
    ;;
esac
