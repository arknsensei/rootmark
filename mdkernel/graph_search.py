"""Graph-aware strategies (SPEC §9): `graph_flat`, `graph_hybrid`, `hier`.

Ego subgraph: union of 2-hop ego graphs of the 5 seeds, at most 2000 nodes
(pruned by edge weight). Personalized PageRank on that subgraph with a sparse
power iteration: alpha 0.85, uniform restart mass on the seeds only, at most
50 iterations or L1 change < 1e-6.
"""

from __future__ import annotations

import networkx as nx
import numpy as np
import scipy.sparse as sp

from mdkernel import embed as E
from mdkernel.index import bm25_tokens
from mdkernel.retrieval import CANDIDATES, Retriever, _ranked, rrf

ALPHA_DEFAULT = 0.6  # set by `eval/run_h2.py --phase tune`; test phase checks equality
SEEDS = 5
HOPS = 2
MAX_NODES = 2000
PPR_ALPHA = 0.85
PPR_ITERS = 50
PPR_TOL = 1e-6
HIER_DOCS = 20


def ego_subgraph(
    g: nx.Graph, seeds: list[str], hops: int = HOPS, max_nodes: int = MAX_NODES
) -> list[str]:
    """Nodes of the union of `hops`-hop ego graphs of `seeds`, deterministic order.

    If the union exceeds `max_nodes`, nodes are admitted hop by hop; within a
    hop, by the strongest edge weight connecting them to already admitted
    nodes (ties by key). Seeds are always kept.
    """
    admitted: list[str] = list(dict.fromkeys(s for s in seeds if s in g))
    seen = set(admitted)
    frontier = list(admitted)
    for _ in range(hops):
        best: dict[str, float] = {}
        for u in frontier:
            for v, d in g[u].items():
                if v not in seen:
                    w = d["weight"]
                    if w > best.get(v, -1.0):
                        best[v] = w
        nxt = sorted(best, key=lambda v: (-best[v], v))
        room = max_nodes - len(admitted)
        nxt = nxt[: max(room, 0)]
        admitted.extend(nxt)
        seen.update(nxt)
        frontier = nxt
        if len(admitted) >= max_nodes:
            break
    return admitted


def personalized_pagerank(g: nx.Graph, nodes: list[str], seeds: list[str]) -> np.ndarray:
    """PPR scores over `nodes` (subgraph), restart mass uniform on `seeds`."""
    pos = {n: i for i, n in enumerate(nodes)}
    rows, cols, vals = [], [], []
    for u in nodes:
        for v, d in g[u].items():
            if v in pos:
                rows.append(pos[v])
                cols.append(pos[u])
                vals.append(d["weight"])
    n = len(nodes)
    w = sp.csc_matrix((vals, (rows, cols)), shape=(n, n), dtype=np.float64)
    out = np.asarray(w.sum(axis=0)).ravel()
    dangling = out == 0
    inv = np.where(dangling, 0.0, 1.0 / np.where(dangling, 1.0, out))
    p = w @ sp.diags(inv)  # column-stochastic on non-dangling columns
    restart = np.zeros(n)
    sidx = [pos[s] for s in seeds if s in pos]
    restart[sidx] = 1.0 / len(sidx)
    r = restart.copy()
    for _ in range(PPR_ITERS):
        nxt = PPR_ALPHA * (p @ r + r[dangling].sum() * restart) + (1 - PPR_ALPHA) * restart
        diff = np.abs(nxt - r).sum()
        r = nxt
        if diff < PPR_TOL:
            break
    return r


class GraphRetriever(Retriever):
    def __init__(self, index, graph: nx.Graph, hot=None, alpha: float = ALPHA_DEFAULT):
        super().__init__(index, hot)
        self.graph = graph
        self.alpha = alpha

    def _cos(self, qv: np.ndarray, keys: list[str]) -> np.ndarray:
        return self.index.vectors[[self.index.pos[k] for k in keys]] @ qv

    def _propagate(self, seeds: list[str]) -> tuple[list[str], np.ndarray]:
        nodes = ego_subgraph(self.graph, seeds)
        ppr = personalized_pagerank(self.graph, nodes, seeds)
        mx = ppr.max()
        return nodes, (ppr / mx if mx > 0 else ppr)

    def graph_flat(self, query: str, k: int = 10) -> list[tuple[str, float]]:
        qv = E.embed_query(query)
        seeds = [key for key, _ in self._flat_vec(qv, SEEDS)]
        nodes, ppr = self._propagate(seeds)
        cos = self._cos(qv, nodes)
        score = self.alpha * cos + (1 - self.alpha) * ppr
        return _ranked(list(zip(nodes, map(float, score), strict=True)), k)

    def graph_hybrid(self, query: str, k: int = 10) -> list[tuple[str, float]]:
        qv = E.embed_query(query)
        fused = rrf(self._flat_vec(qv, CANDIDATES), self._bm25(query, CANDIDATES))
        seeds = [key for key, _ in _ranked(list(fused.items()), SEEDS)]
        nodes, ppr = self._propagate(seeds)
        rrf_max = max(fused[s] for s in seeds)
        cos = self._cos(qv, nodes)
        cmax = cos.max()
        cos_norm = np.clip(cos / cmax, 0.0, None) if cmax > 0 else np.zeros_like(cos)
        rrf_norm = np.array(
            [
                fused[n] / rrf_max if n in fused else 0.5 * c
                for n, c in zip(nodes, cos_norm, strict=True)
            ]
        )
        score = self.alpha * rrf_norm + (1 - self.alpha) * ppr
        return _ranked(list(zip(nodes, map(float, score), strict=True)), k)

    def hier(self, query: str, k: int = 10) -> list[tuple[str, float]]:
        """Top-20 documents by document embedding, then `hybrid` inside them."""
        qv = E.embed_query(query)
        idx = self.index
        docs = sorted(idx.doc_vectors, key=lambda d: (-float(idx.doc_vectors[d] @ qv), d))[
            :HIER_DOCS
        ]
        rows = sorted(r for d in docs for r in idx.doc_rows[d])
        cos = idx.vectors[rows] @ qv
        flat = _ranked(
            [(idx.keys[r], float(c)) for r, c in zip(rows, cos, strict=True)], CANDIDATES
        )
        scores = idx.bm25.get_scores(bm25_tokens(query))
        bm = _ranked([(idx.keys[r], float(scores[r])) for r in rows if scores[r] > 0], CANDIDATES)
        return _ranked(list(rrf(flat, bm).items()), k)
