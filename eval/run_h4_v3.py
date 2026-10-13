"""H4-1 under matcher v3 (T26; `make eval-h4`): delete `data/d3`, rebuild it
from scratch with the frozen v2 builder (under matcher v2, its log must still
equal `MANIFEST.d3.json:events_hash`), then ingest the rebuilt documents from
scratch with matcher v3 and compare that log's hash with
`MANIFEST.d3.json:events_hash_v3`. Writes `h4_v3.json`.

    python -m eval.run_h4_v3 --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import shutil
import tempfile
from pathlib import Path

from eval import runinfo
from eval.d3_v3 import V3_FIELD, build_v2, ingest_docs, v2_fields
from eval.datasets import vault as d3
from eval.datasets.common import die, load_manifest

MATCHER = "v3"


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    manifest = load_manifest(d3.MANIFEST)
    if V3_FIELD not in manifest:
        raise RuntimeError(f"MANIFEST.d3.json has no {V3_FIELD}")
    if d3.D3.exists():
        shutil.rmtree(d3.D3)
    if d3.D3.exists():
        raise RuntimeError("data/d3 could not be deleted")
    rebuilt = build_v2()  # fresh clone checkout, copy, ingest (matcher v2)
    with tempfile.TemporaryDirectory() as t:
        v3_hash = ingest_docs(Path(t), MATCHER).events_hash()
    out = {
        "matcher": MATCHER,
        "H4-1": {
            "manifest_hash": manifest[V3_FIELD],
            "rebuilt_hash": v3_hash,
            "equal": manifest[V3_FIELD] == v3_hash,
        },
        "v2_log": {
            "manifest_hash": manifest["events_hash"],
            "rebuilt_hash": rebuilt["events_hash"],
            "equal": manifest["events_hash"] == rebuilt["events_hash"],
        },
        "v2_fields_equal": v2_fields(manifest) == rebuilt,
        "docs_equal": manifest["docs"] == rebuilt["docs"],
        "rev": rebuilt["rev"],
        "stats": rebuilt["stats"],
    }
    runinfo.write_json(a.run, "h4_v3.json", out)
    runinfo.record_phase(a.run, "h4_v3", {"matcher": MATCHER}, ["h4_v3.json"])
    # a mismatch is a hypothesis result (H4-1 fails), reported, not an error
    print(json.dumps({"H4-1": out["H4-1"], "v2_log": out["v2_log"]}, indent=1))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
