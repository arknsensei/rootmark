"""H1-S v3 (EVAL_DELTA §Koşturma), matcher v3, K4 truth.

`--phase tune`: THETA × THETA_SPLIT grid on the D1-hard tuning split; H1-G
(distinct objective values); selection by SPEC_DELTA §5.3 (max objective,
ties → closest to (0.60, 0.65)); `--apply` writes `THETA_V3`/`THETA_SPLIT_V3`
into `core/src/lineage.rs` and rebuilds. Output `h1s_v3_tuning.json`.

`--phase test`: the compiled V3 constants must equal the tuning selection;
D1-easy (K4) test split → H1-S1..S4, D1-hard test split → H1-H1..H4, both
with the v2-defined S2 as information. Output `h1s_v3.json`.

    python -m eval.run_h1s_v3 --phase tune --run <RUN> [--apply]
    python -m eval.run_h1s_v3 --phase test --run <RUN>
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

from eval import metrics_v3 as M3
from eval import runinfo
from eval import truth as T
from eval.datasets.common import DATA, ROOT, die, file_blake3, load_manifest

MATCHER = "v3"
SETS = {
    "easy": (DATA / "d1k4", DATA / "MANIFEST.d1k4.json"),
    "hard": (DATA / "d1h", DATA / "MANIFEST.d1h.json"),
}
THETAS = [0.50, 0.55, 0.60, 0.65, 0.70]
THETA_SPLITS = [0.55, 0.65, 0.75]
DEFAULT_POINT = (0.60, 0.65)
LINEAGE_RS = ROOT / "core" / "src" / "lineage.rs"
SPLIT_MERGE = M3.SPLIT_OPS + M3.MERGE_OPS


def verify_set(which: str) -> dict:
    """Fail-fast: the files on disk must match the set's manifest."""
    root, manifest = SETS[which]
    m = load_manifest(manifest)
    on_disk = {
        p.relative_to(root).as_posix(): file_blake3(p)
        for p in sorted(root.rglob("*"))
        if p.is_file()
    }
    if on_disk != m["files"]:
        bad = sorted(set(on_disk.items()) ^ set(m["files"].items()))[:5]
        raise RuntimeError(f"{root.name} does not match {manifest.name} (e.g. {bad})")
    return m


def splits(which: str) -> dict:
    s = json.loads((SETS[which][0] / "splits.json").read_text())
    if set(s["tune"]) & set(s["test"]):
        raise RuntimeError(f"{which}: tune/test splits intersect")
    return s


def dominant_index(step: dict) -> int | None:
    if step["op"] not in SPLIT_MERGE:
        if "dominant" in step:
            raise RuntimeError(f"step {step['step']}: unexpected dominant for {step['op']}")
        return None
    d = step["dominant"]
    if step["op"] in M3.SPLIT_OPS:
        ((_o, news),) = step["mapping"]
        cands = news
    else:
        cands = [o for o, _ in step["mapping"]]
    hits = [i for i, r in enumerate(cands) if r == d]
    if len(hits) != 1:
        raise RuntimeError(f"step {step['step']}: dominant {d} not in the mapping")
    return hits[0]


def doc_records(doc_dir: Path, theta: float | None, theta_split: float | None) -> list[dict]:
    t = json.loads((doc_dir / "truth.json").read_text())
    doc = t["doc_path"]
    records = []
    with tempfile.TemporaryDirectory() as tmp:
        v = rootmark_core.Vault(tmp, theta, theta_split, matcher=MATCHER)
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
                M3.step_record(
                    s["op"],
                    list(zip(olds, news, strict=True)),
                    dominant_index(s),
                    prev,
                    r["events"],
                )
            )
    expected = sum(1 for s in t["steps"] if s["scored"])
    if len(records) != expected:
        raise RuntimeError(f"{doc_dir.name}: {len(records)} scored steps, expected {expected}")
    return records


def evaluate(which: str, ids: list[str], theta: float | None, theta_split: float | None) -> dict:
    if not ids:
        raise RuntimeError("no documents to evaluate")
    root = SETS[which][0]
    recs = [r for i in ids for r in doc_records(root / "docs" / i, theta, theta_split)]
    out = M3.aggregate(recs, which)
    out["n_docs"] = len(ids)
    out["n_steps"] = len(recs)
    return out


def select(grid: list[dict]) -> dict:
    """Max objective; ties → closest to (0.60, 0.65)."""

    def key(g):
        dist = math.hypot(g["theta"] - DEFAULT_POINT[0], g["theta_split"] - DEFAULT_POINT[1])
        return (-round(g["objective"], 12), round(dist, 12), g["theta"], g["theta_split"])

    return min(grid, key=key)


