"""Embeddings: `sentence-transformers/all-MiniLM-L6-v2`, CPU, normalized.

The model revision is pinned so that two clean runs load identical weights.
"""

from __future__ import annotations

from functools import lru_cache

import numpy as np

MODEL = "sentence-transformers/all-MiniLM-L6-v2"
REVISION = "1110a243fdf4706b3f48f1d95db1a4f5529b4d41"
DIM = 384
BATCH = 64
TORCH_THREADS = 4


@lru_cache(maxsize=1)
def model():
    import torch
    from sentence_transformers import SentenceTransformer

    torch.manual_seed(0)
    torch.set_num_threads(TORCH_THREADS)
    return SentenceTransformer(MODEL, revision=REVISION, device="cpu")


def embed(texts: list[str]) -> np.ndarray:
    """(n, 384) float32, L2-normalized; input order preserved."""
    if not texts:
        return np.zeros((0, DIM), dtype=np.float32)
    v = model().encode(
        texts,
        batch_size=BATCH,
        normalize_embeddings=True,
        convert_to_numpy=True,
        show_progress_bar=False,
    )
    return np.asarray(v, dtype=np.float32)


def embed_query(q: str) -> np.ndarray:
    return embed([q])[0]
