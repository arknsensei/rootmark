"""P1-C.1 criteria recorded in the RUN (GOAL C1-1..C1-9), so that the round
report is built from one RUN directory only: `p1c1_checks.json`.

Read from files `make p1c1-checks` (and `make p1c-checks` before it) wrote
into the RUN:

- C1-1: `c1_variant.json` (`eval.run_c1_variant`)
- C1-2, C1-3: `p1c1/c1_2_3.json` (`index/tests/p1c1.rs`, 1000 queries × 3
  modes on a vault with proposals)
- C1-4: `p1c1/c1_4.json` (library) and `p1c1_cli.json`
  (`scripts/test_p1c1.sh`: `diff -r` of the index after CLI searches, lock
  control)
- C1-5: `p1c1/c1_5.json` and `p1c1_cli.json`
- C1-6: `p1c1/c1_6.json` and `p1c1_cli.json`
- C1-7: `conformance_p1c1.json` (the full Level 1–3 report); the spec
  version is the one `docs/SPEC_EDITS_CRITERION.md`'s rule gives (1.0.2;
  GOAL C1-7 names 1.1.0 — docs/rounds/P1-C.1/progress/SPEC_EDITS.md,
  ASSUMPTIONS C1.0-1)
- C1-8: `regress_vs_p1c.json` (`eval.compare_p1c1 regress`, K-C1.6
  exclusions); the A ↔ B half is `pair_compare.json` (`verify.sh pair`),
  read by the report
- C1-9: `c3_quality.json` of this RUN against `data/quality_ref_rust.json`,
  every cell and per-query metric, exact

    python -m eval.run_p1c1_checks --run <RUN>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import runinfo
from eval.datasets.common import DATA, die
from eval.gen_quality_ref import METRICS, reference

SPEC_VERSION = "1.0.2"
REF = DATA / "quality_ref_rust.json"


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing (make p1c1-checks)")
    return json.loads(p.read_text(encoding="utf-8"))


def quality_vs_ref(c3: dict, ref: dict) -> dict:
    mine = reference(c3)
    diffs = []
    for key, cell in ref["cells"].items():
        for m in ("n", *METRICS):
            if mine["cells"].get(key, {}).get(m) != cell[m]:
                diffs.append(f"cells.{key}.{m}")
    if sorted(mine["cells"]) != sorted(ref["cells"]):
        diffs.append("cells: keys")
    for mode, rows in ref["per_query"].items():
        got = {r["id"]: r for r in mine["per_query"].get(mode, [])}
        if len(got) != len(rows):
            diffs.append(f"per_query.{mode}: {len(got)} rows, reference {len(rows)}")
        for r in rows:
            if got.get(r["id"]) != r:
                diffs.append(f"per_query.{mode}.{r['id']}")
    n = sum(len(c) for c in ref["cells"].values()) + sum(
        len(rows) for rows in ref["per_query"].values()
    )
    return {
        "reference": REF.name,
        "source_run": ref["source_run"],
        "compared": n,
        "differing": diffs,
        "pass": not diffs,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    c11 = _load(d, "c1_variant.json")
    c123 = _load(d, "p1c1/c1_2_3.json")
    c14 = _load(d, "p1c1/c1_4.json")
    c15 = _load(d, "p1c1/c1_5.json")
    c16 = _load(d, "p1c1/c1_6.json")
    cli = _load(d, "p1c1_cli.json")
    conf = _load(d, "conformance_p1c1.json")
    reg = _load(d, "regress_vs_p1c.json")
    c3 = _load(d, "c3_quality.json")
    if not REF.is_file():
        raise FileNotFoundError(f"{REF} missing")
    ref = json.loads(REF.read_text(encoding="utf-8"))
    by = {lvl: [t for t in conf["tests"] if t["level"] == lvl] for lvl in (1, 2, 3)}
    passed = {lvl: sum(t["result"] == "pass" for t in by[lvl]) for lvl in by}
    out = {
        "C1-1": {
            "cells": {k: {"delta": v["delta"], "pass": v["pass"]} for k, v in c11["C1-1"].items()},
            "threshold": c11["threshold_abs_delta"],
            "per_query_equal_to_rust": c11["per_query_equal_to_rust"],
            "okapi_delta_vs_v2": c11["okapi_delta_vs_v2"],
            "pass": c11["pass"],
        },
        "C1-2": {
            "searches": c123["searches"],
            **c123["C1-2"],
        },
        "C1-3": {
            "searches": c123["searches"],
            "chunks_by_layer": c123["chunks_by_layer"],
            "lexical_queries_sensitive_to_mixed_statistics": c123[
                "lexical_queries_sensitive_to_mixed_statistics"
            ],
            **c123["C1-3"],
        },
        "C1-4": {
            "library": c14,
            "cli": cli,
            "pass": c14["pass"] and cli["pass"],
        },
        "C1-5": {
            "library": c15,
            "cli": cli,
            "pass": c15["pass"] and cli["pass"],
        },
        "C1-6": {
            "lexical": c16["lexical"],
            "rejected_cases": len(c16["rejected"]),
            "cli": cli,
            "pass": c16["pass"] and cli["pass"],
        },
        "C1-7": {
            "C1": f"{passed[1]}/{len(by[1])}",
            "C2": f"{passed[2]}/{len(by[2])}",
            "C3": f"{passed[3]}/{len(by[3])}",
            "spec_version": conf["spec_version"],
            "spec_version_expected": SPEC_VERSION,
            "level_claimed": conf["level_claimed"],
            "fixtures_manifest": conf["fixtures_manifest"],
            "pass": passed[1] == len(by[1]) == 17
            and passed[2] == len(by[2]) == 9
            and passed[3] == len(by[3]) == 5
            and conf["spec_version"] == SPEC_VERSION
            and conf["level_claimed"] == 3
            and conf["fixtures_manifest"] == "ok",
        },
        "C1-8": {
            "regress": {
                "reference_run": reg["reference_run"],
                "n_fields": reg["n_fields"],
                "differing_fields": reg["differing_fields"],
                "differing_excluded": sorted(reg["differing_excluded"]),
                "equal": reg["equal"],
            },
            "pair": {"decided_by": "eval.compare_p1c1 pair (pair_compare.json)"},
            "pass": reg["equal"],
        },
        "C1-9": quality_vs_ref(c3, ref),
    }
    runinfo.write_json(a.run, "p1c1_checks.json", out)
    runinfo.record_phase(a.run, "p1c1_checks", {}, ["p1c1_checks.json"])
    print("; ".join(f"{c} {'pass' if out[c]['pass'] else 'FAIL'}" for c in out if "pass" in out[c]))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
