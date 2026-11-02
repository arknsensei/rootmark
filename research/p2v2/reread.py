"""P2-V.2/Y2–Y3 (TASKS Y2 önce/sonra, Y3 yeniden okuma): the tier-2 readings of
a downloaded run (`reports/rounds/P2-V.2/ci/<run>/tier2-{A,B}/`), with the
commands of `.github/workflows/verify.yml`'s compare job, one inventory.

    python -m p2v2.reread <ci/<run>> --out <dir> [--inventory <inventory.json>]
        [--equiv-old <V0m ci dir> --mac <mac_checks.json>]

Writes, in `<dir>`: `frozen_pair_compare.json` (the frozen `compare_p2r
pair` of the two downloaded RUNs, run here: its `run_a`/`run_b` are this
machine's paths, the CI's are the runner's), `tier2_pair.json`,
`tier2_fails_{A,B}.json`, `portable_{A,B}.json`, `criteria_{A,B}.json`
(as `tier2_criteria_{A,B}.json` too, TASKS Y3) and, with `--equiv-old`,
`equiv_v0m.json` (old chain A ↔ this run's A). Each tool's exit code is in
`exits.json` (0 pass, 3 recorded failure; anything else is an error and
stops: never a silent skip). No timestamps are written.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

RESEARCH = Path(__file__).resolve().parents[1]


def _rel(p: Path) -> Path:
    return Path(os.path.relpath(p.resolve(), RESEARCH))


def _run(args: list[str], exits: dict, name: str) -> int:
    p = subprocess.run(
        [sys.executable, "-m", *args], check=False, cwd=RESEARCH, capture_output=True, text=True
    )
    exits[name] = p.returncode
    if p.returncode not in (0, 3):
        sys.stderr.write(p.stderr[-4000:])
        raise SystemExit(f"error: {name} exit {p.returncode}")
    return p.returncode


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("ci", type=Path)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--inventory", type=Path)
    ap.add_argument("--equiv-old", type=Path)
    ap.add_argument("--mac", type=Path)
    a = ap.parse_args(argv)
    # paths relative to research/ (the tools' working directory): no host
    # value in what is written (`run_a`, `frozen_result_read`)
    ci, o = _rel(a.ci), _rel(a.out)
    (RESEARCH / o).mkdir(parents=True, exist_ok=True)
    inv = ["--inventory", str(_rel(a.inventory))] if a.inventory else []
    ra, rb = ci / "tier2-A/run", ci / "tier2-B/run"
    for r in (ra, rb):
        if not (RESEARCH / r / "manifest.json").is_file():
            raise SystemExit(f"error: {r} is not a RUN")
    exits: dict[str, int] = {}
    frozen = o / "frozen_pair_compare.json"
    _run(
        ["eval.compare_p2r", "pair", str(ra), str(rb), "--out", str(frozen)],
        exits,
        "compare_p2r_pair",
    )
    _run(
        [
            "eval.tier2",
            "pair",
            str(ra),
            str(rb),
            "--frozen",
            str(frozen),
            "--out",
            str(o / "tier2_pair.json"),
            *inv,
        ],
        exits,
        "tier2_pair",
    )
    for s, r in (("A", ra), ("B", rb)):
        side = ci / f"tier2-{s}"
        cls = (RESEARCH / side / "classifier").read_text(encoding="utf-8").strip()
        _run(
            [
                "eval.tier2",
                "fails",
                str(r),
                "--summary",
                str(side / "fail_summary.json"),
                "--classifier",
                cls,
                "--out",
                str(o / f"tier2_fails_{s}.json"),
                *inv,
            ],
            exits,
            f"tier2_fails_{s}",
        )
        _run(
            [
                "eval.portable",
                str(r),
                "--host",
                str(side / "host.json"),
                "--files-from",
                str(frozen),
                "--all",
                "--out",
                str(o / f"portable_{s}.json"),
                *inv,
            ]
            if a.inventory and _portable_takes_inventory()
            else [
                "eval.portable",
                str(r),
                "--host",
                str(side / "host.json"),
                "--files-from",
                str(frozen),
                "--all",
                "--out",
                str(o / f"portable_{s}.json"),
            ],
            exits,
            f"portable_{s}",
        )
        _run(
            ["eval.tier2", "criteria", str(r), "--out", str(o / f"criteria_{s}.json"), *inv],
            exits,
            f"criteria_{s}",
        )
        shutil.copyfile(
            RESEARCH / o / f"criteria_{s}.json", RESEARCH / o / f"tier2_criteria_{s}.json"
        )
    if a.equiv_old:
        old = _rel(a.equiv_old) / "tier2-A/run"
        mac = ["--mac", str(_rel(a.mac))] if a.mac else []
        _run(
            [
                "eval.tier2",
                "equiv",
                str(old),
                str(ra),
                *mac,
                "--out",
                str(o / "equiv_v0m.json"),
                *inv,
            ],
            exits,
            "equiv_v0m",
        )
        condense(RESEARCH / o / "equiv_v0m.json")
    (RESEARCH / o / "exits.json").write_text(json.dumps(exits, indent=1) + "\n", encoding="utf-8")
    print(json.dumps(exits))


NOTE = "named_differences condensed to counts (full list 49.9 MB; regenerate: python -m eval.tier2 equiv <V0m 40613392932 A run> <41239746833 chain A run> --mac reports/rounds/P2-V.1/mac/20261031T204025Z-58377de/mac_checks.json)"


def condense(p: Path) -> None:
    """P2-V.1's committed form of an `equiv` result
    (`reports/rounds/P2-V.1/ci/41239746833/equiv_v0m.json`): the named
    differences as counts per group, the H and commit groups listed (the
    E1 and Z/O lists are ~50 MB); the full result is kept beside it,
    uncommitted (`equiv_v0m.full.json`)."""
    from eval.datasets.common import dump_json

    res = json.loads(p.read_text(encoding="utf-8"))
    shutil.copyfile(p, p.with_name("equiv_v0m.full.json"))
    named = res.pop("named_differences")
    res["named_counts"] = {g: len(v) for g, v in named.items()}
    res["named_H"] = named["H"]
    res["named_commit"] = named["commit"]
    res["note"] = NOTE
    # as committed in P2-V.1: `dump_json` without its final newline
    p.write_text(dump_json(res)[:-1], encoding="utf-8")


def _portable_takes_inventory() -> bool:
    """Whether this tree's `eval.portable` reads an inventory (P2-V.2/F2);
    ROUND_START's does not (its `--help` names no `--inventory`)."""
    p = subprocess.run(
        [sys.executable, "-m", "eval.portable", "--help"],
        check=False,
        cwd=RESEARCH,
        capture_output=True,
        text=True,
    )
    return "--inventory" in p.stdout


if __name__ == "__main__":
    main()
