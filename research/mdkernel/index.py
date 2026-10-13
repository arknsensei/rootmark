"""Chunk index (SPEC §8): LanceDB vectors, BM25, document embeddings.

Index key = `doc_path + "::" + chunk_id`; the same chunk in two documents is
kept twice. Chunks come only from the Rust core (`Vault.chunk`).
"""

from __future__ import annotations

import re
import shutil
from dataclasses import dataclass, field
from pathlib import Path

import lancedb
import mdkernel_core
import numpy as np
import pyarrow as pa
from rank_bm25 import BM25Okapi

from mdkernel import embed as E

TOKEN_RE = re.compile(r"[a-z0-9]+")


def key_of(doc_path: str, chunk_id: str) -> str:
    return f"{doc_path}::{chunk_id}"


def bm25_tokens(text: str) -> list[str]:
    return TOKEN_RE.findall(text.lower())


def collect_chunks(vault: mdkernel_core.Vault, strategy: str) -> list[dict]:
    """All chunks of all documents in `doc_path` order, with `key` added."""
    out = []
    seen = set()
    for d in vault.doc_paths():
        for c in vault.chunk(d, strategy):
            c["key"] = key_of(c["doc_path"], c["chunk_id"])
            if c["key"] in seen:
                raise RuntimeError(f"duplicate index key {c['key']}")
            seen.add(c["key"])
            out.append(c)
    return out


def schema() -> pa.Schema:
    return pa.schema(
        [
            ("key", pa.string()),
            ("doc_path", pa.string()),
            ("chunk_id", pa.string()),
            ("lineage_ids", pa.list_(pa.string())),
            ("heading_path", pa.list_(pa.string())),
            ("text", pa.string()),
            ("vector", pa.list_(pa.float32(), E.DIM)),
        ]
    )


@dataclass
class Index:
    strategy: str
    chunks: list[dict]
    vectors: np.ndarray  # (n, 384), row i ↔ chunks[i]
    table: object  # LanceDB table
    bm25: BM25Okapi
    keys: list[str] = field(init=False)
    key_array: np.ndarray = field(init=False)
    pos: dict[str, int] = field(init=False)
    doc_vectors: dict[str, np.ndarray] = field(init=False)
    doc_rows: dict[str, list[int]] = field(init=False)

    def __post_init__(self):
        self.keys = [c["key"] for c in self.chunks]
        self.key_array = np.array(self.keys)
        self.pos = {k: i for i, k in enumerate(self.keys)}
        self.doc_rows = {}
        for i, c in enumerate(self.chunks):
            self.doc_rows.setdefault(c["doc_path"], []).append(i)
        self.doc_vectors = {}
        for d, rows in self.doc_rows.items():
            m = self.vectors[rows].mean(axis=0)
            n = np.linalg.norm(m)
            self.doc_vectors[d] = (m / n if n > 0 else m).astype(np.float32)

    def chunk(self, key: str) -> dict:
        return self.chunks[self.pos[key]]


def table_name(strategy: str) -> str:
    return f"chunks_{strategy}"


def build(vault: mdkernel_core.Vault, strategy: str, db_dir: Path) -> Index:
    chunks = collect_chunks(vault, strategy)
    if not chunks:
        raise RuntimeError("no chunks to index")
    vectors = E.embed([c["text"] for c in chunks])
    db_dir.mkdir(parents=True, exist_ok=True)
    db = lancedb.connect(str(db_dir))
    name = table_name(strategy)
    if name in db.list_tables().tables:
        db.drop_table(name)
    rows = [
        {
            "key": c["key"],
            "doc_path": c["doc_path"],
            "chunk_id": c["chunk_id"],
            "lineage_ids": c["lineage_ids"],
            "heading_path": c["heading_path"],
            "text": c["text"],
            "vector": vectors[i].tolist(),
        }
        for i, c in enumerate(chunks)
    ]
    table = db.create_table(name, data=pa.Table.from_pylist(rows, schema=schema()))
    bm25 = BM25Okapi([bm25_tokens(c["text"]) for c in chunks])
    return Index(strategy, chunks, vectors, table, bm25)


def reset(db_dir: Path) -> None:
    if db_dir.exists():
        shutil.rmtree(db_dir)
