"""Tokenizer reference for the Rust lexical index (P1-C TASKS C2).

Writes `data/tok_ref.json`: 200 fixed D3 structural chunks and their prototype
BM25 tokens (`mdkernel.index.bm25_tokens`: `text.lower()`, then `[a-z0-9]+`).
The Rust test `index/tests/lexical.rs` requires the same token list for every
chunk, from the tokenizer function and from the tantivy analyzer.

Selection is fixed before any comparison: every D3 structural chunk in
`(doc_path, chunk_id)` order, then the evenly spaced picks `i * n // 200`,
plus a fixed list of edge strings (Unicode case mapping, digits, punctuation).
D3 must match `MANIFEST.d3.json` (`make d3`).

    python -m eval.gen_tok_ref
"""

from __future__ import annotations

import json

from eval.datasets.common import DATA, die
from eval.gen_embed_ref import d3_chunks, pick
from mdkernel.index import bm25_tokens

OUT = DATA / "tok_ref.json"
COUNT = 200
EDGE = [
    "",
    "   ",
    "Hello, World! 123abc ABC-def_ghi",
    "İstanbul ŞEHİR Çay ĞÜÖ ı İ",
    "Straße ΣΊΣΥΦΟΣ Ωmega ǅ ﬁne ＡＢＣ１２３",
    "KK kelvin Ångström café naïve",
    "x86_64 v1.2.3 e=mc^2 #heading [[link|label]] `code` a\tb\nc",
    "mixed123numbers 4567 ab12cd 0x1F",
]


def main() -> None:
    events_hash, cs = d3_chunks()
    sel = pick(cs, COUNT)
    out = {
        "tokenizer": "prototype mdkernel.index.bm25_tokens: text.lower(), re [a-z0-9]+",
        "d3_events_hash": events_hash,
        "strategy": "structural",
        "selection": f"(doc_path, chunk_id) order, picks i * n // {COUNT}, n = {len(cs)}",
        "chunks": [
            {
                "doc_path": c["doc_path"],
                "chunk_id": c["chunk_id"],
                "text": c["text"],
                "tokens": bm25_tokens(c["text"]),
            }
            for c in sel
        ],
        "edge": [{"text": t, "tokens": bm25_tokens(t)} for t in EDGE],
    }
    OUT.write_text(json.dumps(out, ensure_ascii=False, indent=1) + "\n")
    print(f"wrote {OUT} ({COUNT} chunks of {len(cs)}, {len(EDGE)} edge strings)")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
