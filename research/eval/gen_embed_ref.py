"""Embedding reference for the Rust ONNX embedder (P1-C TASKS C1, GOAL C-10).

Writes `data/embed_ref.json`: 50 fixed D3 structural chunks and their vectors
from the prototype embedder (`rootmark.embed`: sentence-transformers
`all-MiniLM-L6-v2` at the pinned revision, CPU, normalized). The Rust test
`index/tests/embed.rs` requires cosine >= 0.999 for every chunk.

Selection is fixed before any comparison: every D3 structural chunk in
`(doc_path, chunk_id)` order, then the evenly spaced picks `i * n // 50`.
D3 must match `MANIFEST.d3.json` (`make d3`).

    python -m eval.gen_embed_ref
"""

from __future__ import annotations

import json

import rootmark_core

from eval.datasets.common import DATA, die, load_manifest
from rootmark import embed as E

D3 = DATA / "d3"
OUT = DATA / "embed_ref.json"
COUNT = 50


def d3_chunks() -> tuple[str, list[dict]]:
    vault = rootmark_core.Vault(str(D3))
    want = load_manifest(DATA / "MANIFEST.d3.json")["events_hash"]
    if vault.events_hash() != want:
        raise RuntimeError(f"data/d3 events_hash {vault.events_hash()} != MANIFEST.d3.json {want}")
    out = [c for d in vault.doc_paths() for c in vault.chunk(d, "structural")]
    out.sort(key=lambda c: (c["doc_path"], c["chunk_id"]))
    return want, out


def pick(cs: list[dict], count: int) -> list[dict]:
    n = len(cs)
    if n < count:
        raise RuntimeError(f"{n} chunks < {count}")
    return [cs[i * n // count] for i in range(count)]


def main() -> None:
    events_hash, cs = d3_chunks()
    sel = pick(cs, COUNT)
    vecs = E.embed([c["text"] for c in sel])
    if vecs.shape != (COUNT, E.DIM):
        raise RuntimeError(f"embedding shape {vecs.shape}")
    out = {
        "model": E.MODEL,
        "revision": E.REVISION,
        "d3_events_hash": events_hash,
        "strategy": "structural",
        "selection": f"(doc_path, chunk_id) order, picks i * n // {COUNT}, n = {len(cs)}",
        "chunks": [
            {
                "doc_path": c["doc_path"],
                "chunk_id": c["chunk_id"],
                "token_count": c["token_count"],
                "text": c["text"],
                "vector": [float(x) for x in v],
            }
            for c, v in zip(sel, vecs, strict=True)
        ],
    }
    OUT.write_text(json.dumps(out, ensure_ascii=False, indent=1) + "\n")
    print(f"wrote {OUT} ({COUNT} chunks of {len(cs)})")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
