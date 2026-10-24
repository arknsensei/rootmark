"""P2-L measurement (TASKS L4; GOAL P2-L L-2..L-6), written and committed
before any measurement with D-TR (`eval.datasets.d_tr`, pre-registered).

**D-TR, 12 cells** (`p2l/d_tr.json`): for each embedder (`minilm-l6-v2`,
`multilingual-minilm-l12-v2`) and lexical variant (`mdk_lower_alnum`,
`uni_fold_tr`): the D-TR documents ingested into a fresh vault (human,
matcher v3), a recipe naming both (`[index] tokenizer`, `[index] embedder`),
one checkpoint, `index_build`, then every query searched in each mode
(`lexical`, `flat`, `hybrid`) with `k = d_tr.K_CHUNKS` at that checkpoint;
doc-level metrics per query (`d_tr` docstring: nDCG@10, recall@10, MRR on
the hits collapsed to documents), cell means; per type (`sentence`,
`title`) and language (`tr`, `en`) for information. A `lexical` cell does
not depend on the embedder, a `flat` cell not on the variant: those pairs
must be equal (`duplicates_equal`; an inequality fails the run).

**English reference** (`p2l/english.json`; L-3, L-4): the C-3 measurement
(`eval.run_c3_quality`'s pieces: D3, the D4 test split, `{hybrid, flat}`,
structural chunks, k 10, the frozen metric code) with the default embedder
and each lexical variant, compared with `data/quality_ref_rust.json`
value by value (`run_p1c1_checks.quality_vs_ref`, 618 values).

**Criteria** (GOAL P2-L; ASSUMPTIONS P2-L L4-3 for the readings):
- L-2: `lexical/uni_fold_tr/minilm-l6-v2` − `lexical/mdk_lower_alnum/minilm-l6-v2`
  nDCG@10 ≥ +0.10;
- L-3: `mdk_lower_alnum`: every reference value equal (Δ 0, 618 values);
- L-4: `uni_fold_tr`: Δ nDCG@10 of the reference's four cells
  (`{hybrid, flat}/structural/{single, multi}`), reported; the default
  tokenizer becomes `uni_fold_tr` iff the largest |Δ| ≤ 0.001, else
  `uni_fold_tr` is opt-in;
- L-5: `flat/mdk_lower_alnum/multilingual-minilm-l12-v2` −
  `flat/mdk_lower_alnum/minilm-l6-v2` nDCG@10 ≥ +0.05;
- L-6: `hybrid/uni_fold_tr/multilingual-minilm-l12-v2` −
  `hybrid/mdk_lower_alnum/minilm-l6-v2` nDCG@10 ≥ +0.10 (the old best
  over every mode of the old configuration is reported beside it).

Writes `p2l/d_tr.json`, `p2l/english.json` and `p2l_quality.json` into the
RUN. Wall-clock time is not written.

    python -m eval.run_p2l --run <RUN> [--only d_tr|english]
"""

from __future__ import annotations

import argparse
import json
import math
import shutil
import tempfile
from pathlib import Path

import rootmark_core

from eval import metrics as M
from eval import runinfo
from eval import truth as T
from eval.datasets import d_tr
from eval.datasets.common import DATA, die
from eval.run_c3_quality import CHECKPOINT, CHUNKING, K, copy_vault
from eval.run_c3_quality import MODES as C3_MODES
from eval.run_h2 import D3, TYPES, cell_means, expected_counts, load_split, verify_d3
from eval.run_p1c1_checks import quality_vs_ref

EMBEDDERS = ["minilm-l6-v2", "multilingual-minilm-l12-v2"]
TOKENIZERS = ["mdk_lower_alnum", "uni_fold_tr"]
MODES = ["lexical", "flat", "hybrid"]
METRICS = ("ndcg@10", "recall@10", "mrr")
REF = DATA / "quality_ref_rust.json"
L2, L5, L6, L4_DEFAULT = 0.10, 0.05, 0.10, 0.001
CONSUMER = "p2l-quality"

# the kernel's built-in trigger and retention (`Policy::builtin`), the
# default recipe's promotion and edit policies
RECIPE = """[promotion]
policy = "human_only"

[edit]
policy = "append_only"

[version_trigger]
idle_ms = 30000
min_sim_change = 0.2

[journal]
retention_hours = 168

[index]
tokenizer = "{tokenizer}"
embedder = "{embedder}"
"""


def recipe(vault: Path, tokenizer: str, embedder: str) -> None:
    (vault / ".rootmark" / "policy.toml").write_text(
        RECIPE.format(tokenizer=tokenizer, embedder=embedder), encoding="utf-8"
    )


