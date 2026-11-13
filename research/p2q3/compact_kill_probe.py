"""P2-Q.3 R5 (finding PQ3-7): does a SIGKILL during `serve --watch`'s shutdown
compaction leave an index that the next start serves but cannot search?

Synthetic vault, default recipe. Each repetition: start the server, append a
paragraph to a note (the hot tier gets a record, so the shutdown compacts),
wait until it is ingested, SIGTERM, then SIGKILL after a delay that sweeps
0–400 ms; restart and call `search` once. Counts only.

    python3 -I research/p2q3/compact_kill_probe.py --rootmark <bin> --work <dir> \
        --out <record.json> [--reps 40]
"""

from __future__ import annotations

import argparse
import json
import shutil
import signal
import socket
import subprocess
import time
import urllib.error
import urllib.request
from pathlib import Path


def ok(cmd):
    r = subprocess.run(cmd, capture_output=True, text=True, check=False, stdin=subprocess.DEVNULL)
    if r.returncode != 0:
        raise SystemExit(f"failed: {' '.join(map(str, cmd[:2]))}: {r.stderr[-300:]}")
    return r.stdout


def port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    n = s.getsockname()[1]
    s.close()
    return n


def start(rm, v: Path, log: Path):
    p_ = port()
    f = open(log, "wb")  # noqa: SIM115
    p = subprocess.Popen(
        [rm, "serve", "--vault", str(v), "--http", f"127.0.0.1:{p_}", "--watch"],
        stdout=subprocess.DEVNULL,
        stderr=f,
    )
    t0 = time.monotonic()
    while b"rootmark serve: watching" not in log.read_bytes():
        if p.poll() is not None:
            raise SystemExit("serve exited")
        if time.monotonic() - t0 > 120:
            raise SystemExit("not ready")
        time.sleep(0.05)
    time.sleep(1.0)
    return p, p_, f


def search(v: Path, p_: int) -> int:
    tok = (v / ".rootmark/server.token").read_text().strip()
    req = urllib.request.Request(
        f"http://127.0.0.1:{p_}/v1/search",
        data=json.dumps({"query": "synthetic", "mode": "lexical", "k": 3}).encode(),
        method="POST",
        headers={
            "Authorization": f"Bearer {tok}",
            "X-Consumer": "probe",
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return r.status
    except urllib.error.HTTPError as e:
        return e.code


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--reps", type=int, default=40)
    a = ap.parse_args(argv)
    rm = a.rootmark
    base = a.work / "base"
    if base.exists():
        shutil.rmtree(base)
    base.mkdir(parents=True)
    for i in range(30):
        (base / f"n{i:02d}.md").write_text(
            f"# Note {i}\n\nA synthetic note number {i}.\n\nItem {i} of the probe.\n",
            encoding="utf-8",
        )
    ok([rm, "ingest-dir", str(base), str(base)])
    ok([rm, "serve", "--vault", str(base), "--mcp", "stdio"])
    runs = []
    for rep in range(a.reps):
        v = a.work / f"v{rep:02d}"
        if v.exists():
            shutil.rmtree(v)
        shutil.copytree(base, v)
        p, _, f = start(rm, v, a.work / "a.log")
        with open(v / "n00.md", "a", encoding="utf-8") as h:
            h.write(f"\nAn added paragraph {rep}.\n")
        time.sleep(1.5)
        delay = (rep % 9) * 0.05
        p.send_signal(signal.SIGTERM)
        time.sleep(delay)
        killed = p.poll() is None
        if killed:
            p.send_signal(signal.SIGKILL)
        p.wait()
        f.close()
        p2, port2, f2 = start(rm, v, a.work / "b.log")
        st = search(v, port2)
        p2.send_signal(signal.SIGTERM)
        p2.wait(timeout=60)
        f2.close()
        runs.append({"delay_ms": int(delay * 1000), "sigkill_sent": killed, "search_status": st})
        shutil.rmtree(v)
    rec = {
        "probe": "P2-Q.3 PQ3-7: SIGKILL during the shutdown compaction",
        "rootmark_version": ok([rm, "--version"]).strip(),
        "reps": a.reps,
        "sigkill_sent": sum(r["sigkill_sent"] for r in runs),
        "search_not_200_after_restart": sum(r["search_status"] != 200 for r in runs),
        "runs": runs,
    }
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(rec, indent=1) + "\n", encoding="utf-8")
    print(
        json.dumps(
            {
                k: rec[k]
                for k in ("rootmark_version", "sigkill_sent", "search_not_200_after_restart")
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
