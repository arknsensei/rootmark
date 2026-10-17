#!/usr/bin/env bash
# P1-E deliberate corruptions: each must be rejected, each untouched control
# must pass.
#   contract-http (E0, E-16; ADDENDUM Ş4): scripts/contract_check.sh's HTTP
#     step against a server build whose HTTP route for one contract tool
#     (`checkpoints`) is removed while MCP still lists it → the check fails,
#     and only its HTTP step fails (tools and codes still equal); a contract
#     copy whose §3 row gives `index_stale` another status → fails;
#   compare-fields (E0, E-13; K-E0.2): a declaration copy without one
#     excluded field's row, with a row the code does not exclude, or with a
#     pair exclusion relabelled `regress`/`karar` → check_compare_fields fails;
#   client-types (E1): a generated contract.gen.ts with a tool outside the
#     contract → scripts/gen_client_types.sh --check fails;
#   plugin-e2e (E4): the harness sending a request outside the contract
#     (E-4) or writing a Markdown file outside the propose step (E-3) →
#     scripts/plugin_e2e.sh fails.
# Writes <out.json>: {cases, rejected, rejected_total, controls_ok,
# controls_total, pass}; exit 1 unless pass.
#
#   scripts/test_tamper_p1e.sh <out.json>
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:?usage: test_tamper_p1e.sh <out.json>}"
work="$(mktemp -d -t rootmark-tamper-p1e.XXXXXX)"
trap 'rm -rf "$work"' EXIT
export ROOTMARK_MODELS="${ROOTMARK_MODELS:-$here/models}"
cd "$here"
cargo build -q --release -p rootmark-server --bin rootmark --example mcp_pipeline
cases=()
add() { # add <group> <case> <expected rejected|accepted> <rc> [<detail JSON>]
  local got=accepted
  [ "$4" != 0 ] && got=rejected
  cases+=("$(jq -n --arg g "$1" --arg c "$2" --arg e "$3" --arg got "$got" --argjson d "${5:-null}" \
    '{group: $g, case: $c, expected: $e, got: $got, detail: $d,
      as_expected: ($e == $got and ($d == null or $d.as_expected))}')")
}
run() { local rc=0; "$@" >/dev/null 2>&1 || rc=$?; echo "$rc"; }

# ---- contract-http
add contract-http "untouched contract and server" accepted "$(run bash scripts/contract_check.sh "$work/cc0.json")"

# the same sources with the HTTP route of `checkpoints` removed (404 before
# the tool runs); built into its own target directory
src="$work/src"
mkdir -p "$src"
git ls-files -z -- Cargo.toml Cargo.lock rust-toolchain.toml core index server bindings spec \
  | tar --null -T - -cf - | tar -x -C "$src"
perl -0pi -e 's{(    // arrival: a writing call takes its place in the writer queue now\n)}{    if tool == "checkpoints" {\n        return err(StatusCode::NOT_FOUND, Code::NotFound, "route removed (deliberate test)");\n    }\n$1}' \
  "$src/server/src/http.rs"
grep -q 'route removed (deliberate test)' "$src/server/src/http.rs" \
  || { echo "test_tamper_p1e: could not patch http.rs" >&2; exit 1; }
(cd "$src" && CARGO_TARGET_DIR="$here/target/tamper-p1e" cargo build -q --release -p rootmark-server --bin rootmark)
rc="$(ROOTMARK_BIN="$here/target/tamper-p1e/release/rootmark" run bash scripts/contract_check.sh "$work/cc1.json")"
detail="$(jq -c '{tools_equal: .tools.equal, codes_equal: .codes.equal, http_pass: .http.pass,
  failed_endpoints: [.http.endpoints[] | select(.ok | not) | .endpoint],
  as_expected: (.tools.equal and .codes.equal and (.http.pass | not)
    and ([.http.endpoints[] | select(.ok | not) | .endpoint] == ["POST /v1/checkpoints"]))}' "$work/cc1.json")"
add contract-http "HTTP endpoint POST /v1/checkpoints removed (MCP still lists the tool)" rejected "$rc" "$detail"

c="$work/contract.md"
sed 's/^| `index_stale` | 503 |/| `index_stale` | 500 |/' docs/CLIENT_CONTRACT.md >"$c"
cmp -s "$c" docs/CLIENT_CONTRACT.md && { echo "test_tamper_p1e: index_stale row not found" >&2; exit 1; }
add contract-http "status of index_stale changed to 500" rejected "$(ROOTMARK_CONTRACT="$c" run bash scripts/contract_check.sh)"

