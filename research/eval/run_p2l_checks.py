"""P2-L round checks recorded in the RUN (`make p2l-checks`; the active
round's step of `make verify FAST=1`, P2-L/L0 d): each criterion's tests
run and recorded by name, pass or fail — no timing, no path.

The criteria and their test targets are arguments (`make p2l-checks`
passes the round's list, `P2L_TESTS` in the Makefile), so a later task adds
its tests without changing this file: `--tests L0=rootmark-server:p2_l_l0,
rootmark-server:p2a_migrate` (one `--tests` per criterion).

Writes `p2l_checks.json`; exit 0 when every test passes, 3 otherwise (the
record is still written).

    python -m eval.run_p2l_checks --run <RUN> --tests <crit>=<pkg>:<target>[,…] …
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys

from eval import runinfo
from eval.datasets.common import ROOT, dump_json

REPO = ROOT.parent
LINE = re.compile(r"^test (\S+) \.\.\. (ok|FAILED|ignored)$")


def criterion(spec: str) -> tuple[str, list[tuple[str, str]]]:
    """`<crit>=<pkg>:<target>[,<pkg>:<target>…]`."""
    crit, sep, rest = spec.partition("=")
    targets = [tuple(t.split(":", 1)) for t in rest.split(",") if t]
    if not sep or not crit or not targets or any(len(t) != 2 or not all(t) for t in targets):
        raise argparse.ArgumentTypeError(f"--tests {spec!r}: <crit>=<pkg>:<target>[,…]")
    return crit, targets


def cargo_tests(package: str, target: str) -> dict:
    """Runs one test target; every test's name and result."""
    p = subprocess.run(
        [
            "cargo",
            "test",
            "--release",
            "-p",
            package,
            "--test",
            target,
            "--",
            "--test-threads=1",
        ],
        check=False,
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    tests = {}
    for line in p.stdout.splitlines():
        m = LINE.match(line.strip())
        if m:
            tests[m.group(1)] = m.group(2)
    if not tests:
        # nothing parsed is never a pass (a build error, a filter)
        raise SystemExit(
            f"run_p2l_checks: no test ran in {package} --test {target}:\n{p.stderr[-2000:]}"
        )
    return {
        "package": package,
        "target": target,
        "exit": p.returncode,
        "tests": dict(sorted(tests.items())),
        "pass": p.returncode == 0 and all(v == "ok" for v in tests.values()),
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--tests", action="append", required=True, type=criterion)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    out: dict = {}
    for crit, targets in a.tests:
        if crit in out:
            raise SystemExit(f"run_p2l_checks: criterion {crit} given twice")
        runs = [cargo_tests(p, t) for p, t in targets]
        out[crit] = {"targets": runs, "pass": all(r["pass"] for r in runs)}
    out["pass"] = all(v["pass"] for v in out.values() if isinstance(v, dict))
    (d / "p2l_checks.json").write_text(dump_json(out))
    for crit, v in out.items():
        if isinstance(v, dict):
            n = sum(len(r["tests"]) for r in v["targets"])
            print(f"p2l-checks: {crit} {n} tests → {'pass' if v['pass'] else 'FAIL'}")
    return 0 if out["pass"] else 3


if __name__ == "__main__":
    sys.exit(main())
