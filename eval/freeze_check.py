"""`make freeze-check`: runs every frozen metric path on a small fixed fixture
and compares the result with `tests/fixtures/freeze_fixture/expected.json`
byte for byte (CLAUDE.md §Ölçüm dürüstlüğü 2: a compatibility fix must leave
this output bit-identical).

Covered: `eval/datasets/synthetic_edits.py` (3-step generation), `eval/truth.py`
(every reference resolved through the vault), `eval/metrics.py` H1-S, H1-R, H2
(flat + hybrid on both chunk strategies, 20 hand-written questions), H3, H4.

    python -m eval.freeze_check [--write]
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path

import mdkernel_core

from eval import metrics as M
from eval import truth as T
from eval.datasets import synthetic_edits as se
from eval.datasets.common import ROOT, blake3_hex, die, dump_json, git
from eval.datasets.git_history import parse_hunks

FIX = ROOT / "tests" / "fixtures" / "freeze_fixture"
EXPECTED = FIX / "expected.json"
EDIT_SOURCES = ("roasting.md", "maintenance.md", "espresso.md", "cupping.md", "safety.md")
EDIT_STEPS = 3
MAX_EDIT_RUNS = 60


def _sets_to_lists(rec: dict) -> dict:
    out = dict(rec)
    for key in ("truth_events", "pred_events"):
        out[key] = [[k, sorted(a), sorted(b)] for k, a, b in rec[key]]
    return out


def _try(fn):
    try:
        return fn()
    except M.MetricError as e:
        return {"MetricError": str(e)}


def _h1s_records(truth: dict, out_dir: Path, vault_dir: Path) -> list[dict]:
    v = mdkernel_core.Vault(str(vault_dir))
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
            M.h1s_step_record(s["op"], list(zip(olds, news, strict=True)), prev, r["events"])
        )
    return records


def check_h1s(tmp: Path) -> dict:
    """3-step edit sequences on fixture documents, as many runs (fixed order)
    as needed until every D1 operation has occurred at least once."""
    vault_dir = FIX / "vault"
    texts = {p.name: p.read_bytes().decode("utf-8") for p in sorted(vault_dir.glob("*.md"))}
    all_ops = {name for name, _ in se.OPS}
    seen: set[str] = set()
    runs, records = [], []
    for i in range(MAX_EDIT_RUNS):
        source = EDIT_SOURCES[i % len(EDIT_SOURCES)]
        name = f"{source}#{i}"
        pool = se.insert_pool(texts, {source})
        out_dir = tmp / "d1" / f"{i:02d}"
        truth = se.generate_doc(name, texts[source], pool, out_dir, n_steps=EDIT_STEPS)
        recs = _h1s_records(truth, out_dir, tmp / "h1s" / f"{i:02d}")
        records.extend(recs)
        ops = [s["op"] for s in truth["steps"]]
        seen.update(o for o in ops if o in all_ops)
        runs.append(
            {
                "doc_path": name,
                "ops": ops,
                "files": {p.name: blake3_hex(p.read_bytes()) for p in sorted(out_dir.iterdir())},
                "records": [_sets_to_lists(r) for r in recs],
            }
        )
        if seen == all_ops:
            break
    if seen != all_ops:
        raise RuntimeError(f"edit runs did not cover all ops: missing {sorted(all_ops - seen)}")
    return {"runs": runs, "ops_covered": sorted(seen), "aggregate": M.h1s_aggregate(records)}


def check_h1r(tmp: Path) -> dict:
    pair = json.loads((FIX / "h1r_pair.json").read_text())
    diff = git(
        "diff", "--no-index", "-U0", str(FIX / pair["old"]), str(FIX / pair["new"]), ok_codes=(0, 1)
    )
    if parse_hunks(diff) != pair["hunks"]:
        raise RuntimeError("h1r_pair.json hunks differ from parse_hunks(git diff)")
    old = (FIX / pair["old"]).read_bytes().decode("utf-8")
    new = (FIX / pair["new"]).read_bytes().decode("utf-8")
    drafts, _ = mdkernel_core.parse_document(old, pair["path"])
    v = mdkernel_core.Vault(str(tmp / "h1r"))
    v.ingest(pair["path"], old)
    old_blocks = v.snapshot_blocks(pair["path"])
    for b, d in zip(old_blocks, drafts, strict=True):
        b["line_start"], b["line_end"] = d["line_start"], d["line_end"]
    v.ingest(pair["path"], new)
    rec = M.h1r_pair_record(
        old_blocks,
        pair["hunks"],
        [lin for lin, _ in v.snapshot(pair["path"])["blocks"]],
        v.snapshot_blocks(pair["path"]),
    )
    return {
        "touched_lines": sorted(M.touched_lines(pair["hunks"])),
        "record": rec,
        "aggregate": M.h1r_aggregate([rec], [pair]),
    }


def check_h2(tmp: Path) -> dict:
    from mdkernel import index as I
    from mdkernel.retrieval import Retriever

    vault_root = tmp / "h2"
    shutil.copytree(FIX / "vault", vault_root / "docs")
    v = mdkernel_core.Vault(str(vault_root))
    v.ingest_dir(str(vault_root / "docs"))
    questions = [json.loads(line) for line in (FIX / "questions.jsonl").read_text().splitlines()]
    if len(questions) != 20:
        raise RuntimeError(f"freeze fixture must hold 20 questions, found {len(questions)}")
    groups = [[set(T.resolve_all(v, g)) for g in q["truth"]] for q in questions]
    # duplicate blocks: k selects the occurrence; a missing occurrence is an error
    dup_doc = "maintenance.md"
    dup = [
        b
        for b in v.snapshot_blocks(dup_doc)
        if b["normalized_text"].startswith("Note: always check")
    ]
    dup_refs = [{"doc_path": dup_doc, "version_id": dup[0]["version_id"], "k": k} for k in (0, 1)]
    dup_lins = T.resolve_all(v, dup_refs)
    if len(dup) != 2 or dup_lins[0] == dup_lins[1] or dup_lins != [b["lineage_id"] for b in dup]:
        raise RuntimeError("duplicate-block truth resolution is wrong")
    try:
        T.resolve(v, {**dup_refs[0], "k": 2})
        raise RuntimeError("k beyond the occurrences must not resolve")
    except T.TruthError as e:
        dup_error = str(e).split(":")[0]
    out_dup = {"lineages_k0_k1_distinct": True, "k2": dup_error}
    out: dict = {}
    for strategy in ("structural", "fixed"):
        idx = I.build(v, strategy, vault_root / "lance")
        r = Retriever(idx)
        corpus = [c["lineage_ids"] for c in idx.chunks]
        for name in ("flat", "hybrid"):
            per_q = []
            for q, g in zip(questions, groups, strict=True):
                ranked = getattr(r, name)(q["question"], 10)
                lins = [idx.chunk(k)["lineage_ids"] for k, _ in ranked]
                m = M.query_metrics(lins, g, M.ideal_relevant_count(corpus, g))
                per_q.append({"id": q["id"], "keys": [k for k, _ in ranked], **m})
            cell = {"per_query": per_q}
            for t in ("single", "multi"):
                rows = [p for p, q in zip(per_q, questions, strict=True) if q["type"] == t]
                cell[t] = {
                    key: M.mean([p[key] for p in rows])
                    for key in ("ndcg@10", "recall@5", "recall@10", "mrr", "full_cover@10")
                }
            out[f"{name}/{strategy}"] = cell
    a = [p["ndcg@10"] for p in out["hybrid/structural"]["per_query"]]
    b = [p["ndcg@10"] for p in out["flat/structural"]["per_query"]]
    out["bootstrap_hybrid_minus_flat"] = M.paired_bootstrap(a, b, n=2000)
    out["duplicate_truth"] = out_dup
    return out


def check_pure() -> dict:
    """Every metric function on fixed, hand-made inputs (machine independent)."""

    def ev(kind, frm, to):
        return {"kind": kind, "meta": {"from_lineages": frm}, "lineage_ids": to}

    recs = [
        M.h1s_step_record("rephrase_light", [("L1", ["L1"])], ["L1"], []),
        M.h1s_step_record("move", [("L2", ["L9"])], ["L2"], []),
        M.h1s_step_record(
            "dup_delete", [("A", ["A"]), ("B", ["X"]), ("C", [])], ["A", "B", "C"], []
        ),
        M.h1s_step_record("insert", [(None, ["L1"])], ["L1"], []),
        M.h1s_step_record(
            "split", [("S", ["S1", "S2"])], ["S"], [ev("BlockSplit", ["S"], ["S1", "S2"])]
        ),
        M.h1s_step_record(
            "merge", [("P", ["M"]), ("Q", ["M"])], ["P", "Q"], [ev("BlockMerged", ["P"], ["M"])]
        ),
        M.h1s_step_record(
            "rephrase_heavy", [("R", ["R"])], ["R"], [ev("BlockSplit", ["R"], ["R", "Z"])]
        ),
    ]
    h1s = M.h1s_aggregate(recs)
    groups = [{"a1", "a2"}, {"b1"}]
    ranked = [["x"], ["a1"], ["b1"], ["a2", "z"], ["q"]]
    corpus = ranked + [["a1", "b1"]]
    return {
        "h1s": {**h1s, "objective": M.h1s_objective(h1s)},
        "h2_query": M.query_metrics(ranked, groups, M.ideal_relevant_count(corpus, groups)),
        "h2_single": M.query_metrics([["x"], ["a1"]], [{"a1", "a2"}], 1),
        "bootstrap": M.paired_bootstrap([0.9, 0.4, 0.7, 0.1], [0.5, 0.5, 0.2, 0.3], n=1000),
        "touched": sorted(
            M.touched_lines([{"old": [5, 2], "new": [5, 3]}, {"old": [20, 0], "new": [21, 1]}])
        ),
        "h2_multi_only_a": M.query_metrics([["a1"], ["a2"], ["x"]], groups, 2),
        "h1r": _pure_h1r(),
        "h4": _pure_h4(),
    }


def _pure_h1r() -> dict:
    def blk(lin, vid, a, b, typ="Paragraph", text=None):
        return {
            "lineage_id": lin,
            "version_id": vid,
            "block_type": typ,
            "normalized_text": text or vid,
            "line_start": a,
            "line_end": b,
        }

    old = [
        blk("L1", "v1", 1, 1),
        blk("L2", "v2", 3, 3),
        blk("L3", "v3", 5, 6),
        blk("L4", "dup", 9, 9),
        blk("L5", "dup", 11, 11),
        blk("L6", "v6", 13, 13, text="x"),
        blk("L7", "v7", 20, 20),
    ]
    new = [{"version_id": "v6b", "block_type": "ListItem", "normalized_text": "x"}]
    r1 = M.h1r_pair_record(old, [{"old": [6, 1], "new": [6, 1]}], ["L1", "L5"], new)
    many = [blk(f"O{i}", f"o{i}", 100 + 10 * i, 100 + 10 * i) for i in range(6)]
    many.append(blk("T", "t", 300, 300))
    r2 = M.h1r_pair_record(many, [{"old": [300, 0], "new": [301, 1]}], [], [])
    pairs = [{"commit": "c1", "path": "a.md"}, {"commit": "c2", "path": "b.md"}]
    return {
        "records": [r1, r2],
        "aggregate": M.h1r_aggregate([r1, r2], pairs),
        "aggregate_few_other": M.h1r_aggregate([r1], pairs[:1]),
    }


def _pure_h4() -> dict:
    cases = {
        "int_vs_bool": ({"x": 1}, {"x": True}),
        "int_vs_str": ({"x": 1}, {"x": "1"}),
        "int_vs_float": ({"x": 2}, {"x": 2.0}),
        "nan": ({"x": float("nan")}, {"x": 1.0}),
        "inf": ({"x": float("inf")}, {"x": float("inf")}),
        "missing": ({"x": 1}, {"y": 1}),
        "list_equal": ({"x": [1, 2]}, {"x": [1, 2]}),
        "list_differs": ({"x": [1, 2]}, {"x": [2, 1]}),
    }
    return {
        k: _try(lambda a=a, b=b: M.max_relative_difference(a, b, ["x"]))
        for k, (a, b) in cases.items()
    }


def check_h3_h4() -> dict:
    lat = [float(x) for x in range(1, 41)]
    a = {"h1s": {"H1-S1": 0.97, "n": 3}, "h2": {"x": 0.5, "label": "s"}}
    b = {"h1s": {"H1-S1": 0.97, "n": 3}, "h2": {"x": 0.25, "label": "s"}}
    return {
        "latency": M.latency_summary(lat),
        "h3": M.h3_summary(
            lat,
            **{
                k: v
                for k, v in M.lineage_loss(["a", "b"], ["b", "c"]).items()
                if k in ("lost", "gained")
            },
            log_equal=True,
        ),
        "h4": M.max_relative_difference(a, b, ["h1s.H1-S1", "h1s.n", "h2.x", "h2.label"]),
    }


def run() -> dict:
    with tempfile.TemporaryDirectory() as t:
        tmp = Path(t)
        return {
            "h1s": check_h1s(tmp),
            "h1r": check_h1r(tmp),
            "h2": check_h2(tmp),
            "h3_h4": check_h3_h4(),
            "pure": check_pure(),
        }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="(re)write expected.json — only in T9")
    a = ap.parse_args(argv)
    got = dump_json(run())
    if a.write:
        EXPECTED.write_text(got, encoding="utf-8")
        print(f"wrote {EXPECTED.relative_to(ROOT)} ({blake3_hex(got.encode())})")
        return
    if not EXPECTED.exists():
        die(f"{EXPECTED} missing")
    want = EXPECTED.read_text(encoding="utf-8")
    if got != want:
        for i, (x, y) in enumerate(zip(got.splitlines(), want.splitlines(), strict=False)):
            if x != y:
                print(
                    f"first difference at line {i + 1}:\n  got:      {x}\n  expected: {y}",
                    file=sys.stderr,
                )
                break
        die("freeze-check FAILED: output differs from expected.json")
    print(f"freeze-check OK ({blake3_hex(got.encode())})")


if __name__ == "__main__":
    main()
