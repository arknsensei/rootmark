"""Holdout-split (P1-B, TASKS B8; GOAL B-9): the frozen v3 matcher at its
compiled (shipping) THETA / THETA_SPLIT on `data/d1hs` — the split-weighted
slice built after the matcher froze — scored with the frozen D1-hard metrics
(`eval/metrics_v3.py`, H1-H1..H4) through the frozen per-document evaluation
of `eval/run_h1s_v3.py`, exactly as `eval/run_holdout.py` scores d1ho. Also
re-checks that d1hs, d1ho, D1-easy and D1-hard share no source and that the
realised split rates meet GOAL Ö3. Output `holdout_split.json`.

    python -m eval.run_holdout_split --run <RUN>
"""

from __future__ import annotations

import argparse
import json

import mdkernel_core

from eval import metrics_v3 as M3
from eval import runinfo
from eval.datasets import synthetic_edits_v4 as s4
from eval.datasets.common import die, file_blake3, load_manifest
from eval.run_h1s_v3 import MATCHER, doc_records

# GOAL B-9 (pre-registered; reported, never used for tuning)
THRESHOLDS = {"H1-H1": (">=", 0.90), "H1-H2": ("<=", 0.03), "H1-H3": (">=", 0.70)}


def verify_data() -> dict:
    """Fail-fast: data/d1hs must match MANIFEST.d1hs.json."""
    m = load_manifest(s4.MANIFEST)
    root = s4.OUT
    on_disk = {
        p.relative_to(root).as_posix(): file_blake3(p)
        for p in sorted(root.rglob("*"))
        if p.is_file()
    }
    if on_disk != m["files"]:
        bad = sorted(set(on_disk.items()) ^ set(m["files"].items()))[:5]
        raise RuntimeError(f"data/d1hs does not match MANIFEST.d1hs.json (e.g. {bad})")
    r = m["realised"]["rates"]
    if r.get("split_edit", 0) < s4.MIN_SPLIT_EDIT or r.get("split_three", 0) < s4.MIN_SPLIT_THREE:
        raise RuntimeError(f"realised split rates below GOAL Ö3: {r}")
    return m


def verdict(agg: dict) -> dict:
    out = {}
    for code, (op, t) in THRESHOLDS.items():
        v = agg[code]
        ok = v >= t if op == ">=" else v <= t
        out[code] = {"value": v, "threshold": f"{op} {t}", "pass": ok}
    return out


def run(run_id: str) -> dict:
    m = verify_data()
    sources = s4.sources()
    if len(sources) != s4.N_DOCS:
        raise RuntimeError(f"d1hs has {len(sources)} sources, expected {s4.N_DOCS}")
    _, texts = s4.book_texts()
    dis = s4.disjointness(sources, texts)
    theta, theta_split = mdkernel_core.matcher_defaults(MATCHER)
    ids = [d["id"] for d in json.loads((s4.OUT / "docs.json").read_text())]
    recs = [r for i in ids for r in doc_records(s4.OUT / "docs" / i, theta, theta_split)]
    agg = M3.aggregate(recs, "hard")
    out = {
        "set": "d1hs",
        "matcher": MATCHER,
        "theta": theta,
        "theta_split": theta_split,
        **{k: agg[k] for k in M3.CODES["hard"]},
        "holdout_split": {"n_docs": len(ids), "n_steps": len(recs), **agg},
        "b9": verdict(agg),
        "disjoint": dis,
        "realised_ops": m["realised"],
        "eligible_docs": m["eligible_docs"],
        "source": m["source"],
    }
    runinfo.write_json(run_id, "holdout_split.json", out)
    runinfo.record_phase(
        run_id,
        "holdout_split",
        {"matcher": MATCHER, "theta": theta, "theta_split": theta_split},
        ["holdout_split.json"],
    )
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    out = run(a.run)
    print(json.dumps({"codes": {k: out[k] for k in M3.CODES["hard"]}, "b9": out["b9"]}, indent=1))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
