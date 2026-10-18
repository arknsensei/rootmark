#!/usr/bin/env bash
# P1-D.1 deliberate corruptions (TASKS D1.7): each must be rejected, each
# untouched control must pass.
#   contract (D1-13): a code row removed, a tool renamed, a status changed
#     in a copy of docs/CLIENT_CONTRACT.md → scripts/contract_check.sh fails;
#   events (K-D1.2): a vault whose change event carries an unpaired or
#     malformed meta.actor does not open (`rootmark replay`); with both removed
#     it opens (an old log);
#   fixtures (D1-10): C2-10 or C3-06 expected output changed in a fixture
#     copy → the conformance checkers fail;
#   policy (K-D1.1): `[edit] policy = "everything"` → the recipe is refused.
# Writes <out.json>: {cases, rejected, rejected_total, controls_ok,
# controls_total, pass}; exit 1 unless pass.
#
#   scripts/test_tamper_p1d1.sh <out.json>
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: test_tamper_p1d1.sh <out.json>}"
work="$(mktemp -d -t rootmark-tamper-p1d1.XXXXXX)"
trap 'rm -rf "$work"' EXIT
export ROOTMARK_MODELS="${ROOTMARK_MODELS:-$here/models}"
cd "$here"
cargo build -q --release -p rootmark-server --bin rootmark --example mcp_pipeline
cargo test -q -p rootmark-core --release --test conformance --no-run 2>/dev/null
cargo test -q -p rootmark-index --release --test conformance_c3 --no-run 2>/dev/null
rootmark="$here/target/release/rootmark"
cases=()
add() { # add <group> <case> <expected rejected|accepted> <rc>
  local got=accepted
  [ "$4" != 0 ] && got=rejected
  cases+=("$(jq -n --arg g "$1" --arg c "$2" --arg e "$3" --arg got "$got" \
    '{group: $g, case: $c, expected: $e, got: $got, as_expected: ($e == $got)}')")
}
run() { local rc=0; "$@" >/dev/null 2>&1 || rc=$?; echo "$rc"; }

# ---- contract
c="$work/contract.md"
cp docs/CLIENT_CONTRACT.md "$c"
add contract "untouched contract" accepted "$(ROOTMARK_CONTRACT="$c" run bash scripts/contract_check.sh)"
sed '/^| `too_large` |/d' docs/CLIENT_CONTRACT.md >"$c"
add contract "code row too_large removed" rejected "$(ROOTMARK_CONTRACT="$c" run bash scripts/contract_check.sh)"
sed 's/^| `checkpoints` |/| `checkpoint_list` |/' docs/CLIENT_CONTRACT.md >"$c"
add contract "tool checkpoints renamed" rejected "$(ROOTMARK_CONTRACT="$c" run bash scripts/contract_check.sh)"
sed 's/^| `exists` | 409 |/| `exists` | 400 |/' docs/CLIENT_CONTRACT.md >"$c"
add contract "status of exists changed" rejected "$(ROOTMARK_CONTRACT="$c" run bash scripts/contract_check.sh)"

# ---- events: an attributed deletion, then its meta.actor tampered
v="$work/vault"
mkdir -p "$v"
printf '# A\n\nfirst paragraph about beans\n\nsecond paragraph about water\n' >"$v/a.md"
"$rootmark" ingest-dir "$v" "$v" >/dev/null
printf 'Second paragraph only.\n' >"$work/d.md"
cat >"$v/.rootmark/policy.toml" <<'T'
promotion = "human_only"
[version_trigger]
idle_ms = 30000
min_sim_change = 0.2
[journal]
retention_hours = 168
[edit]
policy = "unrestricted"
T
"$rootmark" propose "$v" a.md "$work/d.md" --source tamper-agent --mode replace_section --heading A >/dev/null
log="$v/.rootmark/events.jsonl"
cp "$log" "$work/events.orig"
jq -e 'select(.kind == "BlockDeleted" and .meta.actor.kind == "agent")' "$log" >/dev/null || { echo "test_tamper_p1d1: no attributed event to tamper" >&2; exit 1; }
add events "untouched attributed log" accepted "$(run "$rootmark" replay "$v" a.md)"
tamper_last() { # tamper_last <jq filter on the last BlockDeleted>
  python3 - "$work/events.orig" "$log" "$1" <<'PY'
import json, subprocess, sys
src, dst, filt = sys.argv[1:4]
lines = open(src).read().splitlines()
i = max(n for n, l in enumerate(lines) if json.loads(l)["kind"] == "BlockDeleted")
out = subprocess.run(["jq", "-c", filt], input=lines[i], capture_output=True, text=True, check=True).stdout.strip()
lines[i] = out
open(dst, "w").write("\n".join(lines) + "\n")
PY
}
tamper_last 'del(.meta.actor, .meta.source)'
add events "actor and source both removed (old-log reading)" accepted "$(run "$rootmark" replay "$v" a.md)"
tamper_last 'del(.meta.actor)'
add events "actor removed, source kept" rejected "$(run "$rootmark" replay "$v" a.md)"
tamper_last '.meta.actor.kind = "robot"'
add events "actor kind robot" rejected "$(run "$rootmark" replay "$v" a.md)"
tamper_last '.meta.actor.id = ""'
add events "empty actor id" rejected "$(run "$rootmark" replay "$v" a.md)"
cp "$work/events.orig" "$log"

