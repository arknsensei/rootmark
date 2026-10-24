"""P2-P run comparisons (TASKS P5; GOAL P2-P P-6, P-8): the P2-L
comparators as they are (`compare_p2l`) and the P2-P result file.

`fast <run_A> <run_B>` (`scripts/verify.sh` with `FAST=1`): the fast
chain's files (`P2P_FAST_FILES`: P2-L's and `p2p_checks.json`), every field
but the absolute fixture path of the chain's clone (`P2P_INFO`, ortam).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): `compare_p2l.pair` (the
P2-A pair set and the P2-L files) and `P2P_FILES`, every field.

    python -m eval.compare_p2p fast <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2p pair <run_A> <run_B> [--out path.json]

Exit code: 0 when nothing differs, 3 when something does (the comparison is
still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p2l as L
from eval import compare_runs as C
from eval import metrics as M
from eval.datasets.common import dump_json

P2P_FAST_FILES = [*L.P2L_FAST_FILES, "p2p_checks.json"]
P2P_FILES = ["p2p_checks.json"]
# wall-clock fields of the P2-P file: none (test names and results only)
P2P_TIMING: dict[str, list[tuple]] = {}
# the absolute fixture path of the chain's clone (as P2-L's)
P2P_INFO: dict[str, list[tuple]] = dict(L.P2L_INFO)
# no regress mode (the fast chain's regression is eval.fast_regress)
EXCLUDED: list[tuple[str, tuple, str, str]] = []


def fast(a: Path, b: Path) -> dict:
    res = L._compare(a, b, P2P_FAST_FILES)
    res["files"] = P2P_FAST_FILES
    res["equal"] = not res["differing_fields"]
    return res


def pair(a: Path, b: Path) -> dict:
    res = L.pair(a, b)
    q = L._compare(a, b, P2P_FILES)
    res["files"] = [*res["files"], *P2P_FILES]
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
