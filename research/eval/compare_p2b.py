"""P2-B run comparisons (TASKS B5; GOAL P2-B B-9, B-10): the P2-P
comparators as they are (`compare_p2p`) and the P2-B result files.

`fast <run_A> <run_B>` (`scripts/verify.sh` with `FAST=1`): the fast
chain's files (`P2B_FAST_FILES`: P2-P's and the P2-B files), every field but
the absolute fixture path of the chain's clone (`P2B_INFO`, ortam).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): `compare_p2p.pair` (the
P2-A pair set, the P2-L and P2-P files) and `P2B_FILES`, every field.

The P2-B files (`make p2b-checks`): `p2b_checks.json` (the round's tests by
criterion), `p2b/align.json` (B-3/B-4 on D-NOTES), `p2b/det.json` (block
locations, the event stream and the time filter, whole: B-9),
`p2b/client_checks.json`, `p2b/obsidian_e2e.json`, `p2b/vscode_e2e.json`
(B-7, B-8: both clients and the contract check of their calls).

    python -m eval.compare_p2b fast <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2b pair <run_A> <run_B> [--out path.json]

Exit code: 0 when nothing differs, 3 when something does (the comparison is
still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p2l as L
from eval import compare_p2p as P
from eval import compare_runs as C
from eval import metrics as M
from eval.datasets.common import dump_json

P2B_FILES = [
    "p2b_checks.json",
    "p2b/align.json",
    "p2b/det.json",
    "p2b/client_checks.json",
    "p2b/obsidian_e2e.json",
    "p2b/vscode_e2e.json",
]
P2B_FAST_FILES = [*P.P2P_FAST_FILES, *P2B_FILES]
# wall-clock fields of the P2-B files: none (results, counts, bytes)
P2B_TIMING: dict[str, list[tuple]] = {}
# the absolute fixture path of the chain's clone (as P2-P's)
P2B_INFO: dict[str, list[tuple]] = dict(P.P2P_INFO)
# no regress mode (the fast chain's regression is eval.fast_regress)
EXCLUDED: list[tuple[str, tuple, str, str]] = list(P.EXCLUDED)


def fast(a: Path, b: Path) -> dict:
    res = L._compare(a, b, P2B_FAST_FILES)
    res["files"] = P2B_FAST_FILES
    res["equal"] = not res["differing_fields"]
    return res


def pair(a: Path, b: Path) -> dict:
    res = P.pair(a, b)
    q = L._compare(a, b, P2B_FILES)
    res["files"] = [*res["files"], *P2B_FILES]
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
