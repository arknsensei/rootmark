"""Retrieval strategies (SPEC §9): `(query, k) -> [(key, score)]`.

Ties are always broken by key so that rankings are deterministic. Query
embedding happens inside each call, so measured latency includes it.
"""

from __future__ import annotations

import numpy as np

from mdkernel import embed as E
from mdkernel.index import Index, bm25_tokens

RRF_K = 60
CANDIDATES = 50


def _ranked(pairs: list[tuple[str, float]], k: int) -> list[tuple[str, float]]:
    return sorted(pairs, key=lambda p: (-p[1], p[0]))[:k]


def rrf(*rankings: list[tuple[str, float]]) -> dict[str, float]:
    """Reciprocal rank fusion, rank starting at 1: Σ 1/(60 + rank)."""
    out: dict[str, float] = {}
    for ranking in rankings:
        for r, (key, _) in enumerate(ranking, start=1):
            out[key] = out.get(key, 0.0) + 1.0 / (RRF_K + r)
    return out


class Retriever:
    def __init__(self, index: Index, hot=None):
        self.index = index
        self.hot = hot

    def _hidden(self) -> int:
        """Cold results that the hot layer may shadow or hide."""
        return 0 if self.hot is None else len(self.hot) + len(self.hot.deleted)

    # -- primitives (query vector given) ------------------------------------
    def _flat_vec(self, qv: np.ndarray, k: int) -> list[tuple[str, float]]:
        rows = (
            self.index.table.search(qv.tolist())
            .metric("cosine")
            .limit(k + self._hidden())
            .select(["key", "_distance"])
            .to_list()
        )
        cold = [(r["key"], 1.0 - float(r["_distance"])) for r in rows]
        if self.hot is not None:
            hot = self.hot.search_vec(qv, k)
            shadow = set(self.hot.keys)
            cold = [p for p in cold if p[0] not in shadow and not self.hot.is_deleted(p[0])]
            cold = cold + hot
        return _ranked(cold, k)

    def _bm25(self, query: str, k: int) -> list[tuple[str, float]]:
        scores = self.index.bm25.get_scores(bm25_tokens(query))
        order = np.lexsort((self.index.key_array, -scores))[: k + self._hidden()]
        # a document without any query term is not "ranked" by BM25
        cold = [(self.index.keys[i], float(scores[i])) for i in order if scores[i] > 0]
        if self.hot is not None:
            hot = self.hot.bm25(query, k)
            shadow = set(self.hot.keys)
            cold = [p for p in cold if p[0] not in shadow and not self.hot.is_deleted(p[0])] + hot
        return _ranked(cold, k)

    def _hybrid_vec(self, query: str, qv: np.ndarray, k: int) -> list[tuple[str, float]]:
        fused = rrf(self._flat_vec(qv, CANDIDATES), self._bm25(query, CANDIDATES))
        return _ranked(list(fused.items()), k)

    # -- strategies ---------------------------------------------------------
    def flat(self, query: str, k: int = 10) -> list[tuple[str, float]]:
        return self._flat_vec(E.embed_query(query), k)

    def hybrid(self, query: str, k: int = 10) -> list[tuple[str, float]]:
        return self._hybrid_vec(query, E.embed_query(query), k)
