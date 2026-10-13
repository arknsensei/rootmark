"""H4-1: delete `data/d3`, rebuild it from scratch with the frozen builder and
compare the new `events_hash` with `data/MANIFEST.d3.json`.

    python -m eval.run_h4 --run <RUN>
"""

from __future__ import annotations

import argparse
import json

from eval import runinfo
from eval.datasets import vault as d3
from eval.datasets.common import die, load_manifest


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    manifest = load_manifest(d3.MANIFEST)
    if d3.D3.exists():
        import shutil

        shutil.rmtree(d3.D3)
    if d3.D3.exists():
        raise RuntimeError("data/d3 could not be deleted")
    rebuilt = d3.build()  # fresh clone checkout, copy, ingest in doc_path order
    out = {
        "H4-1": {
            "manifest_hash": manifest["events_hash"],
            "rebuilt_hash": rebuilt["events_hash"],
            "equal": manifest["events_hash"] == rebuilt["events_hash"],
        },
        "docs_equal": manifest["docs"] == rebuilt["docs"],
        "rev": rebuilt["rev"],
        "stats": rebuilt["stats"],
    }
    runinfo.write_json(a.run, "h4.json", out)
    runinfo.record_phase(a.run, "h4", {}, ["h4.json"])
    # a mismatch is a hypothesis result (H4-1 fails), reported, not an error
    print(json.dumps(out["H4-1"], indent=1))


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
