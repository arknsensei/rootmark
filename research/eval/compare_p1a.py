"""P1-A run comparison (GOAL A-4; VERIFY V1): every quality field of every
P1-A result file of two runs, field by field, with the frozen comparison of
`eval/compare_runs.py` (same leaves, same latency exclusions, same maximum
relative difference). The P1-A set is the v3 set plus `holdout.json` and
`split_grid.json`. A missing file or field is an error.

    python -m eval.compare_p1a <run_dir_A> <run_dir_B> [--out path.json]

Exit code: 0 when the maximum relative difference is 0, 3 when it is not (the
comparison is still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from eval import compare_runs as C
from eval import metrics as M
from eval.datasets.common import dump_json

RESULT_FILES_P1A = [*C.RESULT_FILES_V3, "holdout.json", "split_grid.json"]


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("run_a", type=Path)
    ap.add_argument("run_b", type=Path)
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    try:
        res = C.compare(a.run_a, a.run_b, RESULT_FILES_P1A)
    except (FileNotFoundError, M.MetricError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    res["A-4_equal"] = res["H4-2_equal"]
    text = dump_json(C._finite(res))
    if a.out:
        a.out.write_text(text)
    print(text, end="")
    if not res["A-4_equal"]:
        print(
            f"A-4 FAILED: max relative difference {res['max_relative_difference']}",
            file=sys.stderr,
        )
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