def apply_constants(theta: float, theta_split: float) -> None:
    src = LINEAGE_RS.read_text()
    src2 = re.sub(
        r"pub const THETA_V3: f64 = [0-9.]+;", f"pub const THETA_V3: f64 = {theta:.2f};", src
    )
    src2 = re.sub(
        r"pub const THETA_SPLIT_V3: f64 = [0-9.]+;",
        f"pub const THETA_SPLIT_V3: f64 = {theta_split:.2f};",
        src2,
    )
    if src2 != src:
        LINEAGE_RS.write_text(src2)
        subprocess.run(["make", "build"], cwd=ROOT, check=True)
        print("lineage.rs V3 constants updated and rebuilt; commit before --phase test")


def phase_tune(run: str, apply: bool) -> None:
    verify_set("hard")
    ids = splits("hard")["tune"]
    codes = M3.CODES["hard"]
    grid = []
    for th in THETAS:
        for ts in THETA_SPLITS:
            m = evaluate("hard", ids, th, ts)
            row = {"theta": th, "theta_split": ts, **{k: m[k] for k in codes}}
            row["objective"] = M3.objective(m, "hard")
            grid.append(row)
            print(
                f"theta={th:.2f} split={ts:.2f} "
                + " ".join(f"{k}={m[k]:.4f}" for k in codes)
                + f" obj={row['objective']:.4f}"
            )
    if len(grid) != len(THETAS) * len(THETA_SPLITS):
        raise RuntimeError("incomplete grid")
    g = M3.distinct_objectives(r["objective"] for r in grid)
    sel = select(grid)
    out = {
        "set": "d1h",
        "split": "tune",
        "matcher": MATCHER,
        "tune_ids": ids,
        "grid": grid,
        "rule": "max S1 + (1-S2) + S3 + S4 (H1-H*); ties -> closest to (0.60, 0.65)",
        "selected": {"theta": sel["theta"], "theta_split": sel["theta_split"]},
        "H1-G": g,
    }
    runinfo.write_json(run, "h1s_v3_tuning.json", out)
    runinfo.record_phase(run, "h1s_v3_tune", out["selected"], ["h1s_v3_tuning.json"])
    print(f"H1-G={g} selected THETA_V3={sel['theta']:.2f} THETA_SPLIT_V3={sel['theta_split']:.2f}")
    if apply:
        apply_constants(sel["theta"], sel["theta_split"])


def check_compiled(sel: dict) -> None:
    compiled = rootmark_core.matcher_defaults(MATCHER)
    if (round(compiled[0], 6), round(compiled[1], 6)) != (
        round(sel["theta"], 6),
        round(sel["theta_split"], 6),
    ):
        raise RuntimeError(f"compiled V3 THETA/THETA_SPLIT {compiled} != tuning selection {sel}")


def phase_test(run: str) -> None:
    tuning = runinfo.read_json(run, "h1s_v3_tuning.json")
    sel = tuning["selected"]
    check_compiled(sel)
    verify_set("easy")
    verify_set("hard")
    hard = splits("hard")
    if set(tuning["tune_ids"]) & set(hard["test"]):
        raise RuntimeError("tuning documents leak into the D1-hard test split")
    easy_m = evaluate("easy", splits("easy")["test"], None, None)
    hard_m = evaluate("hard", hard["test"], None, None)
    out = {
        "matcher": MATCHER,
        "theta": sel["theta"],
        "theta_split": sel["theta_split"],
        **{k: easy_m[k] for k in M3.CODES["easy"]},
        **{k: hard_m[k] for k in M3.CODES["hard"]},
        "H1-G": tuning["H1-G"],
        "easy": {"set": "d1k4", "split": "test", **easy_m},
        "hard": {"set": "d1h", "split": "test", **hard_m},
    }
    runinfo.write_json(run, "h1s_v3.json", out)
    runinfo.record_phase(run, "h1s_v3_test", sel, ["h1s_v3.json"])
    keys = (*M3.CODES["easy"], *M3.CODES["hard"], "H1-G")
    print(json.dumps({k: out[k] for k in keys}, indent=1))


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--phase", choices=["tune", "test"], required=True)
    ap.add_argument("--run", required=True)
    ap.add_argument(
        "--apply",
        action="store_true",
        help="tune: write the selection into lineage.rs (V3 constants) and rebuild",
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
