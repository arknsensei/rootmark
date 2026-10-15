"""C1-9 (GOAL P1-C.1; K-C1.2): writes the Rust quality regression reference
`data/quality_ref_rust.json` from the P1-C official RUN's `c3_quality.json`
(cells and per-query metrics of `{hybrid, flat}/structural`, k 10, D3 + D4
test split) — **only** when a `c1_variant.json` given as evidence passes
C1-1 against that same RUN (K-C1.2: no reference change without it).
Trace ids and timings are not part of the reference.

    python -m eval.gen_quality_ref --c1-variant <RUN_DIR>/c1_variant.json
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from eval.datasets.common import DATA, ROOT, die, dump_json

SOURCE_RUN = ROOT.parent / "reports" / "rounds" / "P1-C" / "runs" / "20261015T014417Z-9aa326b"
OUT = DATA / "quality_ref_rust.json"
METRICS = ("ndcg@10", "recall@5", "recall@10", "mrr", "full_cover@10")


def reference(c3: dict) -> dict:
    return {
        "cells": c3["cells"],
        "per_query": {
            mode: [{k: r[k] for k in ("id", "type", *METRICS)} for r in rows]
            for mode, rows in sorted(c3["per_query"].items())
        },
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--c1-variant", type=Path, required=True)
    a = ap.parse_args(argv)
    ev = json.loads(a.c1_variant.read_text(encoding="utf-8"))
    if ev.get("criterion") != "C1-1" or ev.get("pass") is not True:
        raise RuntimeError(f"{a.c1_variant}: C1-1 did not pass; the reference is not changed")
    if ev["rust_run"] != SOURCE_RUN.name:
        raise RuntimeError(f"C1-1 evidence is against {ev['rust_run']}, not {SOURCE_RUN.name}")
    c3 = json.loads((SOURCE_RUN / "c3_quality.json").read_text(encoding="utf-8"))
    out = {
        "what": "Rust index retrieval quality on D3 + D4 test split (P1-C.1 K-C1.2)",
        "source_run": SOURCE_RUN.name,
        "source_file": "c3_quality.json",
        "c1_1_evidence": {
            "run": a.c1_variant.parent.name,
            "variant": ev["variant"],
            "threshold_abs_delta": ev["threshold_abs_delta"],
            "max_abs_delta": max(abs(d) for c in ev["C1-1"].values() for d in c["delta"].values()),
            "pass": ev["pass"],
        },
        "index": {k: c3["index"][k] for k in ("embedder_id", "records", "snapshots")},
        "checkpoint_id": c3["checkpoint_id"],
        "n_questions": c3["n_questions"],
        **reference(c3),
    }
    OUT.write_text(dump_json(out))
    print(
        f"wrote {OUT.relative_to(ROOT)} from {SOURCE_RUN.name} (C1-1 evidence {out['c1_1_evidence']['run']})"
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
