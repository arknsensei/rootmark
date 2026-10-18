"""P2-0 hypotheses recorded in the RUN (GOAL P2-0 "Önceden kayıtlı
eşikler"), so that the round report is built from one RUN directory only:
`p2_0_checks.json`.

Read from what `make p2-0-checks` wrote into the RUN:

- the chosen window: `p2_0/tuning.json` (the D-B tuning run, committed
  as `docs/rounds/P2-0/progress/Z5_tuning.json` and copied into the RUN —
  the test sets never choose `W`);
- H-P2-0a, a′: `p2_0/d_a.json`; H-P2-0b, b′, c: `p2_0/d_b.json` (a record
  that says `skipped` — no Obsidian binary — leaves them not measured);
  H-P2-0d: `p2_0/d_c.json`; H-P2-0f: `p2_0/d_e.json`;
- H-P2-0e: not measured in the chain (Z6: the probe reads a person's
  keychain item; `scripts/keychain_probe.sh` is run by a person);
- H-P2-0h: the conformance report `conformance_p1d1.json` (37/37, level 3,
  spec 2.0.0);
- regress: `regress_vs_p1e1.json` (`eval.compare_p2_0 regress` against the
  P1-E.1 official RUN);
- tamper: `p2_0/tamper_p2_0.json` (`eval.tamper_p2_0`).
- H-P2-0g is the A ↔ B half (`pair_compare.json`, `eval.compare_p2_0
  pair`: the D-A and D-C `events.jsonl` and `attestations.jsonl` byte for
  byte), read by the report, not here.

    python -m eval.run_p2_0_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import runinfo
from eval.datasets.common import die


def _load(path: Path):
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing (make p2-0-checks)")
    return json.loads(path.read_text(encoding="utf-8"))


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    p = d / "p2_0"
    tuning = _load(p / "tuning.json")
    da = _load(p / "d_a.json")
    db = _load(p / "d_b.json")
    dc = _load(p / "d_c.json")
    de = _load(p / "d_e.json")
    tamper = _load(p / "tamper_p2_0.json")
    conf = _load(d / "conformance_p1d1.json")
    reg = _load(d / "regress_vs_p1e1.json")
    skipped = db.get("skipped") is True
    rate = None if skipped else db["false_unverified_rate"]
    p95 = None if skipped else db["latency_p95_ms"]
    out = {
        "window": {
            "window_s": tuning["window_s"],
            "chosen_by": tuning["chosen_by"],
            "tune_set_blake3": tuning.get("tune_set_blake3"),
        },
        "H-P2-0a": {
            "scenarios": da["scenarios"],
            "new_blocks": da["new_blocks"],
            "canonical_leak": da["canonical_leak"],
            "pass": da["canonical_leak"] == 0 and da["pass_0a"] is True and da["scenarios"] == 200,
        },
        "H-P2-0a′": {
            "scenarios": f"{da['scenarios_0a_prime']}/{da['scenarios']}",
            "canonical_changed": da["canonical_changed"],
            "pass": da["pass_0a_prime"] is True and da["scenarios_0a_prime"] == 200,
        },
        "H-P2-0b": (
            {"skipped": True, "reason": db.get("reason"), "measured": False, "pass": None}
            if skipped
            else {
                "sessions": db["sessions"],
                "saves": db["saves"],
                "new_blocks": db["new_blocks"],
                "unverified_after_window": db["unverified_after_window"],
                "false_unverified_rate": rate,
                "by_kind": _by_kind(db),
                "side_notes": db["side_notes"],
                # Z5 review N-R1: a renamed note the server never answered
                # for would show as zero blocks; expected 0
                "rename_records_without_blocks": sum(
                    1
                    for r in db["records"]
                    if r["kind"] == "rename" and not r["side"] and r["new_blocks"] == 0
                ),
                "pass": rate is not None and rate <= 0.01 and db["sessions"] == 200,
            }
        ),
        "H-P2-0b′": (
            {"skipped": True, "measured": False, "pass": None}
            if skipped
            else {"raised_late": db["raised_late"], "saves": db["saves"], "pass": None}
        ),
        "H-P2-0c": (
            {"skipped": True, "measured": False, "pass": None}
            if skipped
            else {
                "latency_p95_ms": p95,
                "latency_measured": db["latency_measured"],
                "latency_unsettled": db["latency_unsettled"],
                "latency_p95_over_window": db["latency_p95_over_window"],
                "no_write": db["no_write"],
                "pass": p95 is not None and p95 < 500 and db["latency_p95_over_window"] is False,
            }
        ),
        "H-P2-0d": {
            "ticks": dc["ticks"],
            "blocks": dc["blocks"],
            "right": dc["right"],
            "accuracy": dc["accuracy"],
            "by_mode": dc["by_mode"],
            "pass": dc["pass"] is True,
        },
        "H-P2-0e": {
            "measured": False,
            "reason": "the probe reads a person's keychain item (Z6); scripts/keychain_probe.sh is run by a person",
            "pass": None,
        },
        "H-P2-0f": {
            "vaults": f"{de['passed']}/{len(de['vaults'])}",
            "failed": [v["vault"] for v in de["vaults"] if not v["pass"]],
            "pass": de["pass"] is True,
        },
        "H-P2-0h": {
            "level_claimed": conf["level_claimed"],
            "spec_version": conf["spec_version"],
            "passed": conf["passed"],
            "total": conf["total"],
            "pass": conf["level_claimed"] == 3
            and conf["spec_version"] == "2.0.0"
            and conf["passed"] == 37
            and conf["total"] == 37,
        },
        "regress": {
            "reference_run": reg["reference_run"],
            "n_fields": reg["n_fields"],
            "differing_fields": reg["differing_fields"],
            "excluded_differences": reg["excluded_differences"],
            "exclusions_without_difference": reg["exclusions_without_difference"],
            "conformance": reg["conformance"],
            "pass": reg["equal"] is True,
        },
        "tamper": {
            "rejected": f"{tamper['rejected']}/{tamper['rejected_total']}",
            "controls": f"{tamper['controls_ok']}/{tamper['controls_total']}",
            "pass": tamper["pass"] is True,
        },
    }
    runinfo.write_json(a.run, "p2_0_checks.json", out)
    runinfo.record_phase(a.run, "p2_0_checks", {}, ["p2_0_checks.json"])
    print(
        "; ".join(
            f"{k} {'-' if not isinstance(v, dict) or v.get('pass') is None else 'pass' if v['pass'] else 'FAIL'}"
            for k, v in out.items()
            if k != "window"
        )
    )


def _by_kind(db: dict) -> dict:
    """The D-B test records by session kind: new blocks, still `unverified`."""
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