def doc_metrics(hits: list[dict], truth: str) -> dict:
    docs: list[str] = []
    for h in hits:
        if h["doc_path"] not in docs:
            docs.append(h["doc_path"])
    r = docs.index(truth) + 1 if truth in docs else None
    return {
        "rank": r,
        "ndcg@10": 1.0 / math.log2(r + 1) if r is not None and r <= 10 else 0.0,
        "recall@10": 1.0 if r is not None and r <= 10 else 0.0,
        "mrr": 1.0 / r if r is not None else 0.0,
    }


def means(rows: list[dict]) -> dict:
    return {"n": len(rows), **{m: M.mean([x[m] for x in rows]) for m in METRICS}}


def d_tr_cells(work: Path) -> dict:
    docs, queries = d_tr.load_set()
    cells: dict = {}
    per_query: dict = {}
    builds: dict = {}
    for emb in EMBEDDERS:
        for tok in TOKENIZERS:
            v = work / f"{emb}--{tok}"
            vault = rootmark_core.Vault(str(v), matcher="v3")
            vault.ingest_dir(str(docs))
            recipe(v, tok, emb)
            cp = vault.checkpoint("d-tr")
            builds[f"{tok}/{emb}"] = {
                "events_hash": vault.events_hash(),
                "index": vault.index_build(),
                "manifest": json.loads((v / ".rootmark/index/manifest.json").read_text()),
            }
            for mode in MODES:
                res = vault.search_many(
                    [q["query"] for q in queries],
                    cp["checkpoint_id"],
                    k=d_tr.K_CHUNKS,
                    mode=mode,
                    consumer=CONSUMER,
                )
                rows = []
                for q, r in zip(queries, res, strict=True):
                    if any(h["layer"] != "canonical" for h in r["hits"]):
                        raise RuntimeError(f"{q['id']}: a non-canonical hit")
                    rows.append(
                        {
                            "id": q["id"],
                            "type": q["type"],
                            "lang": q["lang"],
                            **doc_metrics(r["hits"], q["truth"]),
                        }
                    )
                key = f"{mode}/{tok}/{emb}"
                per_query[key] = rows
                cells[key] = {
                    **means(rows),
                    "by_type": {
                        t: means([x for x in rows if x["type"] == t])
                        for t in sorted({x["type"] for x in rows})
                    },
                    "by_lang": {
                        g: means([x for x in rows if x["lang"] == g])
                        for g in sorted({x["lang"] for x in rows})
                    },
                }
    dup = []
    for tok in TOKENIZERS:
        a, b = (f"lexical/{tok}/{e}" for e in EMBEDDERS)
        dup.append({"cells": [a, b], "equal": per_query[a] == per_query[b]})
    for emb in EMBEDDERS:
        a, b = (f"flat/{t}/{emb}" for t in TOKENIZERS)
        dup.append({"cells": [a, b], "equal": per_query[a] == per_query[b]})
    if not all(d["equal"] for d in dup):
        raise RuntimeError(f"cells that must be equal differ: {[d for d in dup if not d['equal']]}")
    return {
        "set": json.loads(d_tr.MANIFEST.read_text(encoding="utf-8")),
        "k_chunks": d_tr.K_CHUNKS,
        "builds": builds,
        "cells": cells,
        "duplicates_equal": dup,
        "per_query": per_query,
    }


def english(work: Path, tok: str) -> dict:
    """The C-3 measurement on D3 with the default embedder and `tok`."""
    d3 = rootmark_core.Vault(str(D3))
    verify_d3(d3)
    copy_vault(work)
    recipe(work, tok, EMBEDDERS[0])
    vault = rootmark_core.Vault(str(work))
    if vault.events_hash() != d3.events_hash():
        raise RuntimeError("copied vault differs from data/d3")
    cp = vault.checkpoint(CHECKPOINT)
    build = vault.index_build()
    qs = load_split("test")
    expected = expected_counts("test")
    groups = [[set(T.resolve_all(vault, g)) for g in q["truth"]] for q in qs]
    corpus = [c["lineage_ids"] for d in vault.doc_paths() for c in vault.chunk(d, CHUNKING)]
    cells: dict = {}
    per_query: dict = {}
    for mode in C3_MODES:
        results = vault.search_many(
            [q["question"] for q in qs], CHECKPOINT, k=K, mode=mode, consumer="c3-quality"
        )
        rows = []
        for q, g, r in zip(qs, groups, results, strict=True):
            if len(r["hits"]) != K or any(h["layer"] != "canonical" for h in r["hits"]):
                raise RuntimeError(f"{mode}: {q['id']}: {len(r['hits'])} hits")
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
                raise RuntimeError(f"{mode}/{t}: {len(sub)} queries, expected {expected[t]}")
            cells[f"{mode}/{CHUNKING}/{t}"] = {"n": len(sub), **cell_means(sub)}
    return {
        "checkpoint_id": cp["checkpoint_id"],
        "index": build,
        "cells": cells,
        "per_query": per_query,
    }


