"""H4-V2 (EVAL_DELTA §Metrikler): v2 behaviour reproduced through the
`matcher="v2"` flag.

1. `data/d3/docs` (verified against `MANIFEST.d3.json`) ingested from scratch
   into a fresh vault with matcher v2 → `events_hash` must equal
   `e6945bbc2da9c47376e017e815d7e51b`.
2. v2 D1 (`data/d1`, verified against `MANIFEST.d1.json`; v2 generator, v2
   truth) evaluated on its test split by the v2 runner and v2 `metrics.py`,
   under matcher v2 with the compiled V2 constants → every field equal to
   `reports/runs/20261013T154851Z-f133d0b/h1s.json` as committed at `v2-final`.

Writes `h4_v2repro.json`.

    python -m eval.run_h4_v2repro --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path

import rootmark_core

from eval import runinfo
from eval.compare_runs import leaves
from eval.d3_v3 import ingest_docs, matcher_env, verify_docs
from eval.datasets import vault as d3
from eval.datasets.common import ROOT, die, dump_json, git, load_manifest

MATCHER = "v2"
D3_EXPECTED = "e6945bbc2da9c47376e017e815d7e51b"
REF_TAG = "v2-final"
REF_RUN = "20261013T154851Z-f133d0b"
REF_PATH = f"reports/runs/{REF_RUN}/h1s.json"


def d3_condition() -> dict:
    manifest = load_manifest(d3.MANIFEST)
    verify_docs(manifest)
    with tempfile.TemporaryDirectory() as t:
        v = ingest_docs(Path(t), MATCHER)
        h = v.events_hash()
        lock = v.matcher_lock()
    if lock != MATCHER:
        raise RuntimeError(f"vault locked to {lock}, expected {MATCHER}")
    return {
        "events_hash": h,
        "expected": D3_EXPECTED,
        "manifest_events_hash": manifest["events_hash"],
        "equal": h == D3_EXPECTED,
    }


def h1s_condition() -> dict:
    from eval import run_h1s  # the v2 runner (v2 `metrics.py`, v2 truth)

    ref = json.loads(git("show", f"{REF_TAG}:{REF_PATH}", cwd=ROOT))
    with matcher_env(MATCHER):
        run_h1s.verify_d1()
        theta, theta_split = rootmark_core.matcher_defaults(MATCHER)
        m = run_h1s.evaluate(run_h1s.splits()["test"], None, None)
    # compared as written to disk (JSON), exactly like the reference file
    got = json.loads(dump_json({"split": "test", "theta": theta, "theta_split": theta_split, **m}))
    lg, lr = leaves(got), leaves(ref)
    fields = sorted(set(lg) | set(lr), key=str)
    diff = [
        {
            "field": "/".join(map(str, f)),
            "got": lg.get(f, "<missing>"),
            "ref": lr.get(f, "<missing>"),
        }
        for f in fields
        if f not in lg or f not in lr or lg[f] != lr[f] or type(lg[f]) is not type(lr[f])
    ]
    return {
        "reference": f"{REF_TAG}:{REF_PATH}",
        "n_fields": len(fields),
        "n_differing": len(diff),
        "differing": diff[:20],
        "equal": not diff,
        **{k: got[k] for k in ("H1-S1", "H1-S2", "H1-S3", "H1-S4")},
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    d3c = d3_condition()
    h1c = h1s_condition()
    out = {
        "matcher": MATCHER,
        "H4-V2": {"d3": d3c, "h1s": h1c, "equal": d3c["equal"] and h1c["equal"]},
    }
    runinfo.write_json(a.run, "h4_v2repro.json", out)
    runinfo.record_phase(a.run, "h4_v2repro", {"matcher": MATCHER}, ["h4_v2repro.json"])
    # a mismatch is a hypothesis result (H4-V2 fails), reported, not an error
    print(
        json.dumps(
            {"d3": d3c["equal"], "h1s_fields": h1c["n_fields"], "h1s_equal": h1c["equal"]}, indent=1
        )
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
