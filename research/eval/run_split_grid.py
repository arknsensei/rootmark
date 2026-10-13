"""Split-threshold difficulty zone (P1-A, TASKS A5; GOAL A-7, information
only): the frozen v3 matcher on `data/d1sh` at the compiled THETA with
THETA_SPLIT ∈ {0.45, 0.55, 0.65, 0.75, 0.85}; per cell the frozen D1-hard
metrics (H1-H1..H4, objective S1 + (1-S2) + S3 + S4) and per-variant
retention / event recall; A-7 = number of distinct objective values (6
decimals, `metrics_v3.distinct_objectives`). Nothing here selects or writes a
constant. Also re-checks that no split piece is a substring of the old
block's normalised text. Output `split_grid.json`.

    python -m eval.run_split_grid --run <RUN>
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import mdkernel_core

from eval import metrics_v3 as M3
from eval import runinfo
from eval.datasets import synthetic_edits as se
from eval.datasets import synthetic_edits_v3 as s3
from eval.datasets.common import die, file_blake3, load_manifest
from eval.run_h1s_v3 import MATCHER, doc_records

THETA_SPLITS = [0.45, 0.55, 0.65, 0.75, 0.85]


def verify_split_hard() -> dict:
    m = load_manifest(s3.MANIFEST_SPLIT_HARD)
    root = s3.SPLIT_HARD_DIR
    on_disk = {
        p.relative_to(root).as_posix(): file_blake3(p)
        for p in sorted(root.rglob("*"))
        if p.is_file()
    }
    if on_disk != m["files"]:
        bad = sorted(set(on_disk.items()) ^ set(m["files"].items()))[:5]
        raise RuntimeError(f"data/d1sh does not match MANIFEST.d1sh.json (e.g. {bad})")
    return m


def _block_at(doc_dir: Path, file: str, ref: dict) -> se.Blk:
    blocks, _ = se.parse_blocks((doc_dir / file).read_text(encoding="utf-8"))
    hits = [b for b in blocks if b.version_id == ref["version_id"]]
    return hits[ref["k"]]


def substring_check(ids: list[str]) -> dict:
    """Every split piece differs from any substring of the old block (normalised)."""
    n_pieces = 0
    for i in ids:
        d = s3.SPLIT_HARD_DIR / "docs" / i
        t = json.loads((d / "truth.json").read_text())
        prev_file = t["base_file"]
        for s in t["steps"]:
            if s["scored"] and s["op"] in M3.SPLIT_OPS:
                ((old, news),) = s["mapping"]
                old_norm = s3.norm_text(_block_at(d, prev_file, old))
                for n in news:
                    n_pieces += 1
                    if s3.norm_text(_block_at(d, s["file"], n)) in old_norm:
                        raise RuntimeError(f"{i} step {s['step']}: a split piece is a substring")
            prev_file = s["file"]
    return {"split_pieces_checked": n_pieces, "substring_pieces": 0}


def variants(ids: list[str]) -> list[str | None]:
    out = []
    for i in ids:
        t = json.loads((s3.SPLIT_HARD_DIR / "docs" / i / "truth.json").read_text())
        out += [s.get("variant") for s in t["steps"] if s["scored"]]
    return out


def per_variant(recs: list[dict], vs: list[str | None]) -> dict:
    out = {}
    for v in sorted({x for x in vs if x}):
        rs = [r for r, x in zip(recs, vs, strict=True) if x == v]
        ret = [x for r in rs for x in r["retained"]]
        ev = M3.event_f1(rs)
        out[v] = {
            "steps": len(rs),
            "retention": sum(ret) / len(ret),
            "event_recall": ev["recall"],
            "event_precision": ev["precision"],
        }
    return out


def run(run_id: str) -> dict:
    m = verify_split_hard()
    ids = [d["id"] for d in json.loads((s3.SPLIT_HARD_DIR / "docs.json").read_text())]
    sub = substring_check(ids)
    vs = variants(ids)
    theta, compiled_split = mdkernel_core.matcher_defaults(MATCHER)
    grid = []
    for ts in THETA_SPLITS:
        recs = [r for i in ids for r in doc_records(s3.SPLIT_HARD_DIR / "docs" / i, theta, ts)]
        if len(recs) != len(vs):
            raise RuntimeError(f"{len(recs)} records for {len(vs)} scored steps")
        agg = M3.aggregate(recs, "hard")
        row = {"theta": theta, "theta_split": ts, **{k: agg[k] for k in M3.CODES["hard"]}}
        row["objective"] = M3.objective(agg, "hard")
        row["per_variant"] = per_variant(recs, vs)
        row["H1-H3_detail"] = agg["H1-H3_detail"]
        grid.append(row)
        print(
            f"theta_split={ts:.2f} "
            + " ".join(f"{k}={row[k]:.4f}" for k in M3.CODES["hard"])
            + f" obj={row['objective']:.6f}"
        )
    out = {
        "set": "d1sh",
        "matcher": MATCHER,
        "theta": theta,
        "compiled_theta_split": compiled_split,
        "theta_splits": THETA_SPLITS,
        "grid": grid,
        "A-7": M3.distinct_objectives(r["objective"] for r in grid),
        "n_docs": len(ids),
        "n_steps": len(vs),
        "substring_check": sub,
        "realised_ops": m["realised"],
        "note": "information only; no constant is selected or written",
    }
    runinfo.write_json(run_id, "split_grid.json", out)
    runinfo.record_phase(
        run_id,
        "split_grid",
        {"matcher": MATCHER, "theta": theta, "theta_splits": THETA_SPLITS},
        ["split_grid.json"],
    )
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    out = run(a.run)
    print(f"A-7 distinct objective values = {out['A-7']}")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
