"""P1-D criteria recorded in the RUN (GOAL D-1..D-11), so that the round
report is built from one RUN directory only: `p1d_checks.json`.

Read from what `make p1d-checks` wrote into the RUN (`p1d/`):

- D-1: `p1d/inspector/inspector.json` (`scripts/mcp_inspector.sh`: the
  official MCP Inspector's `tools/list`, every schema against the JSON Schema
  2020-12 meta-schema, one Inspector call per tool). GOAL says 9 tools, its
  table lists 10 (ASSUMPTIONS D5-1): all listed tools must pass, at least 9.
- D-2, D-3, D-4, D-5, D-6, D-11 (MCP half): `p1d/mcp_e2e/mcp_e2e.json`
  (`scripts/mcp_e2e.sh`); each criterion is decided by the named checks in
  `E2E_CHECKS` (every one must be present and pass).
- D-7: `p1d/http/http_e2e.json` (`scripts/http_e2e.sh`, the five D7 cases).
- D-8: within this RUN, the e2e's two runs (`runs_equal`); the A ↔ B half is
  `pair_compare.json` (`eval.compare_p1d pair`, `verify.sh pair`), read by
  the report.
- D-9: `regress_vs_p1c1.json` (`eval.compare_p1d regress` against the P1-C.1
  official RUN: every field of the P1-C.1 pair set, conformance tests).
- D-10: `p1d/d10.json` (`server/tests/serve_watch.rs`: 20 external human
  edits, time until `search` finds each; p95 < 1000 ms, canonical, Human)
  and `p1d/sigterm.json` (SIGTERM during timer compaction never lost).
- D-11: the four MCP checks above and `p1d/tamper_p1d.json`'s `d-11` cases.
- scene: `p1d/scene/scene.json` (`scripts/scene_agent.sh`, TASKS D6).
- tamper: `p1d/tamper_p1d.json` (schema, token, X-Consumer, D-11).

    python -m eval.run_p1d_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import runinfo
from eval.datasets.common import die

GOAL_TOOLS = 9
E2E_CHECKS = {
    "D-3": [
        "D-3 file is the agent's text only (no lineage|version_id|rootmark); refused proposals wrote no file",
        "create: ok, every block proposal by Agent (source mcp-e2e/1.0)",
        "D-3 search by default does not find the proposal",
        "D-3 search with include_proposal finds it, labelled proposal",
    ],
    "D-4": [
        "D-4 MCP promote → decision denied, policy human_only (a response, not an error), layer proposal",
        "D-4 CLI promote --actor human:alice → Promoted (4 lineages)",
        "D-4 after the human promotion, default search finds the block (canonical)",
    ],
    "D-5": [
        "D-5: 200 calls → 200 traces, consumer mcp-e2e/1.0",
        "D-5 traces.jsonl: 207 lines = 1 CLI + 206 by mcp-e2e/1.0 (4 + 200 + 2 later searches), every load call ok",
    ],
    "D-6": [
        "D-6 search --at d6 gives the old state (checkpoint d6, no agent/d6.md)",
        "D-6 search gives the new state (agent/d6.md, canonical)",
    ],
    "D-11": [
        "D-11 ../ path refused",
        "D-11 path outside the vault refused",
        "D-11 content over 10 MB refused (max_propose_bytes)",
        "D-11 replace_section with a missing heading refused",
    ],
}


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing (make p1d-checks)")
    return json.loads(p.read_text(encoding="utf-8"))


def named(e2e: dict, labels: list[str]) -> dict:
    by = {c["check"]: c["pass"] for c in e2e["checks"]}
    missing = [x for x in labels if x not in by]
    if missing:
        raise KeyError(f"mcp_e2e.json lacks checks {missing}")
    res = {x: by[x] for x in labels}
    return {
        "checks": res,
        "passed": f"{sum(res.values())}/{len(labels)}",
        "pass": all(res.values()),
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    insp = _load(d, "p1d/inspector/inspector.json")
    e2e = _load(d, "p1d/mcp_e2e/mcp_e2e.json")
    http = _load(d, "p1d/http/http_e2e.json")
    d10 = _load(d, "p1d/d10.json")
    sig = _load(d, "p1d/sigterm.json")
    scene = _load(d, "p1d/scene/scene.json")
    tamper = _load(d, "p1d/tamper_p1d.json")
    reg = _load(d, "regress_vs_p1c1.json")
    d11_tamper = [
        c for c in tamper["cases"] if c["group"] == "d-11" and c["expected"] == "rejected"
    ]
    out = {
        "D-1": {
            "inspector": insp["inspector"],
            "tools_listed": insp["tools_listed"],
            "schemas_valid": f"{insp['schemas_valid']}/{insp['tools_expected']}",
            "inspector_calls_ok": f"{insp['calls_ok']}/{insp['tools_expected']}",
            "goal_tools": GOAL_TOOLS,
            "pass": insp["pass"] and insp["schemas_valid"] >= GOAL_TOOLS,
        },
        "D-2": {
            "tools_passed": f"{e2e['tools_passed']}/{e2e['tools_total']}",
            "per_tool": e2e["per_tool"],
            "handshake": e2e["handshake_pass"],
            "pass": e2e["tools_passed"] == e2e["tools_total"] >= GOAL_TOOLS
            and e2e["handshake_pass"],
        },
        **{k: named(e2e, E2E_CHECKS[k]) for k in ("D-3", "D-4", "D-5", "D-6")},
        "D-7": {
            "cases": {c["id"]: c["pass"] for c in http["cases"] if c["id"].startswith("D7-")},
            "passed": f"{http['d7_passed']}/{http['d7_total']}",
            "other_http_cases": f"{http['passed'] - http['d7_passed']}/{http['total'] - http['d7_total']}",
            "pass": http["d7_passed"] == http["d7_total"] == 5 and http["pass"],
        },
        "D-8": {
            "e2e_two_runs_equal": e2e["runs_equal"],
            "pair": {"decided_by": "eval.compare_p1d pair (pair_compare.json)"},
            "pass": e2e["runs_equal"],
        },
        "D-9": {
            "reference_run": reg["reference_run"],
            "n_fields": reg["n_fields"],
            "differing_fields": reg["differing_fields"],
            "conformance": reg["conformance"],
            "pass": reg["equal"],
        },
        "D-10": {
            "measured": {
                "edits": d10["edits"],
                "p95_ms": d10["p95_ms"],
                "max_ms": d10["max_ms"],
            },
            "threshold_p95_ms": 1000,
            "layers_canonical_human": d10["layers_canonical_human"],
            "sigterm_compacts": d10["sigterm_compacted"] and d10["sigterm_exit_0"],
            "sigterm_never_lost": sig["pass"],
            "pass": d10["pass"] and sig["pass"],
        },
        "D-11": {
            "mcp": named(e2e, E2E_CHECKS["D-11"]),
            "tamper": {c["case"]: c["as_expected"] for c in d11_tamper},
            "pass": named(e2e, E2E_CHECKS["D-11"])["pass"]
            and len(d11_tamper) == 4
            and all(c["as_expected"] for c in d11_tamper),
        },
        "scene": {
            "passed": f"{scene['passed']}/{scene['total']}",
            "pass": scene["pass"],
        },
        "tamper": {
            "rejected": f"{tamper['rejected']}/{tamper['rejected_total']}",
            "controls": f"{tamper['controls_ok']}/{tamper['controls_total']}",
            "pass": tamper["pass"],
        },
    }
    runinfo.write_json(a.run, "p1d_checks.json", out)
    runinfo.record_phase(a.run, "p1d_checks", {}, ["p1d_checks.json"])
    print("; ".join(f"{c} {'pass' if out[c]['pass'] else 'FAIL'}" for c in out))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
