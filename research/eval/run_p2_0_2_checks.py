"""P2-0.2 hypotheses recorded in the RUN (GOAL P2-0.2 "Önceden kayıtlı
eşikler"), so that the round report is built from one RUN directory only:
`p2_0_2_checks.json`. Every hypothesis names its source file.

Read from what `make p2-0-2-checks` wrote into the RUN:

- H-P2-0.2a: `p2_0_2/d_a.json` (P2-0's D-A) and `p2_0_2/d_a2.json`
  (P2-0.1's D-A2): canonical leak 0 in both; a′: `p2_0_2/d_a.json` 200/200;
- H-P2-0.2b, c: `p2_0_2/d_b.json` (D-B test-3 in the real Obsidian;
  `skipped` without an Obsidian binary → not measured); b′ (information):
  the rename sessions only and the per-format report (`renames`);
- H-P2-0.2d: `p2_0_2/d_c.json` (≥ 0.99) and `p2_0_2/d_c_prime.json` (≥ 0.95,
  agent canonical 0);
- H-P2-0.2e: `p2_0_2/d_a_prime.json` — agent canonical 0 (50), fakes denied
  10/10, agent canonical in the rename carry scenarios 0 (10);
- H-P2-0.2f: `p2_0_2/d_b2.json`; H-P2-0.2h: `p2_0_2/d_e.json` (6/6);
- H-P2-0.2j: the conformance report `conformance_p1d1.json` (39/39, level
  3, spec 2.0.2, C2-14 passed);
- H-P2-0.2k: `p2_0_2/rules.json` (the written rules — DEV_PROCESS §5, the
  CRITERION clause, spec §8's reading rule — and the reader's
  forward-compatibility test log);
- the FAIL summary (`p2_0_2/historical.json`, the round's classifier, also
  the chain's printed summary), regress (`regress_vs_p2_0_1.json`) and
  tamper (`p2_0_2/tamper_p2_0_2.json`) are recorded beside them;
- H-P2-0.2i is the A ↔ B half (`pair_compare.json`, `eval.compare_p2_0_2
  pair`), read by the report, not here.

The thresholds are GOAL P2-0.2's (P2-0's), unchanged.

    python -m eval.run_p2_0_2_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import runinfo
from eval.datasets.common import die

SPEC = "2.0.2"
TOTAL = 39


def _load(path: Path):
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing (make p2-0-2-checks)")
    return json.loads(path.read_text(encoding="utf-8"))


def _skipped(rec: dict) -> bool:
    return rec.get("skipped") is True


NOT_MEASURED = {"skipped": True, "measured": False, "pass": None}


def _w(d: Path) -> float:
    """W of the RUN's P2-0.2 phase (`runinfo.record_phase` params)."""
    phases = _load(d / "manifest.json")["phases"]
    ws = {v["params"]["window_s"] for k, v in phases.items() if k.startswith("p2_0_2_test")}
    if len(ws) != 1:
        raise ValueError(f"the RUN's P2-0.2 phases name {len(ws)} windows")
    return ws.pop()


def _by_kind(db: dict) -> dict:
    """The D-B test-3 records by session kind: new blocks, still `unverified`."""
    out: dict[str, dict[str, int]] = {}
    for r in db["records"]:
        k = out.setdefault(r["kind"], {"saves": 0, "new_blocks": 0, "unverified": 0})
        k["saves"] += 1
        k["new_blocks"] += r["new_blocks"]
        k["unverified"] += r["unverified_after_window"]
    return dict(sorted(out.items()))


