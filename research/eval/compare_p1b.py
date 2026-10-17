"""P1-B run comparison (GOAL B-4; VERIFY V1): every quality field of every
P1-B result file of two runs, field by field, with the frozen comparison of
`eval/compare_runs.py` (same leaves, same latency exclusions), plus a byte
comparison of the governance scenario's `events.jsonl`, `traces.jsonl` and
`checkpoints/` directory. The P1-B set is the P1-A set
(`compare_p1a.RESULT_FILES_P1A`) plus `holdout_split.json`, `p1b_checks.json`
and the scenario's `governance/governance.json`. A missing file, field or
artefact is an error.

    python -m eval.compare_p1b <run_dir_A> <run_dir_B> [--out path.json]

Exit code: 0 when the maximum relative difference is 0 and every artefact is
byte-equal, 3 when not (the comparison is still written), 1 on a missing file
or field.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1a import RESULT_FILES_P1A
from eval.datasets.common import dump_json, file_blake3

RESULT_FILES_P1B = [
    *RESULT_FILES_P1A,
    "holdout_split.json",
    "p1b_checks.json",
    "governance/governance.json",
]
VAULT = Path("governance/vault/.rootmark")
BYTE_FILES = [VAULT / "events.jsonl", VAULT / "traces.jsonl"]
BYTE_DIRS = [VAULT / "checkpoints"]


def _tree(d: Path) -> dict[str, str]:
    if not d.is_dir():
        raise FileNotFoundError(f"{d} missing")
    return {
        p.relative_to(d).as_posix(): file_blake3(p) for p in sorted(d.rglob("*")) if p.is_file()
    }


def bytes_equal(a: Path, b: Path) -> dict:
    out = {}
    for f in BYTE_FILES:
        pa, pb = a / f, b / f
        for p in (pa, pb):
            if not p.is_file():
                raise FileNotFoundError(f"{p} missing")
        out[f.as_posix()] = {
            "blake3": file_blake3(pa),
            "equal": pa.read_bytes() == pb.read_bytes(),
        }
    for d in BYTE_DIRS:
        ta, tb = _tree(a / d), _tree(b / d)
        if not ta:
            raise FileNotFoundError(f"{a / d} is empty")
        out[d.as_posix() + "/"] = {"files": len(ta), "equal": ta == tb}
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("run_a", type=Path)
    ap.add_argument("run_b", type=Path)
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    try:
        res = C.compare(a.run_a, a.run_b, RESULT_FILES_P1B)
        res["bytes"] = bytes_equal(a.run_a, a.run_b)
    except (FileNotFoundError, M.MetricError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["B-4_equal"] = res["H4-2_equal"] and res["bytes_equal"]
    text = dump_json(C._finite(res))
    if a.out:
        a.out.write_text(text)
    print(text, end="")
    if not res["B-4_equal"]:
        print(
            f"B-4 FAILED: max relative difference {res['max_relative_difference']}, "
            f"bytes equal {res['bytes_equal']}",
            file=sys.stderr,
        )
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
