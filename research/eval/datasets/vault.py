"""D3 — the evaluation vault (EVAL §D3; H2, H3, H4 and the D1 source).

Clones `dendronhq/dendron-site` at a fixed rev, copies every `vault/**/*.md`
into `data/d3/docs/`, ingests them in byte-wise `doc_path` order into a fresh
`data/d3/.rootmark`, checks the acceptance bar and compares everything with
`data/MANIFEST.d3.json` (written only with `--write-manifest`).

    python -m eval.datasets.vault [--write-manifest]
"""

from __future__ import annotations

import argparse
import shutil
from pathlib import Path

import rootmark_core

from eval.datasets.common import (
    DATA,
    clone_at,
    die,
    file_blake3,
    load_manifest,
    verify_or_write_manifest,
)

REPO = "https://github.com/dendronhq/dendron-site"
REV = "fbc07769235b"
SUBDIR = "vault"
SRC = DATA / "src" / "dendron-site"
D3 = DATA / "d3"
DOCS = D3 / "docs"
MANIFEST = DATA / "MANIFEST.d3.json"
MIN_DOCS = 200
MIN_CHUNKS = 1000
MIN_WIKILINK_SHARE = 0.40


def build(src_root: Path = SRC, d3: Path = D3) -> dict:
    """Fresh D3 from the pinned source; returns the manifest content."""
    sha = clone_at(REPO, REV, src_root)
    if not sha.startswith(REV):
        raise RuntimeError(f"checked out {sha}, expected {REV}")
    if d3.exists():
        shutil.rmtree(d3)
    docs_dir = d3 / "docs"
    src = src_root / SUBDIR
    files = sorted(
        p
        for p in src.rglob("*.md")
        if not any(part.startswith(".") for part in p.relative_to(src).parts)
    )
    for p in files:
        dest = docs_dir / p.relative_to(src)
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(p, dest)
    vault = rootmark_core.Vault(str(d3))
    ingested = vault.ingest_dir(str(docs_dir))
    docs = {r["doc_path"]: file_blake3(docs_dir / r["doc_path"]) for r in ingested}
    n_chunks = 0
    with_links = 0
    for d in vault.doc_paths():
        chunks = vault.chunk(d, "structural")
        n_chunks += len(chunks)
        with_links += any(c["wikilinks"] for c in chunks)
    share = round(with_links / len(docs), 6) if docs else 0.0
    stats = {
        "docs": len(docs),
        "structural_chunks": n_chunks,
        "wikilink_docs": with_links,
        "wikilink_share": share,
    }
    if len(docs) < MIN_DOCS or n_chunks < MIN_CHUNKS or share < MIN_WIKILINK_SHARE:
        raise RuntimeError(f"D3 acceptance failed: {stats}")
    return {
        "repo": REPO,
        "rev": sha,
        "subdir": SUBDIR,
        "ingest_order": "doc_path bytewise",
        "docs": docs,
        "stats": stats,
        "events_hash": vault.events_hash(),
    }


def events_hash_from_manifest() -> str:
    return load_manifest(MANIFEST)["events_hash"]


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write-manifest", action="store_true")
    a = ap.parse_args(argv)
    manifest = build()
    verify_or_write_manifest(MANIFEST, manifest, a.write_manifest)
    print(f"D3: {manifest['stats']} events_hash={manifest['events_hash']}")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(str(e))
