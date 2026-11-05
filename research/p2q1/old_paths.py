"""P2-Q.1/R4 (Q1-3): live blocks left at the old paths of the Obsidian
scene's deleted and renamed notes, after the scene (the vault of its work
directory, server stopped): `rootmark blame` of each old path. Records
only `k2:` refs (synthetic paths in the clear) and counts.

    P2Q_PATH_KEY_FILE=<key> python -m p2q1.old_paths --vault <work>/vault --snapshot <snapshot> --rootmark <bin> --out <json>
"""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from p2q.common import k2, list_md, path_key

HERE = Path(__file__).resolve().parent
SYNTHETIC = "pilot/"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--vault", required=True, type=Path)
    ap.add_argument("--snapshot", required=True, type=Path)
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args(argv)
    key = path_key()
    o = json.loads((HERE / "scenario.json").read_text(encoding="utf-8"))["obsidian"]
    by_ref = {k2(p, key): p for p in list_md(a.snapshot)}

    def path_of(e: dict) -> str:
        if "doc_path" in e:
            return e["doc_path"]
        return by_ref[e["doc_ref"]]

    olds = [("delete", path_of(e)) for e in o["edits"] if e["op"] == "delete"]
    olds += [("rename", by_ref[r["doc_ref"]]) for r in o["renames"]]
    rows = []
    for kind, p in olds:
        r = subprocess.run([a.rootmark, "blame", str(a.vault), p], capture_output=True, text=True)
        live = len(json.loads(r.stdout)["blocks"]) if r.returncode == 0 else None
        rows.append(
            {
                "kind": kind,
                "old_path": p if p.startswith(SYNTHETIC) else k2(p, key),
                "file_present": (a.vault / p).exists(),
                "blame_exit": r.returncode,
                "live_blocks": live,
            }
        )
    out = {
        "rows": rows,
        "live_blocks_total": sum(x["live_blocks"] or 0 for x in rows),
        "all_blamed": all(x["blame_exit"] == 0 for x in rows),
    }
    a.out.write_text(json.dumps(out, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({k: out[k] for k in ("live_blocks_total", "all_blamed")}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
