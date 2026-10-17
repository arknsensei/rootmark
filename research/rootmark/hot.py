"""Hot layer (SPEC §10): in-memory numpy matrix in front of the LanceDB table.

`flat`/`hybrid` merge hot and cold results; a key present in the hot layer
shadows the cold one, and keys deleted by a re-chunk are hidden until
`compact()` writes everything to LanceDB.
"""

from __future__ import annotations

import numpy as np
import pyarrow as pa
from rank_bm25 import BM25Okapi

from rootmark import embed as E
from rootmark.index import Index, bm25_tokens, schema


class HotLayer:
    def __init__(self, index: Index):
        self.index = index
        self.keys: list[str] = []
        self.chunks: list[dict] = []
        self.matrix = np.zeros((0, E.DIM), dtype=np.float32)
        self.deleted: set[str] = set()

    def __len__(self) -> int:
        return len(self.keys)

    def add(self, chunks: list[dict], vectors: np.ndarray | None = None) -> None:
        if not chunks:
            return
        if vectors is None:
            vectors = E.embed([c["text"] for c in chunks])
        for c, v in zip(chunks, vectors, strict=True):
            k = c["key"]
            self.deleted.discard(k)
            if k in self.keys:
                i = self.keys.index(k)
                self.chunks[i] = c
                self.matrix[i] = v
            else:
                self.keys.append(k)
                self.chunks.append(c)
                self.matrix = np.vstack([self.matrix, v[None, :]])

    def delete(self, keys) -> None:
        for k in keys:
            self.deleted.add(k)
            if k in self.keys:
                i = self.keys.index(k)
                del self.keys[i], self.chunks[i]
                self.matrix = np.delete(self.matrix, i, axis=0)

    def is_deleted(self, key: str) -> bool:
        return key in self.deleted

    def search_vec(self, qv: np.ndarray, k: int) -> list[tuple[str, float]]:
        if not self.keys:
            return []
        s = self.matrix @ qv
        order = np.lexsort((np.array(self.keys), -s))[:k]
        return [(self.keys[i], float(s[i])) for i in order]

    def search(self, q: str, k: int) -> list[tuple[str, float]]:
        return self.search_vec(E.embed_query(q), k)

    def bm25(self, query: str, k: int) -> list[tuple[str, float]]:
        """BM25 of hot chunks scored with the cold model's statistics (idf,
        avgdl, k1, b), so hot and cold scores are on one scale."""
        if not self.keys:
            return []
        m = self.index.bm25
        q = bm25_tokens(query)
        s = np.zeros(len(self.chunks))
        for i, c in enumerate(self.chunks):
            toks = bm25_tokens(c["text"])
            dl = len(toks)
            tf: dict[str, int] = {}
            for t in toks:
                tf[t] = tf.get(t, 0) + 1
            for t in q:
                f = tf.get(t, 0)
                if f:
                    s[i] += (
                        (m.idf.get(t) or 0.0)
                        * f
                        * (m.k1 + 1)
                        / (f + m.k1 * (1 - m.b + m.b * dl / m.avgdl))
                    )
        order = np.lexsort((np.array(self.keys), -s))[:k]
        return [(self.keys[i], float(s[i])) for i in order if s[i] > 0]

    def compact(self) -> None:
        """Writes hot chunks and deletions to the cold LanceDB table and index."""
        idx = self.index
        drop = set(self.deleted) | set(self.keys)
        if drop:
            quoted = ", ".join("'" + k.replace("'", "''") + "'" for k in sorted(drop))
            idx.table.delete(f"key IN ({quoted})")
        if self.keys:
            rows = [
                {
                    "key": c["key"],
                    "doc_path": c["doc_path"],
                    "chunk_id": c["chunk_id"],
                    "lineage_ids": c["lineage_ids"],
                    "heading_path": c["heading_path"],
                    "text": c["text"],
                    "vector": self.matrix[i].tolist(),
                }
                for i, c in enumerate(self.chunks)
            ]
            idx.table.add(pa.Table.from_pylist(rows, schema=schema()))
        keep = [i for i, k in enumerate(idx.keys) if k not in drop]
        chunks = [idx.chunks[i] for i in keep] + self.chunks
        vectors = np.vstack([idx.vectors[keep], self.matrix]) if self.keys else idx.vectors[keep]
        idx.chunks = chunks
        idx.vectors = vectors.astype(np.float32)
        idx.bm25 = BM25Okapi([bm25_tokens(c["text"]) for c in chunks])
        idx.__post_init__()
        self.keys, self.chunks = [], []
        self.matrix = np.zeros((0, E.DIM), dtype=np.float32)
        self.deleted = set()
