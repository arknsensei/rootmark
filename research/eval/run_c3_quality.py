"""C-3 (GOAL P1-C; TASKS C8): retrieval quality of the Rust index against the
prototype v2-final `hybrid` numbers.

D3 (the frozen v2 vault, `make d3`, verified against `MANIFEST.d3.json`) is
copied into a temporary vault; through the binding (`rootmark_core`): one
checkpoint, `index_build` (structural chunks, ONNX MiniLM), then every D4
**test** question (`data/splits/d4.json`; single and multi) is searched at
that checkpoint in the modes `hybrid` and `flat` (k = 10, proposals excluded;
no graph strategy is run). Per query nDCG@10, recall@5, recall@10, MRR and
full_cover@10 with the frozen H2 metric code (`eval.metrics`, truth resolved
by `eval.truth` on the same vault); cell means per `<mode>/structural/<type>`;
the question count of every cell must equal the split's (fail-fast).

Reference: the v2-final RUN `20261013T154851Z-f133d0b` (`research/FROZEN.md`
"v2 nihai RUN"; ASSUMPTIONS P1-C C0-2), cells `hybrid/structural/{single,
multi}` (and `flat/…` for information). C-3 passes iff |Δ nDCG@10| ≤ 0.01
for hybrid single and hybrid multi. Writes `c3_quality.json` into the RUN.

    python -m eval.run_c3_quality --run <RUN>
"""

from __future__ import annotations

import argparse
import shutil
import tempfile
from pathlib import Path

import rootmark_core

from eval import metrics as M
from eval import runinfo
from eval import truth as T
from eval.datasets.common import ROOT, die
from eval.run_h2 import D3, TYPES, K, cell_means, expected_counts, load_split, verify_d3

REF_RUN = ROOT / "reports" / "runs" / "20261013T154851Z-f133d0b"
MODES = ["hybrid", "flat"]
CHUNKING = "structural"
THRESHOLD = 0.01
CHECKPOINT = "c3-quality"
METRICS = ("ndcg@10", "recall@5", "recall@10", "mrr", "full_cover@10")


def reference_cells() -> dict:
    import json

    h2 = json.loads((REF_RUN / "h2.json").read_text(encoding="utf-8"))
    out = {}
    for mode in MODES:
        for t in TYPES:
            key = f"{mode}/{CHUNKING}/{t}"
            if key not in h2["cells"]:
                raise RuntimeError(f"v2-final h2.json has no cell {key}")
            out[key] = h2["cells"][key]
    return out


def copy_vault(dst: Path) -> None:
    src = D3 / ".rootmark"
    if not src.is_dir():
        raise RuntimeError("data/d3 is empty; run `make d3` first")
    shutil.copytree(src, dst / ".rootmark", ignore=shutil.ignore_patterns("index", "index.build"))


def evaluate(work: Path) -> dict:
    d3 = rootmark_core.Vault(str(D3))
    verify_d3(d3)
    copy_vault(work)
    vault = rootmark_core.Vault(str(work))
    if vault.events_hash() != d3.events_hash():
        raise RuntimeError("copied vault differs from data/d3")
    cp = vault.checkpoint(CHECKPOINT)
    build = vault.index_build()
    qs = load_split("test")
    expected = expected_counts("test")
    groups = [[set(T.resolve_all(vault, g)) for g in q["truth"]] for q in qs]
    corpus = [c["lineage_ids"] for d in vault.doc_paths() for c in vault.chunk(d, CHUNKING)]
    if len(corpus) != build["records"]:
        raise RuntimeError(f"{len(corpus)} chunks at the head, index holds {build['records']}")
    cells: dict = {}
    per_query: dict = {}
    for mode in MODES:
        results = vault.search_many(
            [q["question"] for q in qs], CHECKPOINT, k=K, mode=mode, consumer="c3-quality"
        )
        rows = []
        for q, g, r in zip(qs, groups, results, strict=True):
            if r["checkpoint_id"] != cp["checkpoint_id"]:
                raise RuntimeError(f"{q['id']}: searched at {r['checkpoint_id']}")
            if len(r["hits"]) != K:
                raise RuntimeError(f"{mode}: {len(r['hits'])} results for {q['id']}")
            if any(h["layer"] != "canonical" for h in r["hits"]):
                raise RuntimeError(f"{mode}: non-canonical hit for {q['id']}")
            lins = [h["lineage_ids"] for h in r["hits"]]
            rows.append(
                {
                    "id": q["id"],
                    "type": q["type"],
                    "trace_id": r["trace_id"],
                    **M.query_metrics(lins, g, M.ideal_relevant_count(corpus, g)),
                }
            )
        per_query[f"{mode}/{CHUNKING}"] = rows
        for t in TYPES:
            sub = [x for x in rows if x["type"] == t]
            if len(sub) != expected[t]:
                raise RuntimeError(
                    f"cell {mode}/{CHUNKING}/{t}: {len(sub)} queries, expected {expected[t]}"
                )
            cells[f"{mode}/{CHUNKING}/{t}"] = {"n": len(sub), **cell_means(sub)}
    ref = reference_cells()
    delta = {key: {m: cells[key][m] - ref[key][m] for m in METRICS} for key in sorted(cells)}
    c3 = {
        t: {
            "ndcg@10": cells[f"hybrid/{CHUNKING}/{t}"]["ndcg@10"],
            "reference": ref[f"hybrid/{CHUNKING}/{t}"]["ndcg@10"],
            "delta": delta[f"hybrid/{CHUNKING}/{t}"]["ndcg@10"],
        }
        for t in TYPES
    }
    return {
        "criterion": "C-3",
        "threshold_abs_delta_ndcg@10": THRESHOLD,
        "pass": all(abs(c3[t]["delta"]) <= THRESHOLD for t in TYPES),
        "C-3": c3,
        "reference_run": REF_RUN.name,
        "checkpoint_id": cp["checkpoint_id"],
        "index": build,
        "n_questions": expected,
        "cells": cells,
        "reference_cells": ref,
        "delta": delta,
        "per_query": per_query,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    with tempfile.TemporaryDirectory(prefix="rootmark-c3q-") as tmp:
        out = evaluate(Path(tmp))
    runinfo.write_json(a.run, "c3_quality.json", out)
    runinfo.record_phase(
        a.run,
        "c3_quality",
        {"modes": MODES, "chunking": CHUNKING, "k": K, "threshold": THRESHOLD},
        ["c3_quality.json"],
    )
    for t in TYPES:
        c = out["C-3"][t]
        print(
            f"C-3 hybrid/{t}: nDCG@10 {c['ndcg@10']:.4f} (v2 {c['reference']:.4f}, Δ {c['delta']:+.4f})"
        )
    print(f"C-3 {'pass' if out['pass'] else 'FAIL'} (|Δ| ≤ {THRESHOLD})")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
