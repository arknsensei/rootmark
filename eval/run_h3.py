"""H3 on D3: incremental indexing latency and re-chunk lineage stability.

H3-1: on a temporary copy of D3, a probe paragraph is appended to N documents
one at a time; the clock runs from the file write through `Watcher.process`
(ingest → re-chunk → embed → hot layer) until a `flat` search for the probe
text returns one of the newly added hot chunks. The embedding model is loaded
before timing (ASSUMPTIONS H3). Warm-up edits are not measured.

H3-2: for every D3 document, the lineages covered by `fixed` chunks and by
`structural` chunks must both equal the snapshot's lineages (loss/gain 0), and
chunking must not change the event log (hash before == after).

    python -m eval.run_h3 --run <RUN>
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

from eval import metrics, runinfo
from eval.build_index import load_or_build
from eval.datasets.common import DATA, SEED, die, load_manifest
from mdkernel import embed as E
from mdkernel.hot import HotLayer
from mdkernel.retrieval import Retriever
from mdkernel.watcher import Watcher

D3 = DATA / "d3"
N_EDITS = 50
WARMUP = 5
K = 10
TIMEOUT_MS = 10_000


def verify_d3(vault) -> None:
    want = load_manifest(DATA / "MANIFEST.d3.json")["events_hash"]
    got = vault.events_hash()
    if got != want:
        raise RuntimeError(f"data/d3 events_hash {got} != MANIFEST.d3.json {want}")


def probe_text(i: int) -> str:
    return f"Probe paragraph {i}: the zephyrine quartz marmot{i} catalogue lists harbour lanterns."


def h3_latency(tmp: Path) -> dict:
    shutil.copytree(D3, tmp / "d3")
    vault = mdkernel_core.Vault(str(tmp / "d3"))
    _, index, _ = load_or_build("structural", mdkernel_core.Vault(str(D3)))
    hot = HotLayer(index)
    r = Retriever(index, hot)
    w = Watcher(vault, hot, "structural", tmp / "d3" / "docs")
    E.embed_query("warm up the model")  # model load is excluded from H3-1
    docs = sorted(vault.doc_paths())
    rng = random.Random(f"{SEED}:h3")
    picks = rng.sample(docs, N_EDITS + WARMUP)
    latencies, found = [], []
    for i, doc in enumerate(picks):
        path = tmp / "d3" / "docs" / doc
        text = path.read_bytes().decode("utf-8")
        t0 = time.perf_counter()
        path.write_bytes((text.rstrip("\n") + "\n\n" + probe_text(i) + "\n").encode("utf-8"))
        out = w.process(path)
        new = set(out["hot_added"])
        hit = False
        while (time.perf_counter() - t0) * 1000 < TIMEOUT_MS:
            if new & {k for k, _ in r.flat(probe_text(i), K)}:
                hit = True
                break
        ms = (time.perf_counter() - t0) * 1000
        if i >= WARMUP:
            latencies.append(ms)
            found.append(hit)
    if not all(found):
        raise RuntimeError(f"{found.count(False)} probe edits never became searchable")
    return {"latencies_ms": latencies, "n_edits": len(latencies), "warmup": WARMUP}


def h3_rechunk() -> dict:
    vault = mdkernel_core.Vault(str(D3))
    before = vault.events_hash()
    lost = gained = 0
    per_strategy = {}
    snap_total = set()
    for s in ("fixed", "structural"):
        covered = set()
        for d in vault.doc_paths():
            covered |= {lin for c in vault.chunk(d, s) for lin in c["lineage_ids"]}
        per_strategy[s] = covered
    for d in vault.doc_paths():
        snap_total |= {lin for lin, _ in vault.snapshot(d)["blocks"]}
    for s in ("fixed", "structural"):
        loss = metrics.lineage_loss(snap_total, per_strategy[s])
        lost += loss["lost"]
        gained += loss["gained"]
    switch = metrics.lineage_loss(per_strategy["fixed"], per_strategy["structural"])
    lost += switch["lost"]
    gained += switch["gained"]
    after = mdkernel_core.Vault(str(D3)).events_hash()
    return {
        "lineages": len(snap_total),
        "fixed_vs_snapshot": metrics.lineage_loss(snap_total, per_strategy["fixed"]),
        "structural_vs_snapshot": metrics.lineage_loss(snap_total, per_strategy["structural"]),
        "fixed_to_structural": switch,
        "lost": lost,
        "gained": gained,
        "events_hash_before": before,
        "events_hash_after": after,
        "log_equal": before == after,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    verify_d3(mdkernel_core.Vault(str(D3)))
    with tempfile.TemporaryDirectory() as t:
        lat = h3_latency(Path(t))
    rc = h3_rechunk()
    summary = metrics.h3_summary(
        lat["latencies_ms"], lost=rc["lost"], gained=rc["gained"], log_equal=rc["log_equal"]
    )
    out = {
        "H3-1": {"p50_ms": summary["p50"], "p95_ms": summary["p95"], "n": summary["n"]},
        "H3-2": {"lost": rc["lost"], "gained": rc["gained"]},
        "summary": summary,
        "latency": lat,
        "rechunk": rc,
    }
    runinfo.write_json(a.run, "h3.json", out)
    runinfo.record_phase(a.run, "h3", {"n_edits": N_EDITS, "warmup": WARMUP, "k": K}, ["h3.json"])
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
