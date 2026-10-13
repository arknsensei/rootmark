"""H2 on D3 + D4.

`--phase tune`: ALPHA grid for `graph_hybrid` on the structural index, tuning
questions only; writes `h2_tuning.json` (sensitivity curve, selection) and with
`--apply` writes the selection into `mdkernel/graph_search.py::ALPHA_DEFAULT`.
`--phase test`: refuses to run unless `ALPHA_DEFAULT` equals the tuning
selection; then 5 strategies × 2 chunk strategies × {single, multi} on the test
questions, latency, paired bootstrap; writes `h2.json`.

Selection rule (fixed before any tuning run): maximize the mean nDCG@10 over
all tuning questions (single and multi pooled); ties → ALPHA closest to 0.6,
then the smaller ALPHA.

    python -m eval.run_h2 --phase tune --run <RUN> [--apply]
    python -m eval.run_h2 --phase test --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import re
import time

import mdkernel_core

from eval import metrics as M
from eval import runinfo
from eval import truth as T
from eval.build_index import load_or_build
from eval.datasets.common import DATA, ROOT, SEED, die, load_manifest
from mdkernel import graph_search as GS

ALPHAS = [0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]
STRATEGIES = ["flat", "hybrid", "graph_flat", "graph_hybrid", "hier"]
CHUNKINGS = ["structural", "fixed"]
TYPES = ["single", "multi"]
K = 10
WARMUP = 10
BOOTSTRAP = 10_000
D3 = DATA / "d3"
D4 = DATA / "d4"
SPLITS = DATA / "splits" / "d4.json"
GRAPH_SEARCH = ROOT / "mdkernel" / "graph_search.py"


def verify_d3(vault) -> None:
    want = load_manifest(DATA / "MANIFEST.d3.json")["events_hash"]
    if vault.events_hash() != want:
        raise RuntimeError(f"data/d3 events_hash {vault.events_hash()} != MANIFEST.d3.json {want}")


def load_split(split: str) -> list[dict]:
    """Questions of one split only; the other split's records are discarded unread."""
    ids = set(json.loads(SPLITS.read_text())[split])
    other = set(json.loads(SPLITS.read_text())["tune" if split == "test" else "test"])
    if ids & other:
        raise RuntimeError("D4 tune/test splits intersect")
    out = []
    for kind in TYPES:
        for line in (D4 / f"{kind}.jsonl").read_text().splitlines():
            q = json.loads(line)
            if q["id"] in ids:
                out.append(q)
    if {q["id"] for q in out} != ids:
        raise RuntimeError(f"split {split}: {len(ids) - len(out)} questions missing from data/d4")
    return out


def resolve_groups(vault, questions: list[dict]) -> list[list[set[str]]]:
    return [[set(T.resolve_all(vault, g)) for g in q["truth"]] for q in questions]


def run_queries(retriever, name: str, idx, questions, groups) -> list[dict]:
    corpus = [c["lineage_ids"] for c in idx.chunks]
    fn = getattr(retriever, name)
    rows = []
    for q, g in zip(questions, groups, strict=True):
        ranked = fn(q["question"], K)
        if len(ranked) != K:
            raise RuntimeError(f"{name}: {len(ranked)} results for {q['id']}")
        lins = [idx.chunk(k)["lineage_ids"] for k, _ in ranked]
        rows.append(
            {
                "id": q["id"],
                "type": q["type"],
                **M.query_metrics(lins, g, M.ideal_relevant_count(corpus, g)),
            }
        )
    return rows


def cell_means(rows: list[dict]) -> dict:
    return {
        m: M.mean([r[m] for r in rows])
        for m in ("ndcg@10", "recall@5", "recall@10", "mrr", "full_cover@10")
    }


def select_alpha(curve: list[dict]) -> float:
    best = min(
        curve,
        key=lambda c: (-round(c["ndcg_all"], 12), round(abs(c["alpha"] - 0.6), 12), c["alpha"]),
    )
    return best["alpha"]


def apply_alpha(alpha: float) -> None:
    src = GRAPH_SEARCH.read_text()
    new = re.sub(r"^ALPHA_DEFAULT = [0-9.]+", f"ALPHA_DEFAULT = {alpha}", src, flags=re.MULTILINE)
    if new != src:
        GRAPH_SEARCH.write_text(new)
        print("ALPHA_DEFAULT updated; commit before --phase test")


