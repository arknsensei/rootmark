"""P1-C run comparisons (GOAL C-6, C-11; TASKS C9), with the frozen field
comparison of `eval/compare_runs.py` (same leaves, same latency exclusions).

`pair <run_A> <run_B>` (C-6, `scripts/verify.sh pair`): the P1-B result set
(`compare_p1b.RESULT_FILES_P1B`) plus `p1b1_checks.json`, `c3_quality.json`,
`p1c_checks.json` and `p1c_index.json` (the 200 rankings with scores and the
blake3 of every index file — lexical, vectors, hot), every field; the
governance vault's `events.jsonl`, `traces.jsonl` and `checkpoints/` byte for
byte (`compare_p1b.bytes_equal`). No exclusion. `c8_latency.json` (C-8,
machine-dependent) is not compared.

`regress <p1b1_run> <run>` (C-11): the P1-B result set plus
`p1b1_checks.json` of the P1-B.1 official RUN against this RUN, every field.
The only fields allowed to differ are the ones P1-C changes by decision:

- `p1b_checks.json:fixtures_manifest_hash` — `spec/fixtures/MANIFEST.json`
  gained the C2-09 (K-C0.2) and C3-01..04 (GOAL C-1) rows; every listed
  C1/C2 file is byte-identical (the C1/C2 entries' hashes are compared
  field by field and must not differ);
- `p1b1_checks.json:tests_seen` — the number of tests in `p1b1_tests.log`
  (`compat_v3final` + `trace_id_v101`) grew by the three K-C0.1 tests
  (ADDENDUM C-13); every B1 criterion field is compared and must not
  differ.

Every other difference, a missing file or a missing field fails.

    python -m eval.compare_p1c pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p1c regress <p1b1_run> <run> [--out path.json]

Exit code: 0 when nothing (outside the exclusions) differs, 3 when something
does (the comparison is still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1b import RESULT_FILES_P1B, bytes_equal
from eval.datasets.common import dump_json

PAIR_FILES = [
    *RESULT_FILES_P1B,
    "p1b1_checks.json",
    "c3_quality.json",
    "p1c_checks.json",
    "p1c_index.json",
]
REGRESS_FILES = [*RESULT_FILES_P1B, "p1b1_checks.json"]
REGRESS_ALLOWED = {
    "p1b_checks.json:fixtures_manifest_hash",
    "p1b1_checks.json:tests_seen",
}


def pair(a: Path, b: Path) -> dict:
    res = C.compare(a, b, PAIR_FILES)
    res["bytes"] = bytes_equal(a, b)
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["C-6_equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def regress(ref: Path, run: Path) -> dict:
    res = C.compare(ref, run, REGRESS_FILES)
    diff = res["differing_fields"]
    res["reference_run"] = ref.name
    res["allowed_fields"] = sorted(REGRESS_ALLOWED)
    res["differing_excluded"] = sorted(k for k in diff if k in REGRESS_ALLOWED)
    res["differing_outside_exclusions"] = sorted(k for k in diff if k not in REGRESS_ALLOWED)
    res["C-11_equal"] = not res["differing_outside_exclusions"]
    return res


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("mode", choices=["pair", "regress"])
    ap.add_argument("a", type=Path)
    ap.add_argument("b", type=Path)
    ap.add_argument("--out", type=Path)
    x = ap.parse_args(argv)
    try:
        res = pair(x.a, x.b) if x.mode == "pair" else regress(x.a, x.b)
    except (FileNotFoundError, M.MetricError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    text = dump_json(C._finite(res))
    if x.out:
        x.out.write_text(text)
    print(text, end="")
    ok = res["C-6_equal"] if x.mode == "pair" else res["C-11_equal"]
    if not ok:
        what = res.get("differing_outside_exclusions", res["differing_fields"])
        print(f"{x.mode} FAILED: differing {what}", file=sys.stderr)
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
