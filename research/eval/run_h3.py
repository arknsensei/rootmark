"""H3 on D3: incremental indexing latency and re-chunk lineage stability.

H3-1 (as in v1): on a temporary copy of D3, 200 documents (SEED) each get one
`rephrase_light` edit (the frozen D1 operation); the clock runs from the file
write through `Watcher.process` (ingest → re-chunk → embed → hot layer) until
`hot.search(<first sentence of the new block>, 10)` returns. hit@1 / hit@10
(the re-embedded chunk among the results) are reported as information. The
embedding model is loaded before timing (ASSUMPTIONS H3); 5 warm-up edits are
not measured.

H3-2: for every D3 document, the lineages covered by `fixed` chunks and by
`structural` chunks must both equal the snapshot's lineages (loss/gain 0), and
chunking must not change the event log (hash before == after).

    python -m eval.run_h3 --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import random
import re
import shutil
import tempfile
import time
from pathlib import Path

import mdkernel_core

from eval import metrics, runinfo
from eval.build_index import load_or_build
from eval.datasets import synthetic_edits as se
from eval.datasets.common import DATA, SEED, die, load_manifest
from mdkernel import embed as E
from mdkernel.hot import HotLayer
from mdkernel.watcher import Watcher

D3 = DATA / "d3"
N_EDITS = 200
WARMUP = 5
K = 10


def verify_d3(vault) -> None:
    want = load_manifest(DATA / "MANIFEST.d3.json")["events_hash"]
    got = vault.events_hash()
    if got != want:
        raise RuntimeError(f"data/d3 events_hash {got} != MANIFEST.d3.json {want}")


SENT_RE = re.compile(r"(?<=[.!?])\s+")


def query_sentence(text: str) -> str:
    """First sentence of ≥ 5 words of the edited block's new text (else the whole block)."""
    flat = mdkernel_core.normalize(text)
    for sent in SENT_RE.split(flat):
        if len(sent.split()) >= 5:
            return sent
    return flat


def plan_edits(vault, docs_dir: Path, rng: random.Random) -> list[tuple[str, str, str, str]]:
    """(doc_path, old_raw_block, new_raw_block, query) — one `rephrase_light` edit
    (frozen D1 operation) per document, for N_EDITS + WARMUP documents."""
    order = sorted(vault.doc_paths())
    rng.shuffle(order)
    plans = []
    for d in order:
        text = (docs_dir / d).read_bytes().decode("utf-8")
        blocks, fm = se.parse_blocks(text)
        vocab = sorted(
            {
                m.group(2)
                for t in text.split()
                if (m := se.WORD_RE.match(t)) and len(m.group(2)) >= 4
            }
        )
        if len(vocab) < 2:
            continue
        res = se.op_rephrase(se.Doc(blocks, fm), rng, vocab, heavy=False)
        if res is None:
            continue
        new_blocks, [(i, _)] = res
        old_raw, new_raw = blocks[i].text, new_blocks[i].text
        if text.count(old_raw) != 1:
            continue  # the block must be locatable in the file
        plans.append((d, old_raw, new_raw, query_sentence(new_raw)))
        if len(plans) == N_EDITS + WARMUP:
            return plans
    raise RuntimeError(f"only {len(plans)} editable documents")


def h3_latency(tmp: Path) -> dict:
    shutil.copytree(D3, tmp / "d3")
    docs_dir = tmp / "d3" / "docs"
    vault = mdkernel_core.Vault(str(tmp / "d3"))
    _, index, _ = load_or_build("structural", mdkernel_core.Vault(str(D3)))
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
    measured = [r for r in rows if not r["warmup"]]
    return {
        "latencies_ms": [r["ms"] for r in measured],
        "n_edits": len(measured),
        "warmup": WARMUP,
        "hit_at_1_rate": sum(r["hit_at_1"] for r in measured) / len(measured),
        "hit_at_10_rate": sum(r["hit_at_10"] for r in measured) / len(measured),
        "rows": rows,
    }


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
