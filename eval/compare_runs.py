"""H4-2: compares the quality metrics of two independent runs field by field.

Every JSON leaf of every result file is compared; the only exclusions are the
wall-clock latency paths listed in `LATENCY_PATHS` (fixed here). A missing
file, or a field present in one run but not the other, is an error.

    python -m eval.compare_runs <runs_dir_A>/<RUN_A> <runs_dir_B>/<RUN_B> [--out path.json]
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import metrics as M
from eval.datasets.common import dump_json

RESULT_FILES = [
    "h1s_tuning.json",
    "h1s.json",
    "h1r.json",
    "h2_tuning.json",
    "h2.json",
    "h3.json",
    "h4.json",
]

# The only excluded fields: wall-clock latencies (docs/03_EVAL.md §H4). A path
# matches if its leading parts equal a pattern ("*" matches one part, a
# pattern ending at a subtree excludes the whole subtree).
LATENCY_PATHS = {
    "h2.json": [("H2-3",), ("latency_ms",)],
    "h3.json": [
        ("H3-1",),
        ("summary", "p50"),
        ("summary", "p95"),
        ("latency", "latencies_ms"),
        ("latency", "rows", "*", "ms"),
    ],
}


def _excluded(name: str, path: tuple) -> bool:
    for pat in LATENCY_PATHS.get(name, []):
        if len(path) >= len(pat) and all(
            p == "*" or p == str(x) for p, x in zip(pat, path, strict=False)
        ):
            return True
    return False


def leaves(obj, prefix: tuple = ()) -> dict[tuple, object]:
    """Every scalar leaf of a JSON value, keyed by its path (list items by index)."""
    if isinstance(obj, dict):
        out = {}
        for k, v in obj.items():
            out.update(leaves(v, prefix + (k,)))
        return out
    if isinstance(obj, list):
        out = {prefix + ("#len",): len(obj)}
        for i, v in enumerate(obj):
            out.update(leaves(v, prefix + (i,)))
        return out
    return {prefix: obj}


def _key(path: tuple) -> str:
    # `metrics.max_relative_difference` addresses fields by dotted path; a
    # flat dict whose keys contain no dots is addressed by its own key
    return "›".join(str(p).replace(".", "·") for p in path)


def _load(run_dir: Path, name: str) -> dict:
    p = run_dir / name
    if not p.exists():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text())


def compare(a_dir: Path, b_dir: Path) -> dict:
    per_file = {}
    worst = 0.0
    differing: dict = {}
    n = 0
    for name in RESULT_FILES:
        la, lb = leaves(_load(a_dir, name)), leaves(_load(b_dir, name))
        la = {p: v for p, v in la.items() if not _excluded(name, p)}
        lb = {p: v for p, v in lb.items() if not _excluded(name, p)}
        if set(la) != set(lb):
            only = sorted(_key(p) for p in set(la) ^ set(lb))[:5]
            raise M.MetricError(f"{name}: field sets differ, e.g. {only}")
        fa = {_key(p): v for p, v in la.items()}
        fb = {_key(p): v for p, v in lb.items()}
        r = M.max_relative_difference(fa, fb, sorted(fa))
        per_file[name] = r["max_relative_difference"]
        worst = max(worst, r["max_relative_difference"])
        differing.update({f"{name}:{k}": v for k, v in r["differing_fields"].items()})
        n += r["n_fields"]
    return {
        "run_a": str(a_dir),
        "run_b": str(b_dir),
        "n_fields": n,
        "max_relative_difference": worst,
        "per_file": per_file,
        "differing_fields": differing,
        "H4-2_equal": worst == 0.0,
        "excluded_latency_paths": {k: ["/".join(p) for p in v] for k, v in LATENCY_PATHS.items()},
    }


def _finite(obj):
    if isinstance(obj, float) and obj == float("inf"):
        return "inf"
    if isinstance(obj, dict):
        return {k: _finite(v) for k, v in obj.items()}
    return obj


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("run_a", type=Path)
    ap.add_argument("run_b", type=Path)
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    try:
        res = compare(a.run_a, a.run_b)
    except (FileNotFoundError, M.MetricError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    text = dump_json(_finite(res))
    if a.out:
        a.out.write_text(text)
    print(text, end="")


if __name__ == "__main__":
    main()
