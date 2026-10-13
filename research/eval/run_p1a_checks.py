"""P1-A checks recorded in the RUN (GOAL A-1, A-5, A-6), so that the round
report is built from one RUN directory only.

- A-1: `reports/conformance.json` written by `make conformance` (17/17, level 1).
- A-5: mini_vault `events_hash` with matcher v2 and v3 through the bindings
  (expected values coded from docs/rounds/P1-A/GOAL.md).
- A-6: the dependency-direction log written by `scripts/check_deps.sh` and
  `scripts/test_check_deps.sh` (both must report OK).

    python -m eval.run_p1a_checks --run <RUN> --conformance <path> --deps-log <path>
"""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path

import mdkernel_core

from eval import runinfo
from eval.datasets.common import ROOT, die

MINI = ROOT.parent / "spec" / "fixtures" / "mini_vault"
# GOAL A-5 (pre-registered)
A5_EXPECTED = {"v2": "8914f9b418ae360d49c85c951050f432", "v3": "808a41f21a43e121d7d688fee9f5bc24"}
LEVEL1_TOTAL = 17


def mini_hashes() -> dict:
    out = {}
    for m in ("v2", "v3"):
        with tempfile.TemporaryDirectory() as t:
            v = mdkernel_core.Vault(t, matcher=m)
            v.ingest_dir(str(MINI))
            out[m] = v.events_hash()
    return out


def conformance(path: Path) -> dict:
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing; run `make conformance`")
    c = json.loads(path.read_text(encoding="utf-8"))
    if c.get("total") != LEVEL1_TOTAL or len(c.get("tests", [])) != LEVEL1_TOTAL:
        raise RuntimeError(f"{path}: expected {LEVEL1_TOTAL} level-1 tests")
    return c


def deps(path: Path) -> dict:
    if not path.is_file():
        raise FileNotFoundError(f"{path} missing")
    lines = path.read_text(encoding="utf-8").splitlines()
    out = {
        "check_deps_ok": any(ln.startswith("check_deps: OK") for ln in lines),
        "deliberate_reverse_dependency_test_ok": "test_check_deps: OK" in lines,
        "violations": sum(1 for ln in lines if "VIOLATION" in ln),
        "log": lines,
    }
    return out


def run(run_id: str, conf_path: Path, deps_path: Path) -> dict:
    c = conformance(conf_path)
    h = mini_hashes()
    out = {
        "A-1": {
            "passed": c["passed"],
            "total": c["total"],
            "level_claimed": c["level_claimed"],
            "fixtures_manifest": c["fixtures_manifest"],
            "tests": [
                {k: t[k] for k in ("id", "result", "actual_hash", "expected_hash")}
                for t in c["tests"]
            ],
        },
        "A-5": {
            "events_hash": h,
            "expected": A5_EXPECTED,
            "equal": {m: h[m] == A5_EXPECTED[m] for m in h},
        },
        "A-6": deps(deps_path),
    }
    runinfo.write_json(run_id, "p1a_checks.json", out)
    runinfo.record_phase(run_id, "p1a_checks", {}, ["p1a_checks.json"])
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--conformance", type=Path, required=True)
    ap.add_argument("--deps-log", type=Path, required=True)
    a = ap.parse_args(argv)
    out = run(a.run, a.conformance, a.deps_log)
    print(json.dumps({k: out[k] for k in ("A-5",)}, indent=1))
    print(f"A-1 {out['A-1']['passed']}/{out['A-1']['total']}; A-6 ok={out['A-6']['check_deps_ok']}")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
