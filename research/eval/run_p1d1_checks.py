"""P1-D.1 criteria recorded in the RUN (GOAL D1-1..D1-13), so that the round
report is built from one RUN directory only: `p1d1_checks.json`.

Read from what `make p1d1-checks` (and `make p1c1-checks`) wrote into the
RUN:

- D1-1..D1-9, D1-11, D1-13: `p1d1/d1_<n>.json` (`scripts/test_p1d1.sh`:
  `server/tests/p1d1_criteria.rs`, the p1d-final vaults, the contract check).
- D1-10: `conformance_p1d1.json` (the full report: 34 tests, level 3, the
  new C2-10, C3-06, C3-07 passing) and the spec version the CRITERION rule
  gives (`SPEC_VERSION`; docs/rounds/P1-D.1/progress/SPEC_EDITS.md).
- D1-12: within this RUN, D1-5's two pipelined vaults (and the lock-step
  one) and `regress_vs_p1d.json` (`eval.compare_p1d1 regress` against the
  P1-D official RUN, decision exclusions only); the A ↔ B half is
  `pair_compare.json` (`eval.compare_p1d1 pair`), read by the report.
- tamper: `p1d1/tamper_p1d1.json` (`scripts/test_tamper_p1d1.sh`).

    python -m eval.run_p1d1_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import runinfo
from eval.datasets.common import die

SPEC_VERSION = "1.0.3"
NEW_TESTS = ("C2-10", "C3-06", "C3-07")


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing (make p1d1-checks)")
    return json.loads(p.read_text(encoding="utf-8"))


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    c = {n: _load(d, f"p1d1/d1_{n}.json") for n in (1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 13)}
    conf = _load(d, "conformance_p1d1.json")
    reg = _load(d, "regress_vs_p1d.json")
    tamper = _load(d, "p1d1/tamper_p1d1.json")
    by_id = {t["id"]: t for t in conf["tests"]}
    new = {i: by_id.get(i, {}).get("result") for i in NEW_TESTS}
    out = {
        "D1-1": {
            "denied": f"{c[1]['denied']}/6",
            "cases": {x["case"]: x["code"] for x in c[1]["cases"]},
            "files_and_events_unchanged": c[1]["files_and_events_unchanged"],
            "pass": c[1]["pass"],
        },
        "D1-2": {
            "accepted": c[2]["accepted"],
            "section_layers": c[2]["section_layers"],
            "pass": c[2]["pass"],
        },
        "D1-3": {
            "writes": c[3]["writes"],
            "outcomes": c[3]["outcomes"],
            "accepted": c[3]["accepted"],
            "comparisons": c[3]["comparisons"],
            "differing": c[3]["differing"],
            "pass": c[3]["pass"],
        },
        "D1-4": {
            "event_actor": c[4]["event_actor"],
            "history_actor": c[4]["history_actor"],
            "pass": c[4]["pass"],
        },
        "D1-5": {
            "calls": c[5]["calls"],
            "propose": c[5]["propose"],
            "pipelined_a_vs_b": c[5]["pipelined_a_vs_b"],
            "pipelined_vs_lockstep": c[5]["pipelined_vs_lockstep"],
            "pass": c[5]["pass"],
        },
        "D1-6": {
            "correct": f"{c[6]['correct']}/{c[6]['calls']}",
            "log_and_traces_unchanged": c[6]["log_and_traces_unchanged"],
            "measured": {
                "ratio": c[6]["ratio"],
                "ratio_below_0_6": c[6]["ratio_below_0_6"],
                "elapsed_ms_serial": c[6]["elapsed_ms_serial"],
                "elapsed_ms_pipelined": c[6]["elapsed_ms_pipelined"],
            },
            "pass": c[6]["pass"],
        },
        "D1-7": {
            "passed": f"{c[7]['passed']}/3",
            "mcp_promote": c[7]["mcp_promote"]["decision"],
            "http_without_human_id": c[7]["http_without_human_id"]["status"],
            "http_promote_actor": c[7]["http_promote"]["actor"],
            "pass": c[7]["pass"],
        },
        "D1-8": {
            "passed": f"{c[8]['passed']}/{c[8]['of']}",
            "events_unchanged": c[8]["events_unchanged"],
            "pass": c[8]["pass"],
        },
        "D1-9": {"passed": f"{c[9]['passed']}/2", "pass": c[9]["pass"]},
        "D1-10": {
            "passed": f"{conf['passed']}/{conf['total']}",
            "level_claimed": conf["level_claimed"],
            "spec_version": conf["spec_version"],
            "spec_version_by_criterion": SPEC_VERSION,
            "new_tests": new,
            "pass": conf["passed"] == conf["total"] == 34
            and conf["level_claimed"] == 3
            and conf["spec_version"] == SPEC_VERSION
            and all(v == "pass" for v in new.values()),
        },
        "D1-11": {
            "vaults": {
                v["vault"]: {
                    "docs": v["docs"],
                    "replay_equal": v["replay_equal"],
                    "events_hash_equal": v["events_hash_equal"],
                    "events_bytes_unchanged": v["events_bytes_unchanged"],
                    "pass": v["pass"],
                }
                for v in c[11]["vaults"]
            },
            "v3_final_compat": c[11]["v3_final_compat"],
            "pass": c[11]["pass"],
        },
        "D1-12": {
            "d1_5_runs_equal": all(c[5]["pipelined_a_vs_b"].values()),
            "regress_vs_p1d": {
                "reference_run": reg["reference_run"],
                "n_fields": reg["n_fields"],
                "differing_fields": reg["differing_fields"],
                "excluded_differences": reg["excluded_differences"],
                "exclusions_without_difference": reg["exclusions_without_difference"],
                "conformance": reg["conformance"],
                "equal": reg["equal"],
            },
            "pair": {"decided_by": "eval.compare_p1d1 pair (pair_compare.json)"},
            "pass": all(c[5]["pipelined_a_vs_b"].values()) and reg["equal"],
        },
        "D1-13": {
            "tools": len(c[13]["tools"]["contract"]),
            "codes": len(c[13]["codes"]["contract"]),
            "missing": c[13]["missing"],
            "tools_equal": c[13]["tools"]["equal"],
            "codes_equal": c[13]["codes"]["equal"],
            "pass": c[13]["pass"] and not c[13]["missing"],
        },
        "tamper": {
            "rejected": f"{tamper['rejected']}/{tamper['rejected_total']}",
            "controls": f"{tamper['controls_ok']}/{tamper['controls_total']}",
            "pass": tamper["pass"],
        },
    }
    runinfo.write_json(a.run, "p1d1_checks.json", out)
    runinfo.record_phase(a.run, "p1d1_checks", {}, ["p1d1_checks.json"])
    print("; ".join(f"{k} {'pass' if out[k]['pass'] else 'FAIL'}" for k in out))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