def phase_tune(run: str, apply: bool) -> None:
    vault = mdkernel_core.Vault(str(D3))
    verify_d3(vault)
    qs = load_split("tune")
    groups = resolve_groups(vault, qs)
    _, idx, graph = load_or_build("structural", vault)
    curve = []
    for a in ALPHAS:
        rows = run_queries(GS.GraphRetriever(idx, graph, alpha=a), "graph_hybrid", idx, qs, groups)
        point = {"alpha": a, "ndcg_all": M.mean([r["ndcg@10"] for r in rows])}
        for t in TYPES:
            point[f"ndcg_{t}"] = M.mean([r["ndcg@10"] for r in rows if r["type"] == t])
            point[f"n_{t}"] = sum(1 for r in rows if r["type"] == t)
        curve.append(point)
        print(
            f"alpha={a:.1f} nDCG all={point['ndcg_all']:.4f} single={point['ndcg_single']:.4f} multi={point['ndcg_multi']:.4f}"
        )
    sel = select_alpha(curve)
    out = {
        "split": "tune",
        "strategy": "graph_hybrid",
        "chunking": "structural",
        "n_questions": len(qs),
        "tune_ids": [q["id"] for q in qs],
        "curve": curve,
        "selected_alpha": sel,
        "rule": "max mean nDCG@10 over all tuning questions; ties -> closest to 0.6, then smaller",
    }
    runinfo.write_json(run, "h2_tuning.json", out)
    runinfo.record_phase(run, "h2_tune", {"selected_alpha": sel}, ["h2_tuning.json"])
    print(f"selected ALPHA={sel}")
    if apply:
        apply_alpha(sel)


def phase_test(run: str) -> None:
    tuning = runinfo.read_json(run, "h2_tuning.json")
    if abs(GS.ALPHA_DEFAULT - tuning["selected_alpha"]) > 1e-12:
        raise RuntimeError(
            f"ALPHA_DEFAULT {GS.ALPHA_DEFAULT} != tuning selection {tuning['selected_alpha']}"
        )
    vault = mdkernel_core.Vault(str(D3))
    verify_d3(vault)
    qs = load_split("test")
    if set(tuning["tune_ids"]) & {q["id"] for q in qs}:
        raise RuntimeError("tuning questions leak into the test split")
    expected = {t: sum(1 for q in qs if q["type"] == t) for t in TYPES}
    groups = resolve_groups(vault, qs)
    warm = load_split("tune")[:WARMUP]
    cells: dict = {}
    per_query: dict = {}
    latency: dict = {}
    for chunking in CHUNKINGS:
        _, idx, graph = load_or_build(chunking, vault)
        r = GS.GraphRetriever(idx, graph)
        for name in STRATEGIES:
            rows = run_queries(r, name, idx, qs, groups)
            fn = getattr(r, name)
            times = []
            for q in warm + qs:
                t0 = time.perf_counter()
                fn(q["question"], K)
                times.append((time.perf_counter() - t0) * 1000)
            latency[f"{name}/{chunking}"] = M.latency_summary(times, warmup=WARMUP)
            per_query[f"{name}/{chunking}"] = rows
            for t in TYPES:
                sub = [x for x in rows if x["type"] == t]
                if len(sub) != expected[t]:
                    raise RuntimeError(
                        f"cell {name}/{chunking}/{t}: {len(sub)} queries, expected {expected[t]}"
                    )
                cells[f"{name}/{chunking}/{t}"] = {"n": len(sub), **cell_means(sub)}
    if len(cells) != len(STRATEGIES) * len(CHUNKINGS) * len(TYPES):
        raise RuntimeError("incomplete cell grid")

    def col(strategy, t, metric):
        return [x[metric] for x in per_query[f"{strategy}/structural"] if x["type"] == t]

    boot = {}
    for a, b in (("graph_hybrid", "hybrid"), ("graph_flat", "flat")):
        for t in TYPES:
            for metric in ("ndcg@10", "full_cover@10"):
                boot[f"{a}-{b}/{t}/{metric}"] = M.paired_bootstrap(
                    col(a, t, metric), col(b, t, metric), n=BOOTSTRAP, seed=SEED
                )
    c = cells
    h = {
        "H2-1": c["graph_hybrid/structural/multi"]["ndcg@10"]
        - c["hybrid/structural/multi"]["ndcg@10"],
        "H2-2": c["graph_hybrid/structural/single"]["ndcg@10"]
        - c["hybrid/structural/single"]["ndcg@10"],
        "H2-3": latency["graph_hybrid/structural"]["p95"] / latency["hybrid/structural"]["p95"],
        "H2-4": c["graph_hybrid/structural/multi"]["full_cover@10"]
        - c["hybrid/structural/multi"]["full_cover@10"],
        "H2-5": c["graph_flat/structural/multi"]["ndcg@10"] - c["flat/structural/multi"]["ndcg@10"],
    }
    out = {
        "split": "test",
        "alpha": GS.ALPHA_DEFAULT,
        "n_questions": expected,
        **h,
        "cells": cells,
        "latency_ms": latency,
        "bootstrap": boot,
        "per_query": per_query,
    }
    runinfo.write_json(run, "h2.json", out)
    runinfo.record_phase(run, "h2_test", {"alpha": GS.ALPHA_DEFAULT}, ["h2.json"])
    print(json.dumps(h, indent=1))


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--phase", choices=["tune", "test"], required=True)
    ap.add_argument("--run", required=True)
    ap.add_argument("--apply", action="store_true")
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