def checks(d: Path) -> dict:
    p = d / "p2_0_2"
    da, da2 = _load(p / "d_a.json"), _load(p / "d_a2.json")
    db, dc, dcp = _load(p / "d_b.json"), _load(p / "d_c.json"), _load(p / "d_c_prime.json")
    dap, db2, de = _load(p / "d_a_prime.json"), _load(p / "d_b2.json"), _load(p / "d_e.json")
    rules = _load(p / "rules.json")
    conf = _load(d / "conformance_p1d1.json")
    if _skipped(db):
        src = {"source": "p2_0_2/d_b.json"}
        h_b = {**NOT_MEASURED, "reason": db.get("reason"), **src}
        h_b1, h_c = {**NOT_MEASURED, **src}, {**NOT_MEASURED, **src}
    else:
        rate, p95 = db["false_unverified_rate"], db["latency_p95_ms"]
        h_b = {
            "seed": db["seed"],
            "sessions": db["sessions"],
            "saves": db["saves"],
            "new_blocks": db["new_blocks"],
            "unverified_after_window": db["unverified_after_window"],
            "false_unverified_rate": rate,
            "by_kind": _by_kind(db),
            "side_notes": db["side_notes"],
            "link_settings": db.get("link_settings"),
            "coverage": db["coverage"],
            "source": "p2_0_2/d_b.json",
            "pass": rate is not None and rate <= 0.01 and db["sessions"] == 200,
        }
        h_b1 = {**db["renames"], "source": "p2_0_2/d_b.json (renames)", "pass": None}
        h_c = {
            "latency_p95_ms": p95,
            "latency_measured": db["latency_measured"],
            "latency_unsettled": db["latency_unsettled"],
            "latency_p95_over_window": db["latency_p95_over_window"],
            "no_write": db["no_write"],
            "source": "p2_0_2/d_b.json",
            "pass": p95 is not None and p95 < 500 and db["latency_p95_over_window"] is False,
        }
    d_c = {
        "ticks": dc["ticks"],
        "blocks": dc["blocks"],
        "right": dc["right"],
        "accuracy": dc["accuracy"],
        "agent_canonical": dc["agent_canonical"],
        "by_mode": dc["by_mode"],
        "partial_lines": dc["partial_lines"],
        "pass": dc["pass"] is True,
    }
    d_cp = (
        {**NOT_MEASURED, "reason": dcp.get("reason")}
        if _skipped(dcp)
        else {
            "ticks": dcp["ticks"],
            "human_blocks": dcp["human_blocks"],
            "human_canonical": dcp["human_canonical"],
            "human_accuracy": dcp["human_accuracy"],
            "agent_canonical": dcp["agent_canonical"],
            "by_delay": dcp["by_delay"],
            "pass": dcp["ticks"] == 40
            and dcp["human_accuracy"] is not None
            and dcp["human_accuracy"] >= 0.95
            and dcp["agent_canonical"] == 0,
        }
    )
    h_e = (
        {**NOT_MEASURED, "reason": dap.get("reason"), "source": "p2_0_2/d_a_prime.json"}
        if _skipped(dap)
        else {
            "scenarios": dap["scenarios"],
            "agent_blocks": dap["agent_blocks"],
            "agent_canonical": dap["agent_canonical"],
            "by_kind": dap["by_kind"],
            "links_rewritten": dap["links_rewritten"],
            "fakes": dap["fakes"],
            "fakes_denied": dap["fakes_denied"],
            "carry_scenarios": dap["carry_scenarios"],
            "carry_agent_blocks": dap["carry_agent_blocks"],
            "carry_agent_canonical": dap["carry_agent_canonical"],
            "carry_human_canonical_before": dap["carry_human_canonical_before"],
            "carry_human_canonical_after": dap["carry_human_canonical_after"],
            "link_settings": dap.get("link_settings"),
            "source": "p2_0_2/d_a_prime.json",
            "pass": dap["scenarios"] == 50
            and dap["agent_canonical"] == 0
            and dap["fakes"] == 10
            and dap["fakes_denied"] == 10
            and dap["carry_scenarios"] == 10
            and dap["carry_agent_canonical"] == 0,
        }
    )
    tests = {t["id"]: t for t in conf["tests"]}
    return {
        # the W the RUN measured with (X4 review 4b); eval.run_p2_0_2 reads it
        # from docs/rounds/P2-0/progress/Z5_tuning.json
        "window": {"window_s": _w(d), "source": "manifest.json (p2_0_2_test window_s)"},
        "H-P2-0.2a": {
            "d_a": {
                "scenarios": da["scenarios"],
                "new_blocks": da["new_blocks"],
                "canonical_leak": da["canonical_leak"],
            },
            "d_a2": {
                "scenarios": da2["scenarios"],
                "new_blocks": da2["new_blocks"],
                "canonical_leak": da2["canonical_leak"],
                "by_path": da2["by_path"],
            },
            "source": "p2_0_2/d_a.json, p2_0_2/d_a2.json",
            "pass": da["canonical_leak"] == 0
            and da["pass_0a"] is True
            and da["scenarios"] == 200
            and da2["canonical_leak"] == 0
            and da2["pass"] is True,
        },
        "H-P2-0.2a′": {
            "scenarios": f"{da['scenarios_0a_prime']}/{da['scenarios']}",
            "canonical_changed": da["canonical_changed"],
            "source": "p2_0_2/d_a.json",
            "pass": da["pass_0a_prime"] is True and da["scenarios_0a_prime"] == 200,
        },
        "H-P2-0.2b": h_b,
        "H-P2-0.2b′": h_b1,
        "H-P2-0.2c": h_c,
        "H-P2-0.2d": {
            "d_c": d_c,
            "d_c_prime": d_cp,
            "source": "p2_0_2/d_c.json, p2_0_2/d_c_prime.json",
            # a real-Obsidian half not measured leaves the hypothesis unmeasured
            # a D-C failure stands even when D-C′ was not measured (X4 review 4a)
            "pass": False
            if d_c["pass"] is False
            else None
            if d_cp["pass"] is None
            else d_c["pass"] and d_cp["pass"],
        },
        "H-P2-0.2e": h_e,
        "H-P2-0.2f": {
            "sessions": db2["sessions"],
            "new_blocks": db2["new_blocks"],
            "unverified_after_window": db2["unverified_after_window"],
            "false_unverified_rate": db2["false_unverified_rate"],
            "latency_p95_ms": db2["latency_p95_ms"],
            "latency_unsettled": db2["latency_unsettled"],
            "by_kind": db2["by_kind"],
            "source": "p2_0_2/d_b2.json",
            "pass": db2["pass"] is True,
        },
        "H-P2-0.2h": {
            "vaults": f"{de['passed']}/{len(de['vaults'])}",
            "failed": [v["vault"] for v in de["vaults"] if not v["pass"]],
            "source": "p2_0_2/d_e.json",
            "pass": de["pass"] is True,
        },
        "H-P2-0.2j": {
            "level_claimed": conf["level_claimed"],
            "spec_version": conf["spec_version"],
            "passed": conf["passed"],
            "total": conf["total"],
            "new": {"C2-14": tests.get("C2-14", {}).get("result")},
            "source": "conformance_p1d1.json",
            "pass": conf["level_claimed"] == 3
            and conf["spec_version"] == SPEC
            and conf["passed"] == TOTAL
            and conf["total"] == TOTAL
            and tests.get("C2-14", {}).get("result") == "pass",
        },
        "H-P2-0.2k": {**rules, "source": "p2_0_2/rules.json"},
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    out = checks(d)
    p = d / "p2_0_2"
    hist = _load(p / "historical.json")
    tamper = _load(p / "tamper_p2_0_2.json")
    reg = _load(d / "regress_vs_p2_0_1.json")
    out["fail_summary"] = {
        "counted_fails": hist["counted_fails"],
        "historical": hist["historical"],
        "counted": hist["counted"],
        "reference_run": hist["reference_run"],
        "source": "p2_0_2/historical.json (eval.historical_p2_0_2 = the chain's summary)",
        "pass": hist["counted_fails"] == 0,
    }
    out["regress"] = {
        "reference_run": reg["reference_run"],
        "n_fields": reg["n_fields"],
        "differing_fields": reg["differing_fields"],
        "excluded_differences": reg["excluded_differences"],
        "exclusions_without_difference": reg["exclusions_without_difference"],
        "conformance": reg["conformance"],
        "p2_0_1_sets": reg["p2_0_1_sets"],
        "source": "regress_vs_p2_0_1.json",
        "pass": reg["equal"] is True,
    }
    out["tamper"] = {
        "rejected": f"{tamper['rejected']}/{tamper['rejected_total']}",
        "controls": f"{tamper['controls_ok']}/{tamper['controls_total']}",
        "source": "p2_0_2/tamper_p2_0_2.json",
        "pass": tamper["pass"] is True,
    }
    runinfo.write_json(a.run, "p2_0_2_checks.json", out)
    runinfo.record_phase(a.run, "p2_0_2_checks", {}, ["p2_0_2_checks.json"])
    print(
        "; ".join(
            f"{k} {'-' if not isinstance(v, dict) or v.get('pass') is None else 'pass' if v['pass'] else 'FAIL'}"
            for k, v in out.items()
            if k != "window"
        )
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
