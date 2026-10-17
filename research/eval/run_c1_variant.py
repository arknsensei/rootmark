"""C1-1 (GOAL P1-C.1; K-C1.2): does the BM25 variant alone explain the C-3
difference between the Rust index and the prototype?

The prototype pipeline is rerun on D3 (`make d3`) with the frozen pieces —
structural chunks, `sentence-transformers` vectors and query embedding
(`eval.build_index.load_or_build`, `rootmark.embed`), tokens
(`rootmark.index.bm25_tokens`), RRF `1/(60 + rank)` of the top 50 of each
side, k 10, the D4 test split, the frozen H2 metrics — with two lexical
scorers:

- `lucene`: `eval.bm25_lucene` (K-C1.1, the variant the Rust index records)
- `okapi`: the prototype's own `rank_bm25.BM25Okapi` (information: it must
  reproduce the v2-final cells the prototype measured)

Rankings are total: score descending, then `(doc_path, chunk_id)`; the dense
score is the cosine of the unit vectors summed left to right in float64.

C1-1 passes iff, in each of the four cells `{hybrid, flat}/structural/
{single, multi}`, nDCG@10, recall@10 and MRR of `lucene` are within 0.001
of the Rust cells of the P1-C official RUN (`c3_quality.json`,
`20261015T014417Z-9aa326b`). Per-query agreement with the Rust rows is
reported for information. Writes `c1_variant.json` into the RUN.

    python -m eval.run_c1_variant --run <RUN>
"""

from __future__ import annotations

import argparse
import json

import numpy as np

from eval import bm25_lucene as L
from eval import metrics as M
from eval import runinfo
from eval import truth as T
from eval.build_index import load_or_build
from eval.datasets.common import ROOT, die
from eval.run_h2 import TYPES, K, cell_means, expected_counts, load_split, verify_d3
from rootmark import embed as E
from rootmark.index import bm25_tokens

RUST_RUN = ROOT.parent / "reports" / "rounds" / "P1-C" / "runs" / "20261015T014417Z-9aa326b"
V2_RUN = ROOT / "reports" / "runs" / "20261013T154851Z-f133d0b"
CHUNKING = "structural"
MODES = ["hybrid", "flat"]
SCORERS = ["lucene", "okapi"]
RRF_K = 60
CANDIDATES = 50
THRESHOLD = 0.001
CRITERION_METRICS = ("ndcg@10", "recall@10", "mrr")
METRICS = ("ndcg@10", "recall@5", "recall@10", "mrr", "full_cover@10")


def ordered(pairs: list[tuple[int, float]], tie: list[tuple[str, str]]) -> list[tuple[int, float]]:
    return sorted(pairs, key=lambda p: (-p[1], tie[p[0]]))


def rrf(*lists: list[tuple[int, float]]) -> list[tuple[int, float]]:
    out: dict[int, float] = {}
    for lst in lists:
        for r, (i, _) in enumerate(lst, start=1):
            out[i] = out.get(i, 0.0) + 1.0 / (RRF_K + r)
    return list(out.items())