# ---- compare-fields (E0, E-13; K-E0.2)
d="$work/DETERMINISTIC_FIELDS.md"
cp research/eval/DETERMINISTIC_FIELDS.md "$d"
add compare-fields "untouched declaration" accepted "$(ROOTMARK_DETERMINISTIC_FIELDS="$d" run bash scripts/check_compare_fields.sh)"
grep -v '^| `compare_p1d1` | hepsi | `p1d1/d1_5.json` | `responses_a_vs_b_equal`' research/eval/DETERMINISTIC_FIELDS.md >"$d"
cmp -s "$d" research/eval/DETERMINISTIC_FIELDS.md && { echo "test_tamper_p1e: declaration row not found" >&2; exit 1; }
add compare-fields "an excluded field's row removed (undeclared exclusion in code)" rejected "$(ROOTMARK_DETERMINISTIC_FIELDS="$d" run bash scripts/check_compare_fields.sh)"
{ cat research/eval/DETERMINISTIC_FIELDS.md; } | sed 's#^| `compare_p1d1` | hepsi | `p1d1/d1_5.json` | `elapsed_ms_lockstep` | duvar-saati |#&#; /^<!-- \/fields-table -->/i\
| `compare_p1d1` | hepsi | `p1d1/d1_5.json` | `write_responses_a_vs_b_equal` | bilgi | deliberate: a deterministic field declared nondeterministic |' >"$d"
add compare-fields "a row the code does not exclude added" rejected "$(ROOTMARK_DETERMINISTIC_FIELDS="$d" run bash scripts/check_compare_fields.sh)"
sed 's#^\(| `compare_p1d1` | \)hepsi\( | `p1d1/d1_5.json` | `elapsed_ms_lockstep` | \)duvar-saati#\1regress\2karar#' research/eval/DETERMINISTIC_FIELDS.md >"$d"
add compare-fields "a pair exclusion relabelled as a decision" rejected "$(ROOTMARK_DETERMINISTIC_FIELDS="$d" run bash scripts/check_compare_fields.sh)"

# ---- client-types (E1): the generated contract types drift from the contract
g="$work/contract.gen.ts"
cp clients/obsidian/src/contract.gen.ts "$g"
add client-types "untouched generated types" accepted "$(ROOTMARK_CLIENT_TYPES="$g" run bash scripts/gen_client_types.sh --check)"
sed 's/"trace"\] as const/"trace","demote"] as const/' clients/obsidian/src/contract.gen.ts >"$g"
cmp -s "$g" clients/obsidian/src/contract.gen.ts && { echo "test_tamper_p1e: TOOLS line not found" >&2; exit 1; }
add client-types "a tool outside the contract added to the generated types" rejected "$(ROOTMARK_CLIENT_TYPES="$g" run bash scripts/gen_client_types.sh --check)"
# the generator refuses schemas it cannot type (fed a copy of a live tools/list)
tl="$work/tools.json"
jq -n '{tools: [{name: "status", inputSchema: {type: "object", additionalProperties: false, properties: {}, required: []}}]}' >"$tl"
gen() { node clients/obsidian/scripts/gen-types.mjs "$1" "$work/contract_one.md"; }
# a one-tool contract (status only) beside the schema copies
{ echo; echo '## 2. Araçlar'; echo '| `status` | hayır | — | x |'; echo; echo '## 3. Kodlar'; echo '| `internal` | 500 | x |'; echo; echo '## 4. Son'; } >"$work/contract_one.md"
add client-types "generator: one closed schema (control)" accepted "$(run gen "$tl")"
jq '.tools[0].inputSchema.oneOf = [{required: ["x"]}]' "$tl" >"$work/t1.json"
add client-types "generator: top-level oneOf refused" rejected "$(run gen "$work/t1.json")"
jq '.tools[0].inputSchema.required = ["ghost"]' "$tl" >"$work/t2.json"
add client-types "generator: required field without a property refused" rejected "$(run gen "$work/t2.json")"

# ---- plugin-e2e (E4, E-3/E-4): the harness must catch what it measures
pe() { ROOTMARK_E2E_TAMPER="$1" ROOTMARK_E2E_TAMPER_PATH="/v1/demote" bash scripts/plugin_e2e.sh "$work/pe_$1"; }
add plugin-e2e "untouched harness" accepted "$(run pe none)"
add plugin-e2e "a request outside the contract (POST /v1/demote) as the plugin" rejected "$(run pe offcontract)" \
  "$(jq -c '{outside: .e4_access_log.outside_contract, as_expected: (.e4_access_log.outside_contract == 1)}' "$work/pe_offcontract/plugin_e2e.json")"
add plugin-e2e "a Markdown write outside the propose step" rejected "$(run pe mdwrite)" \
  "$(jq -c '{e3: .e3_md_changes_only_at_propose, as_expected: (.e3_md_changes_only_at_propose == false)}' "$work/pe_mdwrite/plugin_e2e.json")"

printf '%s\n' "${cases[@]}" | jq -s '{cases: .,
  rejected: ([.[] | select(.expected == "rejected" and .as_expected)] | length),
  rejected_total: ([.[] | select(.expected == "rejected")] | length),
  controls_ok: ([.[] | select(.expected == "accepted" and .as_expected)] | length),
  controls_total: ([.[] | select(.expected == "accepted")] | length)}
  | .pass = (.rejected == .rejected_total and .controls_ok == .controls_total)' >"$out"
jq -c '{rejected, rejected_total, controls_ok, controls_total, pass}' "$out"
jq -e .pass "$out" >/dev/null
