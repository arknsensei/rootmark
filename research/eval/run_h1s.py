"""H1-S on D1: `--phase tune` (THETA × THETA_SPLIT grid on the tuning split)
and `--phase test` (compiled constants must equal the tuning selection; test
split only). Outputs `h1s_tuning.json` / `h1s.json` in the run directory.

    python -m eval.run_h1s --phase tune --run <RUN> [--apply]
    python -m eval.run_h1s --phase test --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
import tempfile
from pathlib import Path

import rootmark_core

from eval import metrics, runinfo
from eval import truth as T
from eval.datasets.common import DATA, ROOT, die, file_blake3, load_manifest

D1 = DATA / "d1"
THETAS = [0.50, 0.55, 0.60, 0.65, 0.70]
THETA_SPLITS = [0.60, 0.70, 0.80]
DEFAULT_POINT = (0.60, 0.70)
# out-of-grid diagnostic point, used only when the whole grid is flat
PROBE = (0.95, 0.99)
LINEAGE_RS = ROOT / "core" / "src" / "lineage.rs"


def verify_d1() -> dict:
    """Fail-fast: the D1 files on disk must match MANIFEST.d1.json."""
    m = load_manifest(DATA / "MANIFEST.d1.json")
    on_disk = {
        p.relative_to(D1).as_posix(): file_blake3(p) for p in sorted(D1.rglob("*")) if p.is_file()
    }
    if on_disk != m["files"]:
        bad = sorted(set(on_disk.items()) ^ set(m["files"].items()))[:5]
        raise RuntimeError(f"data/d1 does not match MANIFEST.d1.json (e.g. {bad})")
    return m


def splits() -> dict:
    s = json.loads((D1 / "splits.json").read_text())
    if set(s["tune"]) & set(s["test"]):
        raise RuntimeError("D1 tune/test splits intersect")
    return s


def doc_records(doc_dir: Path, theta: float | None, theta_split: float | None) -> list[dict]:
    t = json.loads((doc_dir / "truth.json").read_text())
    doc = t["doc_path"]
    records = []
    with tempfile.TemporaryDirectory() as tmp:
        v = rootmark_core.Vault(tmp, theta, theta_split)
        v.ingest(doc, (doc_dir / t["base_file"]).read_text(encoding="utf-8"), "Synthetic", "d1")
        for s in t["steps"]:
            text = (doc_dir / s["file"]).read_text(encoding="utf-8")
            if not s["scored"]:
                v.ingest(doc, text, "Synthetic", "d1")
                continue
            prev = [lin for lin, _ in v.snapshot(doc)["blocks"]]
            olds = [None if o is None else T.resolve(v, o, doc) for o, _ in s["mapping"]]
            r = v.ingest(doc, text, "Synthetic", "d1")
            news = [[T.resolve(v, n, doc) for n in ns] for _, ns in s["mapping"]]
            records.append(
                metrics.h1s_step_record(s["op"], list(zip(olds, news)), prev, r["events"])
            )
    expected = sum(1 for s in t["steps"] if s["scored"])
    if len(records) != expected:
        raise RuntimeError(f"{doc_dir.name}: {len(records)} scored steps, expected {expected}")
    return records


def evaluate(ids: list[str], theta: float | None, theta_split: float | None) -> dict:
    if not ids:
        raise RuntimeError("no documents to evaluate")
    recs = [r for i in ids for r in doc_records(D1 / "docs" / i, theta, theta_split)]
    out = metrics.h1s_aggregate(recs)
    out["n_docs"] = len(ids)
    out["n_steps"] = len(recs)
    return out


def select(grid: list[dict]) -> dict:
    """Max objective; ties → closest to (0.60, 0.70)."""

    def key(g):
        dist = math.hypot(g["theta"] - DEFAULT_POINT[0], g["theta_split"] - DEFAULT_POINT[1])
        return (-round(g["objective"], 12), round(dist, 12), g["theta"], g["theta_split"])

    return min(grid, key=key)


def apply_constants(theta: float, theta_split: float) -> None:
    src = LINEAGE_RS.read_text()
    src2 = re.sub(r"pub const THETA: f64 = [0-9.]+;", f"pub const THETA: f64 = {theta:.2f};", src)
    src2 = re.sub(
        r"pub const THETA_SPLIT: f64 = [0-9.]+;",
        f"pub const THETA_SPLIT: f64 = {theta_split:.2f};",
        src2,
    )
    if src2 != src:
        LINEAGE_RS.write_text(src2)
        subprocess.run(["make", "build"], cwd=ROOT, check=True)
        print("lineage.rs constants updated and rebuilt; commit before --phase test")


def phase_tune(run: str, apply: bool) -> None:
    verify_d1()
    ids = splits()["tune"]
    grid = []
    for th in THETAS:
        for ts in THETA_SPLITS:
            m = evaluate(ids, th, ts)
            row = {
                "theta": th,
                "theta_split": ts,
                **{k: m[k] for k in ("H1-S1", "H1-S2", "H1-S3", "H1-S4")},
            }
            row["objective"] = metrics.h1s_objective(m)
            grid.append(row)
            print(
                f"theta={th:.2f} split={ts:.2f} S1={m['H1-S1']:.4f} S2={m['H1-S2']:.4f} "
                f"S3={m['H1-S3']:.4f} S4={m['H1-S4']:.4f} obj={row['objective']:.4f}"
            )
    if len(grid) != len(THETAS) * len(THETA_SPLITS):
        raise RuntimeError("incomplete grid")
    keys = ("H1-S1", "H1-S2", "H1-S3", "H1-S4")
    grid_flat = len({tuple(round(g[k], 12) for k in keys) for g in grid}) < 2
    probe = None
    if grid_flat:
        # A flat grid is either an unwired THETA (v1's failure) or data whose
        # difficulty does not reach the pre-registered range. Tell them apart
        # with one out-of-grid probe; it is reported, never selectable.
        pm = evaluate(ids, PROBE[0], PROBE[1])
        probe = {"theta": PROBE[0], "theta_split": PROBE[1], **{k: pm[k] for k in keys}}
        if all(round(probe[k], 12) == round(grid[0][k], 12) for k in keys):
            raise RuntimeError(
                "metrics identical across grid and probe: THETA is not used by the matcher"
            )
        print(f"grid is flat; out-of-grid probe {PROBE} differs: {probe}")
    sel = select(grid)
    out = {
        "split": "tune",
        "tune_ids": ids,
        "grid": grid,
        "rule": "max S1 + (1-S2) + S3 + S4; ties -> closest to (0.60, 0.70)",
        "selected": {"theta": sel["theta"], "theta_split": sel["theta_split"]},
        "grid_flat": grid_flat,
        "sensitivity_probe": probe,
    }
    runinfo.write_json(run, "h1s_tuning.json", out)
    runinfo.record_phase(run, "h1s_tune", out["selected"], ["h1s_tuning.json"])
    print(f"selected THETA={sel['theta']:.2f} THETA_SPLIT={sel['theta_split']:.2f}")
    if apply:
        apply_constants(sel["theta"], sel["theta_split"])


def phase_test(run: str) -> None:
    verify_d1()
    tuning = runinfo.read_json(run, "h1s_tuning.json")
    sel = tuning["selected"]
    compiled = rootmark_core.matcher_defaults()
    if (round(compiled[0], 6), round(compiled[1], 6)) != (
        round(sel["theta"], 6),
        round(sel["theta_split"], 6),
    ):
        raise RuntimeError(f"compiled THETA/THETA_SPLIT {compiled} != tuning selection {sel}")
    s = splits()
    if set(tuning["tune_ids"]) & set(s["test"]):
        raise RuntimeError("tuning documents leak into the test split")
    m = evaluate(s["test"], None, None)
    out = {"split": "test", "theta": sel["theta"], "theta_split": sel["theta_split"], **m}
    runinfo.write_json(run, "h1s.json", out)
    runinfo.record_phase(run, "h1s_test", sel, ["h1s.json"])
    print(json.dumps({k: m[k] for k in ("H1-S1", "H1-S2", "H1-S3", "H1-S4")}, indent=1))


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--phase", choices=["tune", "test"], required=True)
    ap.add_argument("--run", required=True)
    ap.add_argument(
        "--apply", action="store_true", help="tune: write the selection into lineage.rs and rebuild"
    )
    a = ap.parse_args(argv)
    if a.phase == "tune":
        phase_tune(a.run, a.apply)
    else:
        phase_test(a.run)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
