"""H3 regression under matcher v3 (T26; `make eval-h3`).

Same procedure as `eval/run_h3.py` (v2), with the D3 vault replaced by its
matcher-v3 twin: `data/d3/docs` ingested from scratch with matcher v3 (log
hash must equal `MANIFEST.d3.json:events_hash_v3`). A first ingest has no
previous snapshot, so both matchers give every block the same lineage; this
is checked document by document against `data/d3` before the v2-built
embedding index (`data/d3_index`, keyed by lineage) is reused.

H3-1: 200 `rephrase_light` edits (+5 warm-up) on a copy of the v3 vault,
write → `Watcher.process` (ingest with matcher v3 → re-chunk → embed → hot
layer) → `hot.search`; p50/p95 ms. H3-2: lineage loss of the `fixed` and
`structural` chunkings against the v3 snapshots, and the v3 log unchanged by
chunking. Writes `h3_v3.json`.

    python -m eval.run_h3_v3 --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import random
import shutil
import tempfile
import time
from pathlib import Path

import mdkernel_core

from eval import metrics_v3 as M3
from eval import runinfo
from eval.build_index import load_or_build
from eval.d3_v3 import V3_FIELD, ingest_docs, verify_docs
from eval.datasets.common import DATA, SEED, die, load_manifest
from eval.run_h3 import N_EDITS, WARMUP, K, plan_edits
from mdkernel import embed as E
from mdkernel.hot import HotLayer
from mdkernel.watcher import Watcher

D3 = DATA / "d3"
MATCHER = "v3"


def build_v3(root: Path) -> mdkernel_core.Vault:
    """`root/docs` (copy of the D3 documents) + a fresh matcher-v3 vault at `root`."""
    m = load_manifest(DATA / "MANIFEST.d3.json")
    verify_docs(m)
    if V3_FIELD not in m:
        raise RuntimeError(f"MANIFEST.d3.json has no {V3_FIELD}")
    shutil.copytree(D3 / "docs", root / "docs")
    v = ingest_docs(root, MATCHER, root / "docs")
    if v.events_hash() != m[V3_FIELD]:
        raise RuntimeError(f"v3 D3 events_hash {v.events_hash()} != manifest {m[V3_FIELD]}")
    return v


def check_same_lineages(v3: mdkernel_core.Vault, v2: mdkernel_core.Vault) -> int:
    if v3.doc_paths() != v2.doc_paths():
        raise RuntimeError("v3 and v2 D3 vaults hold different documents")
    for d in v2.doc_paths():
        if v3.snapshot(d)["blocks"] != v2.snapshot(d)["blocks"]:
            raise RuntimeError(f"{d}: v3 first-ingest lineages differ from data/d3")
    return len(v2.doc_paths())


def h3_latency(tmp: Path, v2_vault: mdkernel_core.Vault) -> dict:
    root = tmp / "lat"
    vault = build_v3(root)
    docs_dir = root / "docs"
    check_same_lineages(vault, v2_vault)
    _, index, _ = load_or_build("structural", v2_vault)
    hot = HotLayer(index)
    w = Watcher(vault, hot, "structural", docs_dir)
    plans = plan_edits(vault, docs_dir, random.Random(f"{SEED}:h3"))
    E.embed_query("warm up the model")  # model load is excluded from H3-1
    rows = []
    for i, (doc, old_raw, new_raw, query) in enumerate(plans):
        path = docs_dir / doc
        new_text = path.read_bytes().decode("utf-8").replace(old_raw, new_raw, 1)
        t0 = time.perf_counter()
        path.write_bytes(new_text.encode("utf-8"))
        out = w.process(path)
        results = hot.search(query, K)
        ms = (time.perf_counter() - t0) * 1000
        if not out["hot_added"] or not results:
            raise RuntimeError(f"{doc}: edit produced no searchable hot chunk")
        fresh = set(out["hot_added"])
        rows.append(
            {
                "doc_path": doc,
                "ms": ms,
                "warmup": i < WARMUP,
                "reembedded_chunks": len(fresh),
                "hit_at_1": results[0][0] in fresh,
                "hit_at_10": any(k in fresh for k, _ in results),
            }
        )
    if vault.matcher_lock() != MATCHER:
        raise RuntimeError(f"edits ran under {vault.matcher_lock()}, not {MATCHER}")
    measured = [r for r in rows if not r["warmup"]]
    return {
        "latencies_ms": [r["ms"] for r in measured],
        "n_edits": len(measured),
        "warmup": WARMUP,
        "hit_at_1_rate": sum(r["hit_at_1"] for r in measured) / len(measured),
        "hit_at_10_rate": sum(r["hit_at_10"] for r in measured) / len(measured),
        "rows": rows,
    }


def h3_rechunk(tmp: Path) -> dict:
    root = tmp / "rc"
    vault = build_v3(root)
    before = vault.events_hash()
    per_strategy = {}
    for s in ("fixed", "structural"):
        per_strategy[s] = {
            lin for d in vault.doc_paths() for c in vault.chunk(d, s) for lin in c["lineage_ids"]
        }
    snap = {lin for d in vault.doc_paths() for lin, _ in vault.snapshot(d)["blocks"]}
    fixed = M3.M.lineage_loss(snap, per_strategy["fixed"])
    structural = M3.M.lineage_loss(snap, per_strategy["structural"])
    switch = M3.M.lineage_loss(per_strategy["fixed"], per_strategy["structural"])
    after = mdkernel_core.Vault(str(root), matcher=MATCHER).events_hash()
    return {
        "lineages": len(snap),
        "fixed_vs_snapshot": fixed,
        "structural_vs_snapshot": structural,
        "fixed_to_structural": switch,
        "lost": fixed["lost"] + structural["lost"] + switch["lost"],
        "gained": fixed["gained"] + structural["gained"] + switch["gained"],
        "events_hash_before": before,
        "events_hash_after": after,
        "log_equal": before == after,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    v2_vault = mdkernel_core.Vault(str(D3))
    want = load_manifest(DATA / "MANIFEST.d3.json")["events_hash"]
    if v2_vault.events_hash() != want:
        raise RuntimeError(f"data/d3 events_hash {v2_vault.events_hash()} != manifest {want}")
    with tempfile.TemporaryDirectory() as t:
        lat = h3_latency(Path(t), v2_vault)
        rc = h3_rechunk(Path(t))
    summary = M3.M.h3_summary(
        lat["latencies_ms"], lost=rc["lost"], gained=rc["gained"], log_equal=rc["log_equal"]
    )
    out = {
        "matcher": MATCHER,
        "H3-1": {"p50_ms": summary["p50"], "p95_ms": summary["p95"], "n": summary["n"]},
        "H3-2": {"lost": rc["lost"], "gained": rc["gained"]},
        "summary": summary,
        "latency": lat,
        "rechunk": rc,
    }
    runinfo.write_json(a.run, "h3_v3.json", out)
    runinfo.record_phase(
        a.run,
        "h3_v3",
        {"matcher": MATCHER, "n_edits": N_EDITS, "warmup": WARMUP, "k": K},
        ["h3_v3.json"],
    )
    print(f"hit@1 {lat['hit_at_1_rate']:.3f} hit@10 {lat['hit_at_10_rate']:.3f}")
    print(
        json.dumps(
            {"H3-1": out["H3-1"], "H3-2": out["H3-2"], "log_equal": rc["log_equal"]}, indent=1
        )
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
