#!/usr/bin/env python3
"""D-0/R5 (supplement) — are changes made while the server is down taken?

PQ2-1's probe (pq2_1_probe.py) found a held removal never taken after a
SIGKILL and a restart. This probe asks the general question behind it: a
note edited, one deleted and one added while no server runs (stopped with
SIGTERM, then a gap), and `serve --watch` started again — are they taken at
the start? And does `rootmark ingest-dir V V` (server down) take them?
Published binary, synthetic vault; the record holds counts only.

    cd research && python3 -E -s -m d0.offline_probe --rootmark <bin> --models <dir> \
        --work <scratch dir> --out <record.json> --platform macos [--reps 3]
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import signal
import sys
import time
from pathlib import Path

from d0.pq2_1_probe import Server, events, free_port, run

GAP_S = 5.0  # between the offline changes and the restart
SETTLE_S = 10.0


def counts(v: Path):
    ev = events(v)
    out = {}
    for d in ("n1.md", "n2.md", "n9.md"):
        out[d] = {
            "events": sum(1 for e in ev if e.get("doc_path") == d),
            "deleted": sum(
                1 for e in ev if e.get("doc_path") == d and e.get("kind") == "BlockDeleted"
            ),
        }
    return out


def one(rm, work: Path, rep: int):
    v = work / f"v{rep:02d}"
    if v.exists():
        shutil.rmtree(v)
    v.mkdir(parents=True)
    for i in range(4):
        (v / f"n{i}.md").write_text(
            f"# Note {i}\n\nThe synthetic word zqoff{i} once.\n", encoding="utf-8"
        )
    run([rm, "ingest-dir", str(v), str(v)])
    run([rm, "index", "build", str(v)])
    log = work / f"v{rep:02d}.log"
    log.write_bytes(b"")
    port = free_port()
    s = Server(rm, v, port, log)
    s.wait_ready()
    s.stop(signal.SIGTERM)
    base = counts(v)
    # offline: edit n1, delete n2, add n9
    (v / "n1.md").write_text("# Note 1\n\nThe synthetic word zqedited once.\n", encoding="utf-8")
    (v / "n2.md").unlink()
    (v / "n9.md").write_text("# Note 9\n\nThe synthetic word zqnew9 once.\n", encoding="utf-8")
    time.sleep(GAP_S)
    s = Server(rm, v, port, log)
    s.wait_ready()
    time.sleep(SETTLE_S)
    at_restart = counts(v)
    s.stop(signal.SIGTERM)
    run([rm, "ingest-dir", str(v), str(v)])
    after_ingest = counts(v)

    def taken(c, d):
        if d == "n2.md":
            return c[d]["deleted"] > base[d]["deleted"]
        return c[d]["events"] > base[d]["events"]

    rec = {
        "rep": rep,
        "taken_at_restart": {
            k: taken(at_restart, d)
            for k, d in (("edit", "n1.md"), ("delete", "n2.md"), ("add", "n9.md"))
        },
        "taken_by_ingest_dir": {
            k: taken(after_ingest, d)
            for k, d in (("edit", "n1.md"), ("delete", "n2.md"), ("add", "n9.md"))
        },
        "counts": {"before": base, "at_restart": at_restart, "after_ingest_dir": after_ingest},
    }
    shutil.rmtree(v)
    return rec


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--models", required=True)
    ap.add_argument("--work", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--platform", required=True)
    ap.add_argument("--reps", type=int, default=3)
    a = ap.parse_args()
    os.environ["ROOTMARK_MODELS"] = a.models
    work = Path(a.work)
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    runs = []
    for rep in range(a.reps):
        r = one(a.rootmark, work, rep)
        runs.append(r)
        print(
            f"{a.platform} rep {rep}: restart {r['taken_at_restart']} ingest-dir {r['taken_by_ingest_dir']}",
            file=sys.stderr,
            flush=True,
        )
    summary = {
        phase: {k: sum(1 for r in runs if r[phase][k]) for k in ("edit", "delete", "add")}
        for phase in ("taken_at_restart", "taken_by_ingest_dir")
    }
    doc = {
        "probe": "D-0/R5 supplement: changes made while the server is down",
        "platform": a.platform,
        "machine": platform.machine(),
        "rootmark_version": run([a.rootmark, "--version"]).strip(),
        "parameters": {"gap_before_restart_s": GAP_S, "settle_s": SETTLE_S, "reps": a.reps},
        "commands": [
            "rootmark ingest-dir V V; rootmark index build V",
            "rootmark serve --vault V --http 127.0.0.1:PORT --watch; SIGTERM",
            "offline: edit n1.md, delete n2.md, add n9.md; wait 5 s",
            "rootmark serve … --watch (restart), settle 10 s, SIGTERM",
            "rootmark ingest-dir V V (server down)",
        ],
        "summary_of_reps": summary,
        "runs": runs,
    }
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
