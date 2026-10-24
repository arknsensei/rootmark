"""The regression step of the fast verification (`make verify FAST=1`;
P2-L/L0 d, K-P2.4 (3)): this RUN against the last official RUN and the
English quality reference.

- `quality`: `c3_quality.json` of this RUN (D3 + the D4 test split, the
  default lexical variant) against `data/quality_ref_rust.json` — the 618
  values of K-C1.2, every one equal (`run_p1c1_checks.quality_vs_ref`, the
  frozen C1-9 comparison).
- `conformance`: `fast/conformance.json` (`make conformance`) against the
  official RUN's full report (`conformance_p1d1.json`) by test id: every
  test of the reference has the same result and hashes; a test the
  reference does not hold must pass (listed as new).
- `conformance_equiv`: `make conformance` and `rootmark conformance` give the
  same result test by test (`scripts/conformance_equiv.sh`, its exit code
  recorded by the chain in `fast/conformance_equiv.json`).

Writes `fast/regress.json`; exit 0 when everything passes, 3 when something
differs (the record is still written), 1 on a missing file.

    python -m eval.fast_regress --run <RUN> --ref <official RUN dir>
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import runinfo
from eval.datasets.common import DATA, dump_json
from eval.run_p1c1_checks import quality_vs_ref

REF = DATA / "quality_ref_rust.json"
TEST_FIELDS = ("result", "level", "expected_hash", "actual_hash")


def _load(p: Path) -> dict:
    if not p.is_file():
        raise SystemExit(f"fast_regress: {p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def conformance_vs(mine: dict, ref: dict) -> dict:
    """By test id: the reference's tests unchanged, new tests passing."""
    got = {t["id"]: t for t in mine["tests"]}
    want = {t["id"]: t for t in ref["tests"]}
    differing = [
        i
        for i, t in sorted(want.items())
        if i not in got or any(got[i].get(f) != t.get(f) for f in TEST_FIELDS)
    ]
    new = sorted(set(got) - set(want))
    new_failing = [i for i in new if got[i]["result"] != "pass"]
    return {
        "reference_tests": len(want),
        "tests": len(got),
        "differing": differing,
        "new": new,
        "new_failing": new_failing,
        "level_claimed": mine["level_claimed"],
        "pass": not differing and not new_failing and mine["level_claimed"] == 3,
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--ref", type=Path, required=True)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    quality = quality_vs_ref(_load(d / "c3_quality.json"), _load(REF))
    conf = conformance_vs(
        _load(d / "fast" / "conformance.json"), _load(a.ref / "conformance_p1d1.json")
    )
    equiv = _load(d / "fast" / "conformance_equiv.json")
    out = {
        "reference_run": a.ref.name,
        "quality": quality,
        "conformance": conf,
        "conformance_equiv": equiv,
        "pass": quality["pass"] and conf["pass"] and equiv.get("exit") == 0,
    }
    (d / "fast" / "regress.json").write_text(dump_json(out))
    print(
        f"fast_regress: quality {quality['compared']} values, differing {len(quality['differing'])}; "
        f"conformance {conf['tests']} tests ({len(conf['new'])} new), differing {len(conf['differing'])}; "
        f"equiv exit {equiv.get('exit')} → {'pass' if out['pass'] else 'FAIL'}"
    )
    return 0 if out["pass"] else 3


if __name__ == "__main__":
    sys.exit(main())
