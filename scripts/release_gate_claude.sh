#!/usr/bin/env bash
# The R21 release gate (D-1; decision K-D1.2, option (b) of K-D1-T1): real
# Claude Code, current, listed rootmark's tools and ran `propose` against this
# version within 7 days before the release (P2-Q.2 verification, recommendation
# Ö3). A gate of its own beside scripts/release_check.sh (R1–R20, R4a), not an
# item of it: the frozen measurement code reads that checklist's items
# (research/eval/run_p1e1_checks.release_ids, verify_mac E1-4), docs/RELEASE.md
# names this gate outside the item table. Not frozen.
#
# The record: research/p2q2/claude_probe.py --out <dist>/claude_probe (its own
# `recorded_at`, UTC; never a file time). Passes when one record has exit 0,
# rootmark_version = "rootmark <Cargo.toml version>", 11 tools equal to the
# 0.3.1 list, a `propose` call, >= 1 agent event, recorded_at at most 7 days
# (604 800 s) before now and not in the future, and claude_code_version equal
# to the installed `claude --version` ("current"). No record, an older one,
# one without recorded_at, another Claude Code or rootmark version: FAIL.
# Writes <dist>/release_gate_claude.json; exit 0 only on a pass.
#
#   scripts/release_gate_claude.sh [<dist-dir>]      (default dist/)
set -uo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist="${1:-$root/dist}"
cd "$root"
ver="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)"
ok=1; detail="no record (<dist>/claude_probe/*.json: research/p2q2/claude_probe.py --out <dist>/claude_probe)"
cc_now="$(claude --version 2>/dev/null | head -1 || true)"
now_s="$(date -u +%s)"
for f in "$dist"/claude_probe/*.json; do
  [ -f "$f" ] || continue
  if jq -e --arg ver "rootmark $ver" --arg cc "$cc_now" --argjson now "$now_s" '
      (.recorded_at | type == "string")
      and ((.recorded_at | fromdateiso8601) as $t | $t <= $now and ($now - $t) <= 604800)
      and .exit == 0 and .rootmark_version == $ver
      and .tools_listed_count == 11 and .tools_match_0_3_1 == true
      and ([.tool_calls[]? | tostring | select(test("propose"))] | length) >= 1
      and .agent_events >= 1
      and $cc != "" and .claude_code_version == $cc' "$f" >/dev/null 2>&1; then
    ok=0; detail="$(jq -r '"\(.label): \(.claude_code_version), recorded \(.recorded_at), tools \(.tools_listed_count), propose"' "$f")"; break
  fi
  detail="$(jq -r --arg cc "${cc_now:-claude not installed}" '"\(.label // "?"): recorded \(.recorded_at // "never"), \(.claude_code_version // "?") (installed: \($cc)), \(.rootmark_version // "?"), exit \(.exit // "?") — not a pass"' "$f" 2>/dev/null || echo "unreadable record")"
done
jq -n --argjson ok "$([ "$ok" = 0 ] && echo true || echo false)" --arg d "$detail" --arg v "$ver" \
  '{gate: "R21", item: "real Claude Code (current) lists the tools and proposes, recorded <= 7 days ago", version: $v, ok: $ok, detail: $d}' \
  >"$dist/release_gate_claude.json"
printf 'release_gate_claude: %s R21 real Claude Code (current) lists the tools and proposes, recorded <= 7 days ago — %s\n' "$([ "$ok" = 0 ] && echo 'ok  ' || echo FAIL)" "$detail"
exit "$ok"
