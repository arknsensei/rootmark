"""P2-Q comparison of headless runs (GOAL Q-6 determinism, Q-8 Mac ↔ Linux).

    python3 -m p2q.compare <run dir> <run dir> [<run dir> ...] [--out <file>]

Each run dir holds a driver's hashes.json and queries.json. Compared, every
run against the first: events_hash; the index files (every file's blake2b;
vectors and manifest named); rankings — per query, phase and flag, the
ordered (document, chunk) list; scores — equal, or their difference named
with its distance in units in the last place (ULP, float64). A ranking or
byte difference is a difference; a score difference is reported by name
(K-P2V1.2: lexical scores' last ULP depends on the platform's libm).
Exit 1 on any byte or ranking difference.
"""

from __future__ import annotations

import argparse
import json
import struct
import sys
from pathlib import Path


def ulp_distance(a: float, b: float) -> int:
    def key(x: float) -> int:
        i = struct.unpack("<q", struct.pack("<d", x))[0]
        return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)

    return abs(key(a) - key(b))


def load(d: Path) -> dict:
    return {
        "hashes": json.loads((d / "hashes.json").read_text()),
        "queries": json.loads((d / "queries.json").read_text()),
    }


def compare(a: dict, b: dict) -> dict:
    ha, hb = a["hashes"], b["hashes"]
    files = sorted(set(ha["index_files"]) | set(hb["index_files"]))
    idx_diff = [f for f in files if ha["index_files"].get(f) != hb["index_files"].get(f)]
    rank_diff, score_diff = [], []
    for phase in ("ajan-yazdi", "son"):
        for qid, qa in a["queries"].get(phase, {}).items():
            qb = b["queries"].get(phase, {}).get(qid, {})
            for flag in ("default", "include_proposal"):
                la, lb = qa.get(flag) or [], qb.get(flag) or []
                if [(h["doc"], h["chunk_id"], h["layer"]) for h in la] != [
                    (h["doc"], h["chunk_id"], h["layer"]) for h in lb
                ]:
                    rank_diff.append(f"{phase}/{qid}/{flag}")
                    continue
                for i, (x, y) in enumerate(zip(la, lb)):
                    if x["score"] != y["score"]:
                        score_diff.append(
                            {
                                "where": f"{phase}/{qid}/{flag}/{i}",
                                "a": x["score"],
                                "b": y["score"],
                                "ulp": ulp_distance(float(x["score"]), float(y["score"])),
                            }
                        )
    return {
        "events_hash": ha["events_hash"] == hb["events_hash"],
        "vectors": ha.get("vectors") == hb.get("vectors"),
        "manifest": ha.get("manifest") == hb.get("manifest"),
        "index_files_differing": idx_diff,
        "rankings_differing": rank_diff,
        "scores_differing": score_diff,
        "max_ulp": max((s["ulp"] for s in score_diff), default=0),
        "byte_equal": ha["events_hash"] == hb["events_hash"] and not idx_diff,
        "rankings_equal": not rank_diff,
    }


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("runs", nargs="+", type=Path)
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    runs = [load(d) for d in a.runs]
    res = {"base": str(a.runs[0].name), "pairs": []}
    for d, r in zip(a.runs[1:], runs[1:]):
        c = compare(runs[0], r)
        c["other"] = str(d.name)
        res["pairs"].append(c)
    res["all_byte_equal"] = all(
        p["byte_equal"] and p["vectors"] and p["manifest"] for p in res["pairs"]
    )
    res["all_rankings_equal"] = all(p["rankings_equal"] for p in res["pairs"])
    text = json.dumps(res, indent=1, sort_keys=True)
    if a.out:
        a.out.write_text(text + "\n")
    print(text)
    return 0 if res["all_byte_equal"] and res["all_rankings_equal"] else 1


if __name__ == "__main__":
    sys.exit(main())
