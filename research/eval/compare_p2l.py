"""P2-L run comparisons (TASKS L0 d, L5; GOAL P2-L L-9, L-11), with the
frozen comparators of earlier rounds (`compare_p2a`: leaf extraction and
field comparison, the P2-A pair set).

`fast <run_A> <run_B>` (`scripts/verify.sh` with `FAST=1`, K-P2.4 (3)): the
files of the fast chain (`P2L_FAST_FILES`), every field but the absolute
fixture path of the chain's clone (`P2L_INFO`, ortam).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): `compare_p2a.pair`
(everything since P1-A) and the P2-L result files (`P2L_FILES`), every field
but `P2L_INFO`.

    python -m eval.compare_p2l fast <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2l pair <run_A> <run_B> [--out path.json]

Exit code: 0 when nothing differs, 3 when something does (the comparison is
still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p2a as A
from eval import compare_runs as C
from eval import metrics as M
from eval.datasets.common import dump_json

P2L_FAST_FILES = [
    "fast/conformance.json",
    "fast/conformance_rootmark.json",
    "fast/conformance_equiv.json",
    "fast/regress.json",
    "c3_quality.json",
    "p2l_checks.json",
    "p2l_quality.json",
    "p2l/d_tr.json",
    "p2l/english.json",
    "p2l/migrate.json",
]
P2L_FILES = [
    "p2l_checks.json",
    "p2l_quality.json",
    "p2l/d_tr.json",
    "p2l/english.json",
    "p2l/migrate.json",
]
# wall-clock fields of the P2-L files: none
P2L_TIMING: dict[str, list[tuple]] = {}
# the absolute fixture path of the chain's clone (research/eval/DETERMINISTIC_FIELDS.md)
P2L_INFO: dict[str, list[tuple]] = {
    "fast/conformance.json": [("fixtures_root",)],
}
# decision exclusions of a regress: none (P2-L has no regress mode; the fast
# chain's regression is eval.fast_regress)
EXCLUDED: list[tuple[str, tuple, str, str]] = []


def _info(name: str, path: tuple) -> tuple | None:
    for pre in P2L_INFO.get(name, []):
        if len(path) >= len(pre) and all(str(x) == p for p, x in zip(pre, path, strict=False)):
            return (name, pre, "ortam", "clone path")
    return None


def _compare(a: Path, b: Path, files: list[str]) -> dict:
    res = A.compare_files(a, b, files, _info)
    # an ortam field is aside, not a difference (compare_files lists it as an
    # excluded difference)
    res["environment_fields"] = sorted(res.pop("excluded_differences"))
    return res


def fast(a: Path, b: Path) -> dict:
    res = _compare(a, b, P2L_FAST_FILES)
    res["files"] = P2L_FAST_FILES
    res["equal"] = not res["differing_fields"]
    return res


def pair(a: Path, b: Path) -> dict:
    res = A.pair(a, b)
    q = _compare(a, b, P2L_FILES)
    res["files"] = [*res["files"], *P2L_FILES]
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
