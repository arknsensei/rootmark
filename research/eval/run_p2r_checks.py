"""P2-R round checks recorded in the RUN (`make p2r-checks`; the active
round's step of `make verify FAST=1`): `eval.run_p2l_checks`'s runner as it
is — each criterion's tests run and recorded by name, pass or fail, no
timing, no path — written to `p2r_checks.json` (P2-B's `p2b_checks.json`,
which the full chain still writes, is not touched).

    python -m eval.run_p2r_checks --run <RUN> --tests <crit>=<pkg>:<target>[,…] …

Exit 0 when every test passes, 3 otherwise (the record is still written).
"""

from __future__ import annotations

import argparse
import sys

from eval import runinfo
from eval.datasets.common import dump_json
from eval.run_p2l_checks import cargo_tests, criterion


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--tests", action="append", required=True, type=criterion)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    out: dict = {}
    for crit, targets in a.tests:
        if crit in out:
            raise SystemExit(f"run_p2r_checks: criterion {crit} given twice")
        runs = [cargo_tests(p, t) for p, t in targets]
        out[crit] = {"targets": runs, "pass": all(r["pass"] for r in runs)}
    out["pass"] = all(v["pass"] for v in out.values() if isinstance(v, dict))
    (d / "p2r_checks.json").write_text(dump_json(out))
    for crit, v in out.items():
        if isinstance(v, dict):
            n = sum(len(r["tests"]) for r in v["targets"])
            print(f"p2r-checks: {crit} {n} tests → {'pass' if v['pass'] else 'FAIL'}")
    return 0 if out["pass"] else 3


if __name__ == "__main__":
    sys.exit(main())
