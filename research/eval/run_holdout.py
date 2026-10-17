"""Holdout (P1-A, TASKS A5; GOAL A-2): the frozen v3 matcher at its compiled
(shipping) THETA / THETA_SPLIT on `data/d1ho` — a set never used for design or
tuning — scored with the frozen D1-hard metrics (`eval/metrics_v3.py`,
H1-H1..H4) through the frozen per-document evaluation of
`eval/run_h1s_v3.py`. Also checks programmatically that holdout, D1-easy and
D1-hard share no source document. Output `holdout.json`.

    python -m eval.run_holdout --run <RUN>
"""

from __future__ import annotations

import argparse
import json

import rootmark_core

from eval import metrics_v3 as M3
from eval import runinfo
from eval.datasets import synthetic_edits_v2 as s2
from eval.datasets import synthetic_edits_v3 as s3
from eval.datasets.common import die, file_blake3, load_manifest
from eval.run_h1s_v3 import MATCHER, doc_records


def verify_holdout() -> dict:
    """Fail-fast: data/d1ho must match MANIFEST.d1ho.json."""
    m = load_manifest(s3.MANIFEST_HOLDOUT)
    root = s3.HOLDOUT_DIR
    on_disk = {
        p.relative_to(root).as_posix(): file_blake3(p)
        for p in sorted(root.rglob("*"))
        if p.is_file()
    }
    if on_disk != m["files"]:
        bad = sorted(set(on_disk.items()) ^ set(m["files"].items()))[:5]
        raise RuntimeError(f"data/d1ho does not match MANIFEST.d1ho.json (e.g. {bad})")
    return m


def disjointness() -> dict:
    easy = set(s2.easy_sources())
    hard = set(s3.hard_sources())
    holdout = set(s3.holdout_sources())
    out = {
        "n": {"easy": len(easy), "hard": len(hard), "holdout": len(holdout)},
        "holdout_and_easy": len(holdout & easy),
        "holdout_and_hard": len(holdout & hard),
        "easy_and_hard": len(easy & hard),
        "all_three": len(holdout & easy & hard),
    }
    if any(out[k] for k in ("holdout_and_easy", "holdout_and_hard", "easy_and_hard")):
        raise RuntimeError(f"holdout / D1-easy / D1-hard are not disjoint: {out}")
    if len(holdout) != s3.N_HOLDOUT:
        raise RuntimeError(f"holdout has {len(holdout)} sources, expected {s3.N_HOLDOUT}")
    return out


def run(run_id: str) -> dict:
    m = verify_holdout()
    dis = disjointness()
    theta, theta_split = rootmark_core.matcher_defaults(MATCHER)
    ids = [d["id"] for d in json.loads((s3.HOLDOUT_DIR / "docs.json").read_text())]
    recs = [r for i in ids for r in doc_records(s3.HOLDOUT_DIR / "docs" / i, theta, theta_split)]
    agg = M3.aggregate(recs, "hard")
    out = {
        "set": "d1ho",
        "matcher": MATCHER,
        "theta": theta,
        "theta_split": theta_split,
        **{k: agg[k] for k in M3.CODES["hard"]},
        "holdout": {"n_docs": len(ids), "n_steps": len(recs), **agg},
        "disjoint": dis,
        "realised_ops": m["realised"],
        "eligible_docs": m["eligible_docs"],
    }
    runinfo.write_json(run_id, "holdout.json", out)
    runinfo.record_phase(
        run_id,
        "holdout",
        {"matcher": MATCHER, "theta": theta, "theta_split": theta_split},
        ["holdout.json"],
    )
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    out = run(a.run)
    print(json.dumps({k: out[k] for k in M3.CODES["hard"]}, indent=1))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