def evaluate() -> dict:
    vault, idx, _ = load_or_build(CHUNKING)
    verify_d3(vault)
    chunks = idx.chunks
    tie = [(c["doc_path"], c["chunk_id"]) for c in chunks]
    if len(set(tie)) != len(tie):
        raise RuntimeError("(doc_path, chunk_id) is not unique")
    rust = json.loads((RUST_RUN / "c3_quality.json").read_text(encoding="utf-8"))
    if rust["index"]["records"] != len(chunks):
        raise RuntimeError(f"{len(chunks)} prototype chunks, Rust index {rust['index']['records']}")
    vecs = np.asarray(idx.vectors, dtype=np.float64)
    toks = [bm25_tokens(c["text"]) for c in chunks]
    lucene = L.LuceneBM25(toks)
    okapi = idx.bm25  # the prototype's BM25Okapi over the same tokens
    qs = load_split("test")
    expected = expected_counts("test")
    groups = [[set(T.resolve_all(vault, g)) for g in q["truth"]] for q in qs]
    corpus = [c["lineage_ids"] for c in chunks]
    rows: dict = {f"{s}/{m}": [] for s in SCORERS for m in MODES}
    for q, g in zip(qs, groups, strict=True):
        # one query at a time, as the prototype (`Retriever.hybrid`)
        v = E.embed_query(q["question"]).astype(np.float64)
        dense_all = np.cumsum(vecs * v, axis=1)[:, -1]
        dense = ordered(list(enumerate(dense_all.tolist())), tie)
        qt = bm25_tokens(q["question"])
        sparse = {}
        s, hit = lucene.scores(qt)
        sparse["lucene"] = ordered([(i, s[i]) for i in range(len(s)) if hit[i]], tie)
        o = okapi.get_scores(qt)
        sparse["okapi"] = ordered([(i, float(o[i])) for i in range(len(o)) if o[i] > 0], tie)
        ideal = M.ideal_relevant_count(corpus, g)
        for sc in SCORERS:
            for mode in MODES:
                if mode == "flat":
                    top = dense[:K]
                else:
                    top = ordered(rrf(dense[:CANDIDATES], sparse[sc][:CANDIDATES]), tie)[:K]
                if len(top) != K:
                    raise RuntimeError(f"{sc}/{mode}: {len(top)} results for {q['id']}")
                lins = [chunks[i]["lineage_ids"] for i, _ in top]
                rows[f"{sc}/{mode}"].append(
                    {"id": q["id"], "type": q["type"], **M.query_metrics(lins, g, ideal)}
                )
    cells: dict = {}
    for key, rs in rows.items():
        sc, mode = key.split("/")
        for t in TYPES:
            sub = [r for r in rs if r["type"] == t]
            if len(sub) != expected[t]:
                raise RuntimeError(f"{key}/{t}: {len(sub)} queries, expected {expected[t]}")
            cells[f"{sc}/{mode}/{CHUNKING}/{t}"] = {"n": len(sub), **cell_means(sub)}
    h2 = json.loads((V2_RUN / "h2.json").read_text(encoding="utf-8"))["cells"]
    c11, agree = {}, {}
    for mode in MODES:
        rust_rows = {r["id"]: r for r in rust["per_query"][f"{mode}/{CHUNKING}"]}
        mine = rows[f"lucene/{mode}"]
        agree[mode] = {m: sum(r[m] == rust_rows[r["id"]][m] for r in mine) for m in METRICS} | {
            "n": len(mine)
        }
        for t in TYPES:
            key = f"{mode}/{CHUNKING}/{t}"
            ref = rust["cells"][key]
            got = cells[f"lucene/{key}"]
            d = {m: got[m] - ref[m] for m in METRICS}
            c11[key] = {
                "python_lucene": {m: got[m] for m in METRICS},
                "rust": {m: ref[m] for m in METRICS},
                "delta": d,
                "pass": all(abs(d[m]) <= THRESHOLD for m in CRITERION_METRICS),
            }
    okapi_vs_v2 = {
        f"{mode}/{CHUNKING}/{t}": {
            m: cells[f"okapi/{mode}/{CHUNKING}/{t}"][m] - h2[f"{mode}/{CHUNKING}/{t}"][m]
            for m in METRICS
        }
        for mode in MODES
        for t in TYPES
    }
    return {
        "criterion": "C1-1",
        "threshold_abs_delta": THRESHOLD,
        "criterion_metrics": list(CRITERION_METRICS),
        "variant": {"variant": L.VARIANT, "idf": L.IDF, "k1": L.K1, "b": L.B},
        "rust_run": RUST_RUN.name,
        "v2_run": V2_RUN.name,
        "chunks": len(chunks),
        "n_questions": expected,
        "C1-1": c11,
        "pass": all(c["pass"] for c in c11.values()),
        "per_query_equal_to_rust": agree,
        "okapi_delta_vs_v2": okapi_vs_v2,
        "cells": cells,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    out = evaluate()
    runinfo.write_json(a.run, "c1_variant.json", out)
    runinfo.record_phase(
        a.run,
        "c1_variant",
        {"modes": MODES, "scorers": SCORERS, "k": K, "threshold": THRESHOLD},
        ["c1_variant.json"],
    )
    for key, c in out["C1-1"].items():
        d = c["delta"]
        print(
            f"C1-1 {key}: Δ nDCG@10 {d['ndcg@10']:+.6f}, recall@10 {d['recall@10']:+.6f}, "
            f"MRR {d['mrr']:+.6f} {'ok' if c['pass'] else 'FAIL'}"
        )
    print(f"C1-1 {'pass' if out['pass'] else 'FAIL'} (|Δ| ≤ {THRESHOLD})")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
