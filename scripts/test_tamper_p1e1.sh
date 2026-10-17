#!/usr/bin/env bash
# P1-E.1 deliberate corruptions: each must be rejected, each untouched control
# must pass.
#   contract-status (E1.1, E1-8; K-E1.2): scripts/contract_check.sh against
#     the `p1e-final` server (the old `status`: a string `index`, 500 on an
#     unreadable manifest) → the check fails and only its status step fails
#     (tools, codes, endpoints and transport still pass); a contract copy
#     whose §2 `status` row lists no `index {…}` fields → fails;
#   old-index (E1.1, E1-1): the committed `/2` fixture regenerated with the
#     p1d-final `mdk` → byte-equal (control); a fixture copy relabelled to
#     the current format → server/tests/p1e1_status.rs E1-1 fails.
# Writes <out.json>: {cases, rejected, rejected_total, controls_ok,
# controls_total, pass}; exit 1 unless pass. MDK_P1E_FINAL_BIN / MDK_OLD:
# prebuilt p1e-final / p1d-final `mdk` (else built from the tags).
#
#   scripts/test_tamper_p1e1.sh <out.json>
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: test_tamper_p1e1.sh <out.json>}"
work="$(mktemp -d -t mdk-tamper-p1e1.XXXXXX)"
trap 'rm -rf "$work"' EXIT
export MDK_MODELS="${MDK_MODELS:-$here/models}"
cd "$here"
cargo build -q --release -p mdkernel-server --bin mdk --example mcp_pipeline
cases=()
add() { # add <group> <case> <expected rejected|accepted> <rc> [<detail JSON>]
  local got=accepted
  [ "$4" != 0 ] && got=rejected
  cases+=("$(jq -n --arg g "$1" --arg c "$2" --arg e "$3" --arg got "$got" --argjson d "${5:-null}" \
    '{group: $g, case: $c, expected: $e, got: $got, detail: $d,
      as_expected: ($e == $got and ($d == null or $d.as_expected))}')")
}
run() { local rc=0; "$@" >/dev/null 2>&1 || rc=$?; echo "$rc"; }
old_build() { # old_build <tag> → path of its `mdk` (its own tree and target/)
  mkdir -p "$work/$1"
  git archive "$(git rev-parse --verify "$1^{commit}")" | tar -xf - -C "$work/$1"
  (cd "$work/$1" && cargo build -q --release -p mdkernel-server --bin mdk)
  echo "$work/$1/target/release/mdk"
}

# ---- contract-status
add contract-status "untouched contract and server" accepted "$(run bash scripts/contract_check.sh "$work/cc0.json")"
p1e_bin="${MDK_P1E_FINAL_BIN:-$(old_build p1e-final)}"
rc="$(MDK_BIN="$p1e_bin" run bash scripts/contract_check.sh "$work/cc1.json")"
detail="$(jq -c '{tools_equal: .tools.equal, codes_equal: .codes.equal,
  endpoints_ok: all(.http.endpoints[]; .ok), transport_ok: all(.http.transport[]; .ok),
  failed_states: [.http.status_index[] | select(.ok | not) | .state],
  as_expected: (.tools.equal and .codes.equal and all(.http.endpoints[]; .ok) and all(.http.transport[]; .ok)
    and (.http.pass | not)
    and ([.missing[] | select(startswith("http-status-index:") | not)]
      | all(startswith("http-endpoint:GET /v1/status") or startswith("http-endpoint:POST /v1/status") or startswith("http-code:internal")))
    and ([.http.status_index[] | select(.ok | not)] | length) >= 3)}' "$work/cc1.json")"
add contract-status "p1e-final server (string index, status 500 on an unreadable manifest)" rejected "$rc" "$detail"

c="$work/contract.md"
sed -E 's/index \{state, format, expected_format, auto_rebuild\}/index/' docs/CLIENT_CONTRACT.md >"$c"
cmp -s "$c" docs/CLIENT_CONTRACT.md && { echo "test_tamper_p1e1: status row index fields not found" >&2; exit 1; }
add contract-status "status row without index {…} fields" rejected "$(MDK_CONTRACT="$c" run bash scripts/contract_check.sh)"

# ---- old-index
add old-index "fixture regenerated with the p1d-final mdk" accepted "$(run bash scripts/gen_old_index.sh --check)"
fx="$here/server/tests/fixtures/index_v2/vault/.mdkernel/index/manifest.json"
cp "$fx" "$work/manifest.keep"
trap 'cp "$work/manifest.keep" "$fx"; rm -rf "$work"' EXIT
jq -c '.format = "mdkernel-index/3"' "$work/manifest.keep" >"$fx"
rc="$(run cargo test -q --release -p mdkernel-server --test p1e1_status e1_1)"
cp "$work/manifest.keep" "$fx"
add old-index "fixture manifest relabelled to the current format" rejected "$rc"

printf '%s\n' "${cases[@]}" | jq -s '{cases: .,
  rejected: ([.[] | select(.expected == "rejected" and .as_expected)] | length),
  rejected_total: ([.[] | select(.expected == "rejected")] | length),
  controls_ok: ([.[] | select(.expected == "accepted" and .as_expected)] | length),
  controls_total: ([.[] | select(.expected == "accepted")] | length)}
  | . + {pass: (.rejected == .rejected_total and .controls_ok == .controls_total)}' >"$out"
jq -r '"test_tamper_p1e1: rejected \(.rejected)/\(.rejected_total), controls \(.controls_ok)/\(.controls_total)"' "$out"
jq -e .pass "$out" >/dev/null
