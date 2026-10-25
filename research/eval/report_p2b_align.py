"""P2-B/B2 report (not a measurer): the alignment error of D-NOTES' real
notes per note class, from the official `p2b/align.json` (its failing
documents and their bad blocks) and each note's block count (`rootmark
replay`, the same ingest as the measurer). Writes JSON to stdout.

    python -m eval.report_p2b_align <align.json> <rootmark>
"""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from eval import p2b_align
from eval.datasets import d_notes

align, rootmark = json.loads(Path(sys.argv[1]).read_text()), sys.argv[2]
m = json.loads(d_notes.MANIFEST.read_text(encoding="utf-8"))
bad = {f["doc"]: f["bad"] for f in align["notes"]["failing_docs"]}
blocks: dict[str, int] = {}
with tempfile.TemporaryDirectory() as t:
    v = Path(t) / "vault"
    for n in m["notes"]:
        f = v / p2b_align.vault_path(n["path"])
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_bytes((d_notes.NOTES / n["path"]).read_bytes())
    subprocess.run([rootmark, "ingest-dir", str(v), str(v)], check=True, capture_output=True)
    for n in m["notes"]:
        vp = p2b_align.vault_path(n["path"])
        r = subprocess.run([rootmark, "replay", str(v), vp], capture_output=True, check=True)
        blocks[vp] = len(json.loads(r.stdout)["blocks"])
out = {}
for c in list(d_notes.CLASSES) + ["turkish", "all"]:
    ns = [
        n
        for n in m["notes"]
        if c == "all" or (c == "turkish" and n["turkish"]) or c in n["classes"]
    ]
    b = sum(blocks[p2b_align.vault_path(n["path"])] for n in ns)
    e = sum(bad.get(p2b_align.vault_path(n["path"]), 0) for n in ns)
    out[c] = {"notes": len(ns), "blocks": b, "bad": e, "error": e / b if b else 0.0}
if out["all"]["bad"] != align["notes"]["bad"] or out["all"]["blocks"] != align["notes"]["blocks"]:
    sys.exit("report_p2b_align: totals differ from align.json")
print(json.dumps(out, indent=1, sort_keys=True))
