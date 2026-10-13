"""`make freeze-check-v3`: runs every v3 metric path on a small fixed fixture
with matcher v3 and compares the result with
`tests/fixtures/freeze_fixture_v3/expected.json` byte for byte (a
compatibility fix after `T23: freeze eval v3` must leave this output
bit-identical; CLAUDE.md v3 rules).

Covered: `eval/datasets/synthetic_edits_v2.py` (K4 truth of the v2 generator
and every D1-hard operation, 3-step sequences), `eval/truth.py` resolution,
`eval/metrics_v3.py` (step records, easy and hard aggregates, objective, H1-G,
the v2-defined S2, the imported H1-R functions) on fixture data and on fixed
hand-made inputs.

    python -m eval.freeze_check_v3 [--write]
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path

import mdkernel_core

from eval import metrics_v3 as M3
from eval import truth as T
from eval.datasets import synthetic_edits as se
from eval.datasets import synthetic_edits_v2 as se2
from eval.datasets.common import ROOT, blake3_hex, die, dump_json

FIX = ROOT / "tests" / "fixtures" / "freeze_fixture_v3"
EXPECTED = FIX / "expected.json"
# Matcher version, the THETA / THETA_SPLIT of every non-grid ingest and the
# grid are carried by the fixture (params.json), not by this module or the
# compiled constants (P1-A FREEZE_EXCEPTIONS F1)
_P = json.loads((FIX / "params.json").read_text(encoding="utf-8"))
MATCHER = _P["matcher"]
EDIT_SOURCES = ("roasting.md", "maintenance.md", "espresso.md", "cupping.md", "safety.md")
EDIT_STEPS = 3
MAX_EDIT_RUNS = 80
GRID = tuple(tuple(c) for c in _P["grid"])
# THETA / THETA_SPLIT of every non-grid ingest: the compiled V3 constants at
# freeze time (T23), pinned in the fixture so that tuning cannot change this
# output (prototype FREEZE_EXCEPTIONS v3/T24; P1-A F1 moved them to params.json)
PARAMS = tuple(_P["params"])


def _rec_lists(rec: dict) -> dict:
    out = dict(rec)
    for key in ("truth_events", "pred_events"):
        out[key] = [[k, sorted(a), sorted(b), d] for k, a, b, d in rec[key]]
    return out


def _try(fn):
    try:
        return fn()
    except M3.MetricError as e:
        return {"MetricError": str(e)}


def _dominant_index(step: dict) -> int | None:
    if "dominant" not in step:
        return None
    if step["op"] in M3.SPLIT_OPS:
        cands = step["mapping"][0][1]
    else:
        cands = [o for o, _ in step["mapping"]]
    (i,) = [k for k, r in enumerate(cands) if r == step["dominant"]]
    return i


def _records(truth: dict, out_dir: Path, vault_dir: Path, params=PARAMS) -> list[dict]:
    v = mdkernel_core.Vault(str(vault_dir), params[0], params[1], matcher=MATCHER)
    doc = truth["doc_path"]
    v.ingest(doc, (out_dir / truth["base_file"]).read_text(encoding="utf-8"), "Synthetic", "d1")
    records = []
    for s in truth["steps"]:
        text = (out_dir / s["file"]).read_text(encoding="utf-8")
        if not s["scored"]:
            v.ingest(doc, text, "Synthetic", "d1")
            continue
        prev = [lin for lin, _ in v.snapshot(doc)["blocks"]]
        olds = [None if o is None else T.resolve(v, o, doc) for o, _ in s["mapping"]]
        r = v.ingest(doc, text, "Synthetic", "d1")
        news = [[T.resolve(v, n, doc) for n in ns] for _, ns in s["mapping"]]
        records.append(
            M3.step_record(
                s["op"], list(zip(olds, news, strict=True)), _dominant_index(s), prev, r["events"]
            )
        )
    return records


def _texts() -> dict[str, str]:
    return {p.name: p.read_bytes().decode("utf-8") for p in sorted((FIX / "vault").glob("*.md"))}


def _run_entry(name, truth, out_dir, recs) -> dict:
    return {
        "doc_path": name,
        "ops": [s["op"] for s in truth["steps"]],
        "dominant": [s.get("dominant") for s in truth["steps"]],
        "files": {p.name: blake3_hex(p.read_bytes()) for p in sorted(out_dir.iterdir())},
        "records": [_rec_lists(r) for r in recs],
    }


def check_easy(tmp: Path) -> dict:
    """v2 generator + K4 truth (as `build_d1k4`) until split and merge occurred."""
    texts = _texts()
    runs, records, seen = [], [], set()
    for i in range(MAX_EDIT_RUNS):
        source = EDIT_SOURCES[i % len(EDIT_SOURCES)]
        name = f"{source}#{i}"
        out_dir = tmp / "easy" / f"{i:02d}"
        se.generate_doc(name, texts[source], se.insert_pool(texts, {source}), out_dir, EDIT_STEPS)
        truth = se2.k4_truth(out_dir, json.loads((out_dir / "truth.json").read_text()))
        recs = _records(truth, out_dir, tmp / "easy_v" / f"{i:02d}")
        records.extend(recs)
        seen.update(s["op"] for s in truth["steps"] if s["scored"])
        runs.append(_run_entry(name, truth, out_dir, recs))
        if seen >= {op for op, _ in se.OPS}:
            break
    else:
        raise RuntimeError(f"easy runs did not cover all ops: {sorted(seen)}")
    return {"runs": runs, "aggregate": _try(lambda: M3.aggregate(records, "easy"))}


def check_hard(tmp: Path) -> dict:
    """D1-hard generator until every hard op occurred; a 3-point grid on it."""
    texts = _texts()
    runs, seqs, records, seen = [], [], [], set()
    all_ops = {op for op, _ in se2.OPS_HARD}
    for i in range(MAX_EDIT_RUNS):
        source = EDIT_SOURCES[i % len(EDIT_SOURCES)]
        name = f"{source}#{i}"
        out_dir = tmp / "hard" / f"{i:02d}"
        truth = se2.generate_hard_doc(
            name, texts[source], se.insert_pool(texts, {source}), out_dir, EDIT_STEPS
        )
        recs = _records(truth, out_dir, tmp / "hard_v" / f"{i:02d}")
        records.extend(recs)
        seqs.append((truth, out_dir))
        seen.update(s["op"] for s in truth["steps"] if s["scored"])
        runs.append(_run_entry(name, truth, out_dir, recs))
        if seen >= all_ops:
            break
    else:
        raise RuntimeError(f"hard runs did not cover all ops: missing {sorted(all_ops - seen)}")
    grid = []
    for k, (th, ts) in enumerate(GRID):
        recs = [
            r
            for j, (t, d) in enumerate(seqs)
            for r in _records(t, d, tmp / "grid" / f"{k}-{j:02d}", (th, ts))
        ]
        m = _try(lambda recs=recs: M3.aggregate(recs, "hard"))
        obj = None if "MetricError" in m else M3.objective(m, "hard")
        grid.append({"theta": th, "theta_split": ts, "objective": obj})
    objs = [g["objective"] for g in grid if g["objective"] is not None]
    return {
        "runs": runs,
        "aggregate": _try(lambda: M3.aggregate(records, "hard")),
        "grid": grid,
        "H1-G": _try(lambda: M3.distinct_objectives(objs)),
    }


def check_h1r(tmp: Path) -> dict:
    pair = json.loads((FIX / "h1r_pair.json").read_text())
    old = (FIX / pair["old"]).read_bytes().decode("utf-8")
    new = (FIX / pair["new"]).read_bytes().decode("utf-8")
    drafts, _ = mdkernel_core.parse_document(old, pair["path"])
    v = mdkernel_core.Vault(str(tmp / "h1r"), PARAMS[0], PARAMS[1], matcher=MATCHER)
    v.ingest(pair["path"], old)
    old_blocks = v.snapshot_blocks(pair["path"])
    for b, d in zip(old_blocks, drafts, strict=True):
        b["line_start"], b["line_end"] = d["line_start"], d["line_end"]
    v.ingest(pair["path"], new)
    rec = M3.h1r_pair_record(
        old_blocks,
        pair["hunks"],
        [lin for lin, _ in v.snapshot(pair["path"])["blocks"]],
        v.snapshot_blocks(pair["path"]),
    )
    return {"record": rec, "aggregate": M3.h1r_aggregate([rec], [pair])}


def check_pure() -> dict:
    """metrics_v3 on fixed, hand-made inputs (machine independent)."""

    def ev(kind, frm, to, dom=None):
        meta = {"from_lineages": frm}
        if dom is not None:
            meta["dominant"] = dom
        return {"kind": kind, "meta": meta, "lineage_ids": to}

    sr = M3.step_record
    recs = [
        sr("rephrase_light", [("L1", ["L1"])], None, ["L1"], []),
        sr("rephrase_extreme", [("L3", ["L4"])], None, ["L3"], []),
        sr("move_edit", [("L2", ["L2"])], None, ["L2"], []),
        sr(
            "dup_delete_adjacent",
            [("A", ["A"]), ("C", ["X"]), ("B", [])],
            None,
            ["A", "B", "C"],
            [],
        ),
        sr("insert", [(None, ["L1"])], None, ["L1"], []),
        # correct K4 split: dominant part (index 1) keeps S
        sr(
            "split_edit",
            [("S", ["N1", "S"])],
            1,
            ["S"],
            [ev("BlockSplit", ["S"], ["N1", "S"], "S")],
        ),
        # wrong dominant: the shorter part kept S → retention 0, false match 1, event wrong
        sr(
            "split_three",
            [("T", ["T", "N2", "N3"])],
            1,
            ["T"],
            [ev("BlockSplit", ["T"], ["T", "N2", "N3"], "T")],
        ),
        # correct K4 merge: result keeps Q (dominant index 1); not in the S2 denominator
        sr(
            "merge_edit",
            [("P", ["Q"]), ("Q", ["Q"])],
            1,
            ["P", "Q"],
            [ev("BlockMerged", ["P", "Q"], ["Q"], "Q")],
        ),
        # merge with dominant mismatch in the event (v2-style event, no dominant)
        sr(
            "merge",
            [("U", ["M"]), ("W", ["M"])],
            0,
            ["U", "W"],
            [ev("BlockMerged", ["U", "W"], ["M"])],
        ),
        sr(
            "rephrase_heavy",
            [("R", ["R"])],
            None,
            ["R"],
            [ev("BlockSplit", ["R"], ["R", "Z"], "R")],
        ),
    ]
    out = {"records": [_rec_lists(r) for r in recs]}
    for which in ("easy", "hard"):
        m = _try(lambda which=which: M3.aggregate(recs, which))
        out[which] = m
        if "MetricError" not in m:
            out[f"{which}_objective"] = M3.objective(m, which)
    out["H1-G"] = M3.distinct_objectives([3.1, 3.1000001, 3.2, 2.0, 3.2000004])
    out["errors"] = {
        "dominant_missing": _try(lambda: sr("split", [("S", ["A", "B"])], None, ["S"], [])),
        "dominant_unexpected": _try(lambda: sr("move", [("S", ["S"])], 0, ["S"], [])),
        "unknown_op": _try(lambda: sr("frobnicate", [], None, [], [])),
        "empty_grid": _try(lambda: M3.distinct_objectives([])),
    }
    return out


def run() -> dict:
    with tempfile.TemporaryDirectory() as t:
        tmp = Path(t)
        return {
            "matcher": MATCHER,
            # key kept for bit-identity with expected.json: the pinned PARAMS
            # (equal to the compiled V3 defaults when the fixture was frozen)
            "matcher_defaults": list(PARAMS),
            "easy": check_easy(tmp),
            "hard": check_hard(tmp),
            "h1r": check_h1r(tmp),
            "pure": check_pure(),
        }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="(re)write expected.json")
    a = ap.parse_args(argv)
    text = dump_json(run())
    digest = blake3_hex(text.encode("utf-8"))
    if a.write:
        EXPECTED.write_text(text, encoding="utf-8")
        print(f"wrote {EXPECTED.relative_to(ROOT)} ({digest})")
        return
    if not EXPECTED.exists():
        die(f"{EXPECTED} missing; run with --write once")
    if EXPECTED.read_text(encoding="utf-8") != text:
        print(
            f"freeze-check-v3 FAILED: output {digest} differs from expected.json", file=sys.stderr
        )
        sys.exit(1)
    print(f"freeze-check-v3 OK ({digest})")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
