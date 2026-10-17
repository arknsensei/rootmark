"""Builds (or loads) the D3 index and chunk graph for one chunk strategy.

Cached under `data/d3_index/<strategy>/`, keyed by the D3 `events_hash`, the
embedding model revision and the strategy; any mismatch rebuilds from scratch.
The cache lives outside `data/d3` so that rebuilding D3 (H4) does not delete
it, and is invalidated whenever D3 changes.

    python -m eval.build_index --strategy structural [--stats]
"""

from __future__ import annotations

import argparse
import json
import pickle
import shutil

import rootmark_core

from eval.datasets.common import DATA, die, dump_json
from rootmark import embed as E
from rootmark import graph as G
from rootmark import index as I

D3 = DATA / "d3"
CACHE = DATA / "d3_index"
STRATEGIES = ("structural", "fixed")


def _stamp(vault, strategy: str) -> dict:
    return {
        "events_hash": vault.events_hash(),
        "model": E.MODEL,
        "revision": E.REVISION,
        "strategy": strategy,
        "graph": {
            "wikilink": G.W_WIKILINK,
            "consecutive": G.W_CONSECUTIVE,
            "same_parent": G.W_SAME_PARENT,
            "knn": G.W_KNN,
            "k": G.KNN,
            "cap": G.CAP,
        },
    }


def load_or_build(strategy: str, vault: rootmark_core.Vault | None = None):
    """Returns (vault, index, graph)."""
    if strategy not in STRATEGIES:
        raise ValueError(strategy)
    vault = vault or rootmark_core.Vault(str(D3))
    if not vault.doc_paths():
        raise RuntimeError("data/d3 is empty; run `make d3` first")
    d = CACHE / strategy
    stamp = _stamp(vault, strategy)
    stamp_file = d / "stamp.json"
    if stamp_file.exists() and json.loads(stamp_file.read_text()) == stamp:
        with open(d / "index_state.pkl", "rb") as f:
            chunks, vectors = pickle.load(f)
        import lancedb

        table = lancedb.connect(str(d / "lance")).open_table(I.table_name(strategy))
        from rank_bm25 import BM25Okapi

        index = I.Index(
            strategy, chunks, vectors, table, BM25Okapi([I.bm25_tokens(c["text"]) for c in chunks])
        )
        return vault, index, G.load(G.graph_path(d, strategy))
    if d.exists():
        shutil.rmtree(d)
    d.mkdir(parents=True)
    index = I.build(vault, strategy, d / "lance")
    graph = G.build_graph(index)
    G.save(graph, G.graph_path(d, strategy))
    with open(d / "index_state.pkl", "wb") as f:
        pickle.dump((index.chunks, index.vectors), f, protocol=pickle.HIGHEST_PROTOCOL)
    stamp_file.write_text(dump_json(stamp))
    return vault, index, graph


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--strategy", choices=STRATEGIES, required=True)
    ap.add_argument("--stats", action="store_true")
    a = ap.parse_args(argv)
    _, index, graph = load_or_build(a.strategy)
    print(f"{a.strategy}: {len(index.keys)} chunks")
    if a.stats:
        print(json.dumps(G.edge_stats(graph), indent=1))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
