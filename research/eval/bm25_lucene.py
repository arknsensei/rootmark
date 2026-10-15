"""Lucene/tantivy BM25 (K-C1.1; P1-C.1/C1.1), in Python, over the prototype's
tokens (`mdkernel.index.bm25_tokens`), for the C1-1 evidence (K-C1.2).

The variant the Rust index records in its manifest (`lexical.*`):

- `variant` `lucene-bm25`, `k1` 1.2, `b` 0.75
- IDF `ln(1 + (N - n + 0.5) / (n + 0.5))` — always positive (the prototype's
  `rank_bm25.BM25Okapi` uses `ln((N - n + 0.5) / (n + 0.5))` with `k1` 1.5
  and an `epsilon * mean IDF` floor for negative values)
- document length = token count; average length over the scored corpus
  (1 when the corpus holds no token, as the Rust index)
- a repeated query token counts again (as `BM25Okapi.get_scores` and the
  Rust index)

`score(doc) = sum over query tokens t in doc of
idf(t) * tf * (k1 + 1) / (tf + k1 * (1 - b + b * dl / avgdl))`, accumulated
in query-token order with the same operation order as
`index/src/search.rs::bm25`. Documents without any query token are not
scored (`hit` false). Written from the formula only: no Rust code is read.
"""

from __future__ import annotations

import math
from collections import Counter

VARIANT = "lucene-bm25"
K1 = 1.2
B = 0.75
IDF = "ln(1 + (N - n + 0.5) / (n + 0.5))"


def idf(n_docs: int, df: int) -> float:
    return math.log(1.0 + (n_docs - df + 0.5) / (df + 0.5))


class LuceneBM25:
    """BM25 over a fixed corpus of token lists (row i ↔ corpus[i])."""

    def __init__(self, corpus: list[list[str]]):
        self.n_docs = len(corpus)
        self.doc_len = [len(toks) for toks in corpus]
        total = sum(self.doc_len)
        self.avgdl = total / self.n_docs if self.n_docs and total else 1.0
        self.postings: dict[str, list[tuple[int, int]]] = {}
        for i, toks in enumerate(corpus):
            for t, tf in Counter(toks).items():
                self.postings.setdefault(t, []).append((i, tf))

    def scores(self, query: list[str]) -> tuple[list[float], list[bool]]:
        """(score per document, whether the document holds a query token)."""
        score = [0.0] * self.n_docs
        hit = [False] * self.n_docs
        for t in query:
            p = self.postings.get(t)
            if not p:
                continue
            w = idf(self.n_docs, len(p))
            for i, tf in p:
                tf = float(tf)
                dl = float(self.doc_len[i])
                score[i] += w * tf * (K1 + 1.0) / (tf + K1 * (1.0 - B + B * dl / self.avgdl))
                hit[i] = True
        return score, hit
