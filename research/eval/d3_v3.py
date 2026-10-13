"""D3 under matcher v3 (EVAL_DELTA §Koşturma; T26).

D3 itself is still built by the frozen v2 builder `eval.datasets.vault` —
under matcher v2, so its log reproduces `MANIFEST.d3.json:events_hash`
(H4-V2). The v3 log of the same documents is a fresh ingest of
`data/d3/docs` into a separate vault with matcher v3; its hash is the
`events_hash_v3` field that T26 adds to `MANIFEST.d3.json` (the v2 fields
are never rewritten).

    python -m eval.d3_v3 [--write-v3]      # what `make d3` runs
"""

from __future__ import annotations

import argparse
import os
import tempfile
from contextlib import contextmanager
from pathlib import Path

import mdkernel_core

from eval.datasets import vault as d3
from eval.datasets.common import ROOT, die, dump_json, load_manifest

V3_FIELD = "events_hash_v3"


@contextmanager
def matcher_env(name: str):
    """Default matcher for code that never passes one (the frozen v2 builders)."""
    old = os.environ.get("MDK_MATCHER")
    os.environ["MDK_MATCHER"] = name
    try:
        yield
    finally:
        if old is None:
            del os.environ["MDK_MATCHER"]
        else:
            os.environ["MDK_MATCHER"] = old


def build_v2() -> dict:
    """Fresh `data/d3` with the frozen builder, under matcher v2."""
    with matcher_env("v2"):
        return d3.build()


def ingest_docs(root: Path, matcher: str, docs: Path = d3.DOCS) -> mdkernel_core.Vault:
    """Fresh vault at `root` holding every D3 document, ingested with `matcher`."""
    v = mdkernel_core.Vault(str(root), matcher=matcher)
    if v.doc_paths():
        raise RuntimeError(f"{root} is not empty")
    v.ingest_dir(str(docs))
    return v


def v3_events_hash() -> str:
    with tempfile.TemporaryDirectory() as t:
        return ingest_docs(Path(t), "v3").events_hash()


def v2_fields(manifest: dict) -> dict:
    return {k: v for k, v in manifest.items() if k != V3_FIELD}


def verify_docs(manifest: dict) -> None:
    """Fail-fast: `data/d3/docs` must hold exactly the manifest's documents."""
    from eval.datasets.common import file_blake3

    on_disk = {
        p.relative_to(d3.DOCS).as_posix(): file_blake3(p) for p in sorted(d3.DOCS.rglob("*.md"))
    }
    if on_disk != manifest["docs"]:
        raise RuntimeError("data/d3/docs does not match MANIFEST.d3.json")


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write-v3", action="store_true", help=f"add/replace `{V3_FIELD}`")
    a = ap.parse_args(argv)
    current = load_manifest(d3.MANIFEST)
    rebuilt = build_v2()
    if rebuilt != v2_fields(current):
        diffs = sorted(k for k in rebuilt if rebuilt[k] != current.get(k))
        raise RuntimeError(f"MANIFEST.d3.json (v2 fields) mismatch in {diffs}")
    h3 = v3_events_hash()
    if a.write_v3:
        d3.MANIFEST.write_text(dump_json({**current, V3_FIELD: h3}), encoding="utf-8")
        print(f"wrote {V3_FIELD}={h3} into {d3.MANIFEST.relative_to(ROOT)}")
    elif V3_FIELD not in current:
        raise RuntimeError(f"MANIFEST.d3.json has no {V3_FIELD}; run with --write-v3 once")
    elif current[V3_FIELD] != h3:
        raise RuntimeError(f"{V3_FIELD} {h3} != MANIFEST.d3.json {current[V3_FIELD]}")
    print(f"data/MANIFEST.d3.json verified: events_hash={rebuilt['events_hash']} {V3_FIELD}={h3}")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
