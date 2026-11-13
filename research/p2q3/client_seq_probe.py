"""P2-Q.3 R4 (f), TESTS.md R3: the real client's stop sequence with a given
(published) binary — a note deleted, then SIGINT, SIGTERM, ~0.5 s, SIGKILL
if the server still runs; after a restart the note must be closed (its
`BlockDeleted` in the log, no search hit). Synthetic vault, default recipe
(`window_s` 2.0, hold 2.5 s).

    python3 -I research/p2q3/client_seq_probe.py --rootmark <bin> --work <dir> \
        --out <record.json> --platform macos [--reps 5]

The record holds counts and versions only: no content, path or host value.
"""

from __future__ import annotations

import argparse
import json
import platform
import shutil
import signal
import socket
import subprocess
import time
import urllib.request
from pathlib import Path

NOTES = 12
TARGET = "n00.md"
WORD = "zqseq00"


def ok(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True, check=False, stdin=subprocess.DEVNULL)
    if r.returncode != 0:
        raise SystemExit(
            f"failed ({r.returncode}): {' '.join(map(str, cmd[:2]))}: {r.stderr[-300:]}"
        )
    return r.stdout


def port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    n = s.getsockname()[1]
    s.close()
    return n


class Server:
    def __init__(self, rm, v: Path, log: Path):
        self.v, self.port, self.log = v, port(), log
        log.write_bytes(b"")
        with open(log, "ab") as f:
            self.p = subprocess.Popen(
                [rm, "serve", "--vault", str(v), "--http", f"127.0.0.1:{self.port}", "--watch"],
                stdout=subprocess.DEVNULL,
                stderr=f,
            )
        t0 = time.monotonic()
        while b"watch: start: reconciled" not in log.read_bytes():
            if self.p.poll() is not None:
                raise SystemExit(f"serve exited {self.p.returncode}")
            if time.monotonic() - t0 > 180:
                raise SystemExit("serve not ready")
            time.sleep(0.05)

    def hits(self) -> int:
        tok = (self.v / ".rootmark/server.token").read_text().strip()
        body = json.dumps(
            {"query": WORD, "mode": "lexical", "k": 10, "include_proposal": True}
        ).encode()
        req = urllib.request.Request(
            f"http://127.0.0.1:{self.port}/v1/search",
            data=body,
            method="POST",
            headers={
                "Authorization": f"Bearer {tok}",
                "X-Consumer": "p2q3-probe",
                "Content-Type": "application/json",
            },
        )
        with urllib.request.urlopen(req, timeout=60) as f:
            r = json.loads(f.read())
        return sum(1 for h in r.get("hits", []) if h.get("doc_path") == TARGET)


def deleted(v: Path) -> int:
    n = 0
    for line in (v / ".rootmark/events.jsonl").read_text(encoding="utf-8").splitlines():
        e = json.loads(line)
        n += e.get("kind") == "BlockDeleted" and e.get("doc_path") == TARGET
    return n


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--platform", required=True)
    ap.add_argument("--reps", type=int, default=5)
    a = ap.parse_args(argv)
    rm = a.rootmark
    base = a.work / "base"
    if base.exists():
        shutil.rmtree(base)
    base.mkdir(parents=True)
    for i in range(NOTES):
        (base / f"n{i:02d}.md").write_text(
            f"# Note {i:02d}\n\nA synthetic note with the word zqseq{i:02d}.\n\nItem {i}.\n",
            encoding="utf-8",
        )
    ok([rm, "ingest-dir", str(base), str(base)])
    ok([rm, "serve", "--vault", str(base), "--mcp", "stdio"])  # recipe and index
    runs = []
    for rep in range(a.reps):
        v = a.work / f"v{rep:02d}"
        if v.exists():
            shutil.rmtree(v)
        shutil.copytree(base, v)
        s = Server(rm, v, a.work / f"v{rep:02d}-1.log")
        hits0 = s.hits()
        (v / TARGET).unlink()
        time.sleep(0.1)
        s.p.send_signal(signal.SIGINT)
        s.p.send_signal(signal.SIGTERM)
        time.sleep(0.5)
        killed = s.p.poll() is None
        if killed:
            s.p.send_signal(signal.SIGKILL)
        s.p.wait(timeout=60)
        at_stop = deleted(v)
        s2 = Server(rm, v, a.work / f"v{rep:02d}-2.log")
        time.sleep(4.0)  # hold 2.5 s and quiet
        after = deleted(v)
        hits = s2.hits()
        s2.p.send_signal(signal.SIGTERM)
        s2.p.wait(timeout=60)
        runs.append(
            {
                "rep": rep,
                "hits_before": hits0,
                "sigkill_sent": killed,
                "taken_at_stop": at_stop > 0,
                "block_deleted_after_restart": after,
                "search_hits_after_restart": hits,
                "closed": after > 0 and hits == 0,
            }
        )
        print(f"{a.platform} rep {rep}: closed={runs[-1]['closed']} sigkill={killed}")
    rec = {
        "probe": "P2-Q.3 R4 (f) R3: SIGINT, SIGTERM, ~0.5 s, SIGKILL after a deletion",
        "platform": a.platform,
        "machine": platform.machine(),
        "rootmark_version": ok([rm, "--version"]).strip(),
        "reps": a.reps,
        "closed": sum(r["closed"] for r in runs),
        "sigkill_sent": sum(r["sigkill_sent"] for r in runs),
        "runs": runs,
    }
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(rec, indent=1) + "\n", encoding="utf-8")
    return 0 if rec["closed"] == a.reps else 1


if __name__ == "__main__":
    raise SystemExit(main())
