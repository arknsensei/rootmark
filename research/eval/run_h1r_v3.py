"""H1-R v3 on D2 (real git history), matcher v3: the same (commit, file)
pairs as v2; for every pair the parent text, then the child text, is ingested
into a fresh vault and the surviving old lineages are counted with the v2
H1-R functions (imported through `eval.metrics_v3`). Writes `h1r_v3.json`.

    python -m eval.run_h1r_v3 --run <RUN>
"""

from __future__ import annotations

import argparse
import json
import tempfile

import rootmark_core

from eval import metrics_v3 as M3
from eval import runinfo
from eval.datasets.common import DATA, die, file_blake3, load_manifest

D2 = DATA / "d2"
MATCHER = "v3"
V2_PAIRS = 836  # pairs of the v2 run (reports/runs/20261013T154851Z-f133d0b/h1r.json)


def verify_d2() -> dict:
    m = load_manifest(DATA / "MANIFEST.d2.json")
    on_disk = {
        p.relative_to(D2).as_posix(): file_blake3(p) for p in sorted(D2.rglob("*")) if p.is_file()
    }
    if on_disk != m["files"]:
        bad = sorted(set(on_disk.items()) ^ set(m["files"].items()))[:5]
        raise RuntimeError(f"data/d2 does not match MANIFEST.d2.json (e.g. {bad})")
    return m


def pair_record(pair: dict) -> dict:
    old = (D2 / "blobs" / pair["old"]).read_bytes().decode("utf-8")
    new = (D2 / "blobs" / pair["new"]).read_bytes().decode("utf-8")
    drafts, _ = rootmark_core.parse_document(old, pair["path"])
    with tempfile.TemporaryDirectory() as tmp:
        v = rootmark_core.Vault(tmp, matcher=MATCHER)
        v.ingest(pair["path"], old, "Human", pair["commit"])
        old_blocks = v.snapshot_blocks(pair["path"])
        if [b["version_id"] for b in old_blocks] != [d["version_id"] for d in drafts]:
            raise RuntimeError(f"{pair['commit']}:{pair['path']}: snapshot/draft order mismatch")
        for b, d in zip(old_blocks, drafts, strict=True):
            b["line_start"], b["line_end"] = d["line_start"], d["line_end"]
        v.ingest(pair["path"], new, "Human", pair["commit"])
        snap = v.snapshot(pair["path"])
        new_blocks = v.snapshot_blocks(pair["path"])
    return M3.h1r_pair_record(
        old_blocks, pair["hunks"], [lin for lin, _ in snap["blocks"]], new_blocks
    )


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    a = ap.parse_args(argv)
    m = verify_d2()
    pairs = json.loads((D2 / "pairs.json").read_text())
    if len(pairs) != m["n_pairs"] or len(pairs) != V2_PAIRS:
        raise RuntimeError(f"{len(pairs)} pairs (manifest {m['n_pairs']}, v2 run {V2_PAIRS})")
    records = [pair_record(p) for p in pairs]
    out = {
        "matcher": MATCHER,
        "repo": m["repo"],
        "rev": m["rev"],
        "commits": len(m["commits"]),
        "pairs": len(pairs),
        **M3.h1r_aggregate(records, pairs),
    }
    runinfo.write_json(a.run, "h1r_v3.json", out)
    runinfo.record_phase(
        a.run,
        "h1r_v3",
        {"matcher": MATCHER, "theta": rootmark_core.matcher_defaults(MATCHER)[0]},
        ["h1r_v3.json"],
    )
    print(
        json.dumps(
            {k: out[k] for k in ("H1-R1", "H1-R2", "touched_retention", "loss_categories")},
            indent=1,
        )
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
