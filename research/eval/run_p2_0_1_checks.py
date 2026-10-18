"""P2-0.1 hypotheses recorded in the RUN (GOAL P2-0.1 "Önceden kayıtlı
eşikler"), so that the round report is built from one RUN directory only:
`p2_0_1_checks.json`. Every hypothesis names its source file.

Read from what `make p2-0-1-checks` wrote into the RUN:

- H-P2-0.1a, a′: `p2_0_1/d_a.json` (P2-0's D-A, unchanged);
- H-P2-0.1b, b′, c: `p2_0_1/d_b.json` (D-B test-2 in the real Obsidian;
  `skipped` without an Obsidian binary → not measured); b′ is the rename
  sessions' false-unverified count (information);
- H-P2-0.1d: `p2_0_1/d_c.json`; H-P2-0.1d′: `p2_0_1/d_c_prime.json`;
- H-P2-0.1e: `p2_0_1/d_a_prime.json`; H-P2-0.1f: `p2_0_1/d_b2.json`;
  H-P2-0.1g: `p2_0_1/d_a2.json`; H-P2-0.1h: `p2_0_1/d_e.json`;
- H-P2-0.1j: the conformance report `conformance_p1d1.json` (38/38, level
  3, spec 2.0.1);
- H-P2-0.1k: `p2_0_1/historical.json` (the round's classifier,
  `eval.historical_p2_0_1`, which is also the chain's printed FAIL summary):
  counted FAILs 0;
- regress: `regress_vs_p2_0.json` (`eval.compare_p2_0_1 regress` against the
  P2-0 official RUN); tamper: `p2_0_1/tamper_p2_0_1.json`.
- H-P2-0.1i is the A ↔ B half (`pair_compare.json`, `eval.compare_p2_0_1
  pair`), read by the report, not here.

The thresholds are GOAL P2-0.1's, unchanged.

    python -m eval.run_p2_0_1_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import runinfo
from eval.datasets.common import die


def _load(path: Path):
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing (make p2-0-1-checks)")
    return json.loads(path.read_text(encoding="utf-8"))


def _skipped(rec: dict) -> bool:
    return rec.get("skipped") is True


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    p = d / "p2_0_1"
    da = _load(p / "d_a.json")
    db = _load(p / "d_b.json")
    dc = _load(p / "d_c.json")
    dcp = _load(p / "d_c_prime.json")
    dap = _load(p / "d_a_prime.json")
    db2 = _load(p / "d_b2.json")
    da2 = _load(p / "d_a2.json")
    de = _load(p / "d_e.json")
    hist = _load(p / "historical.json")
    tamper = _load(p / "tamper_p2_0_1.json")
    conf = _load(d / "conformance_p1d1.json")
    reg = _load(d / "regress_vs_p2_0.json")
    not_measured = {"skipped": True, "measured": False, "pass": None}
    if _skipped(db):
        h_b = {**not_measured, "reason": db.get("reason"), "source": "p2_0_1/d_b.json"}
        h_b1 = {**not_measured, "source": "p2_0_1/d_b.json"}
        h_c = {**not_measured, "source": "p2_0_1/d_b.json"}
    else:
        rate = db["false_unverified_rate"]
        ren = [r for r in db["records"] if r["kind"] == "rename"]
        h_b = {
            "sessions": db["sessions"],
            "saves": db["saves"],
            "new_blocks": db["new_blocks"],
            "unverified_after_window": db["unverified_after_window"],
            "false_unverified_rate": rate,
            "by_kind": _by_kind(db),
            "side_notes": db["side_notes"],
            "rename_records_without_blocks": sum(
                1 for r in ren if not r["side"] and r["new_blocks"] == 0
            ),
            "source": "p2_0_1/d_b.json",
            "pass": rate is not None and rate <= 0.01 and db["sessions"] == 200,
        }
        h_b1 = {
            "rename_sessions": len({r["session"] for r in ren}),
            "rename_saves": len(ren),
            "new_blocks": sum(r["new_blocks"] for r in ren),
            "unverified_after_window": sum(r["unverified_after_window"] for r in ren),
            "source": "p2_0_1/d_b.json",
            "pass": None,
        }
        p95 = db["latency_p95_ms"]
        h_c = {
            "latency_p95_ms": p95,
            "latency_measured": db["latency_measured"],
            "latency_unsettled": db["latency_unsettled"],
            "latency_p95_over_window": db["latency_p95_over_window"],
            "no_write": db["no_write"],
            "source": "p2_0_1/d_b.json",
            "pass": p95 is not None and p95 < 500 and db["latency_p95_over_window"] is False,
        }
    h_dp = (
        {**not_measured, "reason": dcp.get("reason"), "source": "p2_0_1/d_c_prime.json"}
        if _skipped(dcp)
        else {
            "ticks": dcp["ticks"],
            "human_blocks": dcp["human_blocks"],
            "human_canonical": dcp["human_canonical"],
            "human_accuracy": dcp["human_accuracy"],
            "agent_blocks": dcp["agent_blocks"],
            "agent_canonical": dcp["agent_canonical"],
            "by_delay": dcp["by_delay"],
            "source": "p2_0_1/d_c_prime.json",
            "pass": dcp["ticks"] == 40
            and dcp["human_accuracy"] is not None
            and dcp["human_accuracy"] >= 0.95
            and dcp["agent_canonical"] == 0,
        }
    )
    h_e = (
        {**not_measured, "reason": dap.get("reason"), "source": "p2_0_1/d_a_prime.json"}
        if _skipped(dap)
        else {
            "scenarios": dap["scenarios"],
            "agent_blocks": dap["agent_blocks"],
            "agent_canonical": dap["agent_canonical"],
            "by_kind": dap["by_kind"],
            "links_rewritten": dap["links_rewritten"],
            "fakes": dap["fakes"],
            "fakes_denied": dap["fakes_denied"],
            "source": "p2_0_1/d_a_prime.json",
            "pass": dap["scenarios"] == 50
            and dap["agent_canonical"] == 0
            and dap["fakes"] == 10
            and dap["fakes_denied"] == 10,
        }
    )
    out = {
        "window": {"window_s": 0.5, "source": "docs/rounds/P2-0/progress/Z5_tuning.json"},
        "H-P2-0.1a": {
            "scenarios": da["scenarios"],
            "new_blocks": da["new_blocks"],
            "canonical_leak": da["canonical_leak"],
            "source": "p2_0_1/d_a.json",
            "pass": da["canonical_leak"] == 0 and da["pass_0a"] is True and da["scenarios"] == 200,
        },
        "H-P2-0.1a′": {
            "scenarios": f"{da['scenarios_0a_prime']}/{da['scenarios']}",
            "canonical_changed": da["canonical_changed"],
            "source": "p2_0_1/d_a.json",
            "pass": da["pass_0a_prime"] is True and da["scenarios_0a_prime"] == 200,
        },
        "H-P2-0.1b": h_b,
        "H-P2-0.1b′": h_b1,
        "H-P2-0.1c": h_c,
        "H-P2-0.1d": {
            "ticks": dc["ticks"],
            "blocks": dc["blocks"],
            "right": dc["right"],
            "accuracy": dc["accuracy"],
            "agent_canonical": dc["agent_canonical"],
            "by_mode": dc["by_mode"],
            "partial_lines": dc["partial_lines"],
            "source": "p2_0_1/d_c.json",
            "pass": dc["pass"] is True,
        },
        "H-P2-0.1d′": h_dp,
        "H-P2-0.1e": h_e,
        "H-P2-0.1f": {
            "sessions": db2["sessions"],
            "new_blocks": db2["new_blocks"],
            "unverified_after_window": db2["unverified_after_window"],
            "false_unverified_rate": db2["false_unverified_rate"],
            "latency_p95_ms": db2["latency_p95_ms"],
            "latency_unsettled": db2["latency_unsettled"],
            "by_kind": db2["by_kind"],
            "source": "p2_0_1/d_b2.json",
            "pass": db2["pass"] is True,
        },
        "H-P2-0.1g": {
            "scenarios": da2["scenarios"],
            "new_blocks": da2["new_blocks"],
            "canonical_leak": da2["canonical_leak"],
            "by_path": da2["by_path"],
            "source": "p2_0_1/d_a2.json",
            "pass": da2["pass"] is True,
        },
        "H-P2-0.1h": {
            "vaults": f"{de['passed']}/{len(de['vaults'])}",
            "failed": [v["vault"] for v in de["vaults"] if not v["pass"]],
            "source": "p2_0_1/d_e.json",
            "pass": de["pass"] is True,
        },
        "H-P2-0.1j": {
            "level_claimed": conf["level_claimed"],
            "spec_version": conf["spec_version"],
            "passed": conf["passed"],
            "total": conf["total"],
            "source": "conformance_p1d1.json",
            "pass": conf["level_claimed"] == 3
            and conf["spec_version"] == "2.0.1"
            and conf["passed"] == 38
            and conf["total"] == 38,
        },
        "H-P2-0.1k": {
            "counted_fails": hist["counted_fails"],
            "historical": hist["historical"],
            "counted": hist["counted"],
            "reference_run": hist["reference_run"],
            "source": "p2_0_1/historical.json (eval.historical_p2_0_1 = the chain's summary)",
            "pass": hist["counted_fails"] == 0,
        },
        "regress": {
            "reference_run": reg["reference_run"],
            "n_fields": reg["n_fields"],
            "differing_fields": reg["differing_fields"],
            "excluded_differences": reg["excluded_differences"],
            "exclusions_without_difference": reg["exclusions_without_difference"],
            "conformance": reg["conformance"],
            "p2_0_sets": reg["p2_0_sets"],
            "source": "regress_vs_p2_0.json",
            "pass": reg["equal"] is True,
        },
        "tamper": {
            "rejected": f"{tamper['rejected']}/{tamper['rejected_total']}",
            "controls": f"{tamper['controls_ok']}/{tamper['controls_total']}",
            "source": "p2_0_1/tamper_p2_0_1.json",
            "pass": tamper["pass"] is True,
        },
    }
    runinfo.write_json(a.run, "p2_0_1_checks.json", out)
    runinfo.record_phase(a.run, "p2_0_1_checks", {}, ["p2_0_1_checks.json"])
    print(
        "; ".join(
            f"{k} {'-' if not isinstance(v, dict) or v.get('pass') is None else 'pass' if v['pass'] else 'FAIL'}"
            for k, v in out.items()
            if k != "window"
        )
    )


def _by_kind(db: dict) -> dict:
    """The D-B test-2 records by session kind: new blocks, still `unverified`."""
    out: dict[str, dict[str, int]] = {}
    for r in db["records"]:
        k = out.setdefault(r["kind"], {"saves": 0, "new_blocks": 0, "unverified": 0})
        k["saves"] += 1
        k["new_blocks"] += r["new_blocks"]
        k["unverified"] += r["unverified_after_window"]
    return out


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