def english_all(work: Path) -> dict:
    ref = json.loads(REF.read_text(encoding="utf-8"))
    out: dict = {"reference": REF.name, "source_run": ref["source_run"]}
    for tok in TOKENIZERS:
        e = english(work / tok, tok)
        cmp = quality_vs_ref(e, ref)
        delta = {
            k: e["cells"][k]["ndcg@10"] - c["ndcg@10"] for k, c in sorted(ref["cells"].items())
        }
        out[tok] = {**e, "vs_reference": cmp, "delta_ndcg@10": delta}
    return out


def criteria(dt: dict | None, en: dict | None) -> dict:
    out: dict = {}
    if dt is not None:
        c = {k: v["ndcg@10"] for k, v in dt["cells"].items()}
        old = [c[f"{m}/mdk_lower_alnum/minilm-l6-v2"] for m in MODES]
        l2 = c["lexical/uni_fold_tr/minilm-l6-v2"] - c["lexical/mdk_lower_alnum/minilm-l6-v2"]
        l5 = (
            c["flat/mdk_lower_alnum/multilingual-minilm-l12-v2"]
            - c["flat/mdk_lower_alnum/minilm-l6-v2"]
        )
        new = c["hybrid/uni_fold_tr/multilingual-minilm-l12-v2"]
        l6 = new - c["hybrid/mdk_lower_alnum/minilm-l6-v2"]
        out["L-2"] = {"value": l2, "threshold": L2, "pass": l2 >= L2}
        out["L-5"] = {"value": l5, "threshold": L5, "pass": l5 >= L5}
        out["L-6"] = {
            "value": l6,
            "threshold": L6,
            "pass": l6 >= L6,
            "info_vs_best_old_mode": new - max(old),
        }
    if en is not None:
        l3 = en["mdk_lower_alnum"]["vs_reference"]
        out["L-3"] = {"compared": l3["compared"], "differing": l3["differing"], "pass": l3["pass"]}
        d = en["uni_fold_tr"]["delta_ndcg@10"]
        worst = max(abs(x) for x in d.values())
        out["L-4"] = {
            "delta_ndcg@10": d,
            "max_abs_delta": worst,
            "threshold": L4_DEFAULT,
            "default_tokenizer": "uni_fold_tr" if worst <= L4_DEFAULT else "mdk_lower_alnum",
            "differing_values": len(en["uni_fold_tr"]["vs_reference"]["differing"]),
        }
    return out


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--only", choices=["d_tr", "english"])
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    (d / "p2l").mkdir(exist_ok=True)
    dt = en = None
    with tempfile.TemporaryDirectory(prefix="rootmark-p2l-") as tmp:
        if a.only in (None, "d_tr"):
            dt = d_tr_cells(Path(tmp) / "d_tr")
            runinfo.write_json(a.run, "p2l/d_tr.json", dt)
        if a.only in (None, "english"):
            en = english_all(Path(tmp) / "en")
            runinfo.write_json(a.run, "p2l/english.json", en)
        shutil.rmtree(tmp, ignore_errors=True)
    if dt is None and (d / "p2l/d_tr.json").exists():
        dt = json.loads((d / "p2l/d_tr.json").read_text(encoding="utf-8"))
    if en is None and (d / "p2l/english.json").exists():
        en = json.loads((d / "p2l/english.json").read_text(encoding="utf-8"))
    crit = criteria(dt, en)
    runinfo.write_json(
        a.run,
        "p2l_quality.json",
        {
            "criteria": crit,
            "cells": {k: v["ndcg@10"] for k, v in (dt or {}).get("cells", {}).items()},
        },
    )
    runinfo.record_phase(
        a.run,
        "p2l_quality",
        {
            "embedders": EMBEDDERS,
            "tokenizers": TOKENIZERS,
            "modes": MODES,
            "k_chunks": d_tr.K_CHUNKS,
        },
        ["p2l/d_tr.json", "p2l/english.json", "p2l_quality.json"],
    )
    for k, v in crit.items():
        print(f"{k}: {json.dumps({x: y for x, y in v.items() if x not in ('delta_ndcg@10',)})}")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
