"""P1-C.1 run comparisons (GOAL C1-8; DECISIONS K-C1.6), with the frozen
leaf extraction and field comparison (`compare_runs.leaves`,
`metrics.max_relative_difference`; latency paths excluded as there).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): the P1-C pair set
(`compare_p1c.PAIR_FILES`) plus `c1_variant.json` and `p1c1_checks.json`,
every field, and the governance vault's bytes (`compare_p1b.bytes_equal`).
No exclusion.

`regress <p1c_run> <run>` (C1-8): the P1-C pair set of the P1-C official
RUN against this RUN, every field, except the subtrees K-C1.6 names (each
caused by a decision of this round; listed in `ALLOWED`). A field set that
differs outside them, a missing file or a missing field fails.

    python -m eval.compare_p1c1 pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p1c1 regress <p1c_run> <run> [--out path.json]

Exit code: 0 when nothing (outside the exclusions) differs, 3 when something
does (the comparison is still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1b import bytes_equal
from eval.compare_p1c import PAIR_FILES as P1C_FILES
from eval.datasets.common import dump_json

PAIR_FILES = [*P1C_FILES, "c1_variant.json", "p1c1_checks.json"]

# K-C1.6: (file, path prefix) — the subtree under the prefix may differ
ALLOWED = [
    ("p1b_checks.json", ("fixtures_manifest_hash",)),
    ("p1b1_checks.json", ("criteria", "B1-3", "spec_version")),
    ("p1b1_checks.json", ("criteria", "B1-3", "pass")),
    ("p1c_checks.json", ("C-1", "spec_version")),
    ("p1c_checks.json", ("C-11", "pass")),
    ("p1c_checks.json", ("C-11", "differing_outside_exclusions")),
    ("p1c_checks.json", ("C-7", "proposal_hits_with_include_proposal")),
    ("p1c_index.json", ("index_files_build", "manifest.json")),
    ("p1c_index.json", ("index_files_compact", "manifest.json")),
    ("p1c_index.json", ("index_files_final", "manifest.json")),
]


def _allowed(name: str, path: tuple) -> str | None:
    for f, pre in ALLOWED:
        if f == name and path[: len(pre)] == pre:
            return f"{f}:{C._key(pre)}"
    return None


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def compare(a: Path, b: Path, files: list[str], allow: bool) -> dict:
    """Every leaf of every file; with `allow`, leaves under an `ALLOWED`
    prefix are compared but their differences (and field-set differences)
    are reported apart."""
    n = 0
    differing: dict = {}
    excluded: dict = {}
    for name in files:
        la = {p: v for p, v in C.leaves(_load(a, name)).items() if not C._excluded(name, p)}
        lb = {p: v for p, v in C.leaves(_load(b, name)).items() if not C._excluded(name, p)}
        for p in set(la) ^ set(lb):
            tag = _allowed(name, p) if allow else None
            if tag is None:
                raise M.MetricError(f"{name}: field sets differ at {C._key(p)}")
            excluded.setdefault(tag, []).append(f"{C._key(p)} only in {'A' if p in la else 'B'}")
        common = sorted(set(la) & set(lb), key=C._key)
        fa = {C._key(p): la[p] for p in common}
        fb = {C._key(p): lb[p] for p in common}
        r = M.max_relative_difference(fa, fb, sorted(fa))
        n += r["n_fields"]
        by_key = {C._key(p): p for p in common}
        for k, v in r["differing_fields"].items():
            tag = _allowed(name, by_key[k]) if allow else None
            if tag is None:
                differing[f"{name}:{k}"] = v
            else:
                excluded.setdefault(tag, []).append(f"{k}: {la[by_key[k]]!r} → {lb[by_key[k]]!r}")
    return {
        "run_a": str(a),
        "run_b": str(b),
        "files": files,
        "n_fields": n,
        "differing_fields": differing,
        "differing_excluded": {k: sorted(v) for k, v in sorted(excluded.items())},
    }


def pair(a: Path, b: Path) -> dict:
    res = compare(a, b, PAIR_FILES, allow=False)
    res["bytes"] = bytes_equal(a, b)
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def regress(ref: Path, run: Path) -> dict:
    res = compare(ref, run, P1C_FILES, allow=True)
    res["reference_run"] = ref.name
    res["allowed"] = [f"{f}:{C._key(p)}" for f, p in ALLOWED]
    res["equal"] = not res["differing_fields"]
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
    if not res["equal"]:
        print(f"{x.mode} FAILED: differing {sorted(res['differing_fields'])}", file=sys.stderr)
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
