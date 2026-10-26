"""P2-R run comparisons (TASKS R5; GOAL P2-R R-6, R-10): the P2-B
comparators as they are (`compare_p2b`) and the P2-R result files.

`fast <run_A> <run_B>` (`scripts/verify.sh` with `FAST=1`): the fast
chain's files (`P2R_FAST_FILES`: conformance, regress and C-3, P2-B's and
the P2-R files — the active block's checks), every field but
the absolute fixture path of the chain's clone (`P2R_INFO`, ortam).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): `compare_p2b.pair` (the
P2-A pair set, the P2-L, P2-P and P2-B files) and `P2R_FILES`, every field.

The P2-R files (`make p2r-checks`, and `make p2b-checks`'s alignment check):
`p2r_checks.json` (the round's tests by criterion), `p2r/volume.json`
(D-VOL, R-7), `p2r/det.json` (a prune's outputs and files whole: R-6),
`p2b/align_regress.json` (R0 c5: the alignment against R-3's value).

    python -m eval.compare_p2r fast <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2r pair <run_A> <run_B> [--out path.json]

Exit code: 0 when nothing differs, 3 when something does (the comparison is
still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p2b as B
from eval import compare_p2l as L
from eval import compare_runs as C
from eval import metrics as M
from eval.datasets.common import dump_json

P2R_FILES = [
    "p2r_checks.json",
    "p2r/volume.json",
    "p2r/det.json",
    "p2b/align_regress.json",
]
# the fast chain (K-P2.4 (3): the active block's checks, P2-R/R5): the
# conformance, regress and C-3 files of every fast chain, P2-B's and P2-R's
# files; Block 1's checks (P2-L, P2-P) run in the full chain only
FAST_BASE = [f for f in L.P2L_FAST_FILES if f not in L.P2L_FILES]
P2R_FAST_FILES = [*FAST_BASE, *B.P2B_FILES, *P2R_FILES]
# wall-clock fields of the P2-R files: none (results, counts, bytes)
P2R_TIMING: dict[str, list[tuple]] = {}
# the absolute fixture path of the chain's clone (as P2-B's)
P2R_INFO: dict[str, list[tuple]] = dict(B.P2B_INFO)
# no regress mode (the fast chain's regression is eval.fast_regress)
EXCLUDED: list[tuple[str, tuple, str, str]] = list(B.EXCLUDED)


def fast(a: Path, b: Path) -> dict:
    res = L._compare(a, b, P2R_FAST_FILES)
    res["files"] = P2R_FAST_FILES
    res["equal"] = not res["differing_fields"]
    return res


def pair(a: Path, b: Path) -> dict:
    res = B.pair(a, b)
    q = L._compare(a, b, P2R_FILES)
    res["files"] = [*res["files"], *P2R_FILES]
    res["n_fields"] += q["n_fields"]
    res["differing_fields"].update(q["differing_fields"])
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("mode", choices=["fast", "pair"])
    ap.add_argument("a", type=Path)
    ap.add_argument("b", type=Path)
    ap.add_argument("--out", type=Path)
    x = ap.parse_args(argv)
    try:
        res = fast(x.a, x.b) if x.mode == "fast" else pair(x.a, x.b)
    except (FileNotFoundError, M.MetricError, json.JSONDecodeError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    text = dump_json(C._finite(res))
    if x.out:
        x.out.write_text(text)
    print(text, end="")
    if not res["equal"]:
        print(f"{x.mode} FAILED: differing {sorted(res['differing_fields'])}", file=sys.stderr)
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
