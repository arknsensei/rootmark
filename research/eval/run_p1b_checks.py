"""P1-B checks recorded in the RUN (GOAL B-1, B-2, B-3, B-5, B-6, B-10 and
the governance scenario of B-4), so that the round report is built from one
RUN directory only. Inputs are the files `make p1b-checks` and `make regress`
wrote into the RUN; nothing here holds an absolute path (the pair comparison
reads this file field by field).

    python -m eval.run_p1b_checks --run <RUN> --seeds <seed> <seed>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval import runinfo
from eval.datasets.common import die

LEVEL1_TOTAL = 17
LEVEL2_TOTAL = 8


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing (make p1b-checks / make regress)")
    return json.loads(p.read_text(encoding="utf-8"))


def conformance(d: Path) -> dict:
    c = _load(d, "conformance.json")
    by = {1: [], 2: []}
    for t in c["tests"]:
        by[t["level"]].append(t)
    if len(by[1]) != LEVEL1_TOTAL or len(by[2]) != LEVEL2_TOTAL:
        raise RuntimeError("conformance.json: expected 17 level-1 and 8 level-2 tests")
    keep = ("id", "result", "actual_hash", "expected_hash")
    return {
        "level_claimed": c["level_claimed"],
        "fixtures_manifest": c["fixtures_manifest"],
        "fixtures_manifest_hash": c["fixtures_manifest_hash"],
        "fixtures_root_absolute": str(c["fixtures_root"]).startswith("/"),
        "fixtures_root_source": c["fixtures_root_source"],
        "B-1": {
            "passed": sum(t["result"] == "pass" for t in by[2]),
            "total": LEVEL2_TOTAL,
            "tests": [{k: t[k] for k in keep} for t in by[2]],
        },
        "B-2": {
            "passed": sum(t["result"] == "pass" for t in by[1]),
            "total": LEVEL1_TOTAL,
            "tests": [{k: t[k] for k in keep} for t in by[1]],
        },
    }


def regress(d: Path) -> dict:
    v3 = _load(d, "regress_vs_v3final.json")
    p1a = _load(d, "regress_vs_p1a.json")
    return {
        "v3_final": {
            "n_fields": v3["n_fields"],
            "max_relative_difference": v3["max_relative_difference"],
            "files": v3["files"],
        },
        "p1a_official": {
            "n_fields": p1a["n_fields"],
            "max_relative_difference": p1a["max_relative_difference"],
            "files": p1a["files"],
        },
        "equal": v3["max_relative_difference"] == 0 and p1a["max_relative_difference"] == 0,
    }


def deps(d: Path) -> dict:
    lines = (d / "check_deps.log").read_text(encoding="utf-8").splitlines()
    return {
        "check_deps_ok": any(ln.startswith("check_deps: OK") for ln in lines),
        "deliberate_reverse_dependency_test_ok": "test_check_deps: OK" in lines,
        "violations": sum(1 for ln in lines if "VIOLATION" in ln),
    }


def run(run_id: str, seeds: list[int]) -> dict:
    d = runinfo.run_dir(run_id, create=False)
    gov = _load(d / "governance", "governance.json")
    fuzz = {str(s): _load(d, f"fuzz_{s}.json") for s in seeds}
    out = {
        **conformance(d),
        "B-3": regress(d),
        "B-5": _load(d, "v3_compat.json"),
        "B-6": _load(d, "policy_bounds.json"),
        "B-10": {
            "seeds": fuzz,
            "replay_equal": all(f["replay_equal"] for f in fuzz.values()),
        },
        "governance": gov,
        "deps": deps(d),
    }
    runinfo.write_json(run_id, "p1b_checks.json", out)
    runinfo.record_phase(run_id, "p1b_checks", {"seeds": seeds}, ["p1b_checks.json"])
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--seeds", type=int, nargs="+", required=True)
    a = ap.parse_args(argv)
    out = run(a.run, a.seeds)
    print(
        json.dumps(
            {
                "level_claimed": out["level_claimed"],
                "B-1": f"{out['B-1']['passed']}/{out['B-1']['total']}",
                "B-2": f"{out['B-2']['passed']}/{out['B-2']['total']}",
                "B-3_equal": out["B-3"]["equal"],
                "B-5": out["B-5"]["pass"],
                "B-6_cases": len(out["B-6"]["cases"]),
                "B-10": out["B-10"]["replay_equal"],
            }
        )
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