# ---- fixtures
conf() { # conf <fixture root>
  local r="$work/report.json"
  rm -f "$r"
  ROOTMARK_FIXTURES="$1" ROOTMARK_CONFORMANCE_REPORT="$r" cargo test -q -p rootmark-core --release --test conformance >/dev/null 2>&1 || return 1
  ROOTMARK_FIXTURES="$1" ROOTMARK_CONFORMANCE_REPORT="$r" cargo test -q -p rootmark-index --release --test conformance_c3 >/dev/null 2>&1 || return 1
  jq -e '.level_claimed == 3 and .passed == .total and .total == 37' "$r" >/dev/null
}
cp -R spec/fixtures "$work/fx0"
add fixtures "untouched fixture copy" accepted "$(run conf "$work/fx0")"
cp -R spec/fixtures "$work/fx1"
python3 - "$work/fx1/C2-10/expected.json" <<'PY'
import json, sys
p = sys.argv[1]
d = json.load(open(p))
for o in d["output"]:
    if o["op"] == "actor_tamper" and "open_err" in o["result"]:
        o["result"] = {"opened": True}
        break
open(p, "w").write(json.dumps(d, indent=2) + "\n")
PY
# the manifest is regenerated so that only the expectation differs
(cd research && uv run --frozen python - "$work/fx1") <<'PY'
import json, sys, pathlib, blake3
root = pathlib.Path(sys.argv[1])
m = json.loads((root / "MANIFEST.json").read_text())
files = m["files"]
for k in list(files):
    files[k] = blake3.blake3((root / k).read_bytes()).hexdigest()[:32]
(root / "MANIFEST.json").write_text(json.dumps(m, indent=2) + "\n")
PY
add fixtures "C2-10 expects a malformed actor to open" rejected "$(run conf "$work/fx1")"
cp -R spec/fixtures "$work/fx2"
python3 - "$work/fx2/C3-06/expected.json" <<'PY'
import json, sys
p = sys.argv[1]
d = json.load(open(p))
for o in d["output"]:
    if o["op"] == "agent_write" and o["result"]["decision"] == "denied":
        o["result"]["decision"] = "allowed"
        break
open(p, "w").write(json.dumps(d, indent=2) + "\n")
PY
add fixtures "C3-06 expects a denied write allowed (manifest untouched)" rejected "$(run conf "$work/fx2")"

# ---- policy
p="$work/pvault"
mkdir -p "$p/.rootmark"
sed 's/policy = "unrestricted"/policy = "everything"/' "$v/.rootmark/policy.toml" >"$p/.rootmark/policy.toml"
add policy "[edit] policy = everything" rejected "$(run "$rootmark" policy show "$p")"
sed 's/policy = "unrestricted"/policy = "append_only"/' "$v/.rootmark/policy.toml" >"$p/.rootmark/policy.toml"
add policy "[edit] policy = append_only (control)" accepted "$(run "$rootmark" policy show "$p")"

printf '%s\n' "${cases[@]}" | jq -s '{cases: .,
  rejected: ([.[] | select(.expected == "rejected" and .as_expected)] | length),
  rejected_total: ([.[] | select(.expected == "rejected")] | length),
  controls_ok: ([.[] | select(.expected == "accepted" and .as_expected)] | length),
  controls_total: ([.[] | select(.expected == "accepted")] | length)}
  | .pass = (.rejected == .rejected_total and .controls_ok == .controls_total)' >"$out"
jq -c '{rejected, rejected_total, controls_ok, controls_total, pass}' "$out"
jq -e .pass "$out" >/dev/null
