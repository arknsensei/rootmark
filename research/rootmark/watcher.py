"""Watcher (SPEC §10): file change → `Vault.ingest` → re-chunk the document →
chunks that contain a changed lineage go to the hot layer; chunks that no
longer exist are hidden."""

from __future__ import annotations

from pathlib import Path

import rootmark_core

from rootmark.hot import HotLayer
from rootmark.index import key_of


class Watcher:
    def __init__(self, vault: rootmark_core.Vault, hot: HotLayer, strategy: str, docs_root: Path):
        self.vault = vault
        self.hot = hot
        self.strategy = strategy
        self.docs_root = Path(docs_root)

    def process(self, path: Path) -> dict:
        path = Path(path)
        doc = path.relative_to(self.docs_root).as_posix()
        r = self.vault.ingest(doc, path.read_text(encoding="utf-8"))
        changed = set(r["changed_lineages"])
        new_chunks = self.vault.chunk(doc, self.strategy)
        for c in new_chunks:
            c["key"] = key_of(c["doc_path"], c["chunk_id"])
        new_keys = {c["key"] for c in new_chunks}
        old_keys = {self.hot.index.keys[i] for i in self.hot.index.doc_rows.get(doc, [])}
        old_keys |= {k for k, c in zip(self.hot.keys, self.hot.chunks) if c["doc_path"] == doc}
        fresh = [
            c for c in new_chunks if changed & set(c["lineage_ids"]) or c["key"] not in old_keys
        ]
        self.hot.delete(sorted(old_keys - new_keys))
        self.hot.add(fresh)
        return {"doc_path": doc, "events": len(r["events"]), "hot_added": [c["key"] for c in fresh]}
