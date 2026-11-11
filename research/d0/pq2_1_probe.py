#!/usr/bin/env python3
"""D-0/R5 — PQ2-1 probe (docs/rounds/D-0/progress/DESIGN.md §4).

Does a removal the watcher still holds (a note deleted or moved away less
than `window_s` + 0.5 s before the server is SIGKILLed) get taken by the
next `serve --watch` start? The product is not changed: the published
binary is run as it is, on a synthetic vault (no personal content).

    python3 -I research/d0/pq2_1_probe.py --rootmark <bin> --models <dir> \
        --work <scratch dir> --out <record.json> --platform macos [--reps 10]

The record holds counts, versions, commands and times only: no note
content, no absolute path, no host name.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import signal
import socket
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

NOTES = 20
KILL_AFTER_S = 0.5  # < window_s + 0.5 s = 2.5 s (default recipe)
TERM_AFTER_S = 5.0  # > 2.5 s: the control's removal is due before SIGTERM
SETTLE_S = 10.0  # after the restart is ready (hold x 4)
TARGET = "n00.md"
MOVED = "sub/n00-moved.md"
TOKEN = "zqprobe00"  # the target note's unique word


def make_vault(v: Path):
    (v / "sub").mkdir(parents=True)
    for i in range(NOTES):
        body = (
            f"# Note {i:02d}\n\nThis synthetic note carries the word zqprobe{i:02d} once.\n\n"
            f"A second paragraph about topic {i % 5} and item {i}.\n"
        )
        (v / f"n{i:02d}.md").write_text(body, encoding="utf-8")
    (v / "sub" / "keep.md").write_text(
        "# Keep\n\nA note that keeps the sub directory.\n", encoding="utf-8"
    )


def run(cmd, **kw):
    p = subprocess.run(cmd, check=False, capture_output=True, text=True, **kw)
    if p.returncode != 0:
        raise SystemExit(
            f"command failed ({p.returncode}): {' '.join(map(str, cmd[:3]))} …: {p.stderr[-400:]}"
        )
    return p.stdout


def free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


class Server:
    def __init__(self, rm, vault: Path, port: int, log: Path):
        self.vault, self.port = vault, port
        self.logf = open(log, "ab")  # noqa: SIM115 (closed in stop())
        self.p = subprocess.Popen(
            [rm, "serve", "--vault", str(vault), "--http", f"127.0.0.1:{port}", "--watch"],
            stdout=subprocess.DEVNULL,
            stderr=self.logf,
        )
        self.log = log

    def call(self, tool, body=None):
        tok = (self.vault / ".rootmark" / "server.token").read_text().strip()
        url = f"http://127.0.0.1:{self.port}/v1/{tool}"
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(
            url,
            data=data,
            method="POST" if data else "GET",
            headers={
                "Authorization": f"Bearer {tok}",
                "X-Consumer": "d0-probe",
                "Content-Type": "application/json",
            },
        )
        with urllib.request.urlopen(req, timeout=60) as f:
            return json.loads(f.read())

    def wait_ready(self, timeout=180):
        t0 = time.monotonic()
        while time.monotonic() - t0 < timeout:
            if self.p.poll() is not None:
                raise SystemExit(f"server exited early ({self.p.returncode})")
            if b"rootmark serve: watching" in self.log.read_bytes():
                try:
                    self.call("status")
                    return time.monotonic() - t0
                except OSError:  # not answering yet (URLError is an OSError)
                    time.sleep(0.05)
                    continue
            time.sleep(0.05)
        raise SystemExit("server not ready in time")

    def stop(self, sig):
        self.p.send_signal(sig)
        try:
            self.p.wait(timeout=60)
        except subprocess.TimeoutExpired:
            self.p.kill()
            self.p.wait()
        self.logf.close()
        return self.p.returncode


def events(v: Path):
    f = v / ".rootmark" / "events.jsonl"
    return [json.loads(x) for x in f.read_text(encoding="utf-8").splitlines() if x.strip()]


def deleted_for(v: Path, path: str) -> int:
    return sum(
        1 for e in events(v) if e.get("kind") == "BlockDeleted" and e.get("doc_path") == path
    )


def hits_for(srv: Server, path: str) -> int:
    r = srv.call("search", {"query": TOKEN})
    hits = r.get("results") or r.get("hits") or []
    return sum(1 for h in hits if h.get("doc_path") == path)


def one(rm, base: Path, work: Path, op: str, stop: str, rep: int):
    v = work / f"{op}-{stop}-{rep:02d}"
    if v.exists():
        shutil.rmtree(v)
    shutil.copytree(base, v)
    log = work / f"{op}-{stop}-{rep:02d}.log"
    log.write_bytes(b"")
    port = free_port()
    srv = Server(rm, v, port, log)
    ready1 = srv.wait_ready()
    del0 = deleted_for(v, TARGET)
    hits0 = hits_for(srv, TARGET)
    t_op = time.monotonic()
    if op == "delete":
        (v / TARGET).unlink()
    else:
        (v / TARGET).rename(v / MOVED)
    if stop == "kill":
        time.sleep(KILL_AFTER_S)
        del_before_stop = deleted_for(v, TARGET)
        stop_at = time.monotonic() - t_op
        rc1 = srv.stop(signal.SIGKILL)
    else:
        time.sleep(TERM_AFTER_S)
        del_before_stop = deleted_for(v, TARGET)
        stop_at = time.monotonic() - t_op
        rc1 = srv.stop(signal.SIGTERM)
    del_after_stop = deleted_for(v, TARGET)
    srv2 = Server(rm, v, port, log)
    ready2 = srv2.wait_ready()
    time.sleep(SETTLE_S)
    del_after_restart = deleted_for(v, TARGET)
    hits_after = hits_for(srv2, TARGET)
    moved_hits = hits_for(srv2, MOVED) if op == "move" else None
    status = srv2.call("status")
    workaround = None
    if rep == 0 and stop == "kill" and hits_after > 0:
        # a stop-gap a user could apply: write the old path again, then remove it under the watcher
        (v / TARGET).write_text("# Note 00\n\nrewritten\n", encoding="utf-8")
        time.sleep(3.0)
        (v / TARGET).unlink()
        time.sleep(SETTLE_S)
        workaround = {
            "deleted_after": deleted_for(v, TARGET),
            "search_hits_after": hits_for(srv2, TARGET),
        }
    log_sha = hashlib.sha256((v / ".rootmark" / "events.jsonl").read_bytes()).hexdigest()
    n_events = len(events(v))
    rc2 = srv2.stop(signal.SIGTERM)
    # recovery path (one per op x stop, rep 0): a re-ingest with the server down
    reingest = None
    if rep == 0 and stop == "kill":
        out = json.loads(run([rm, "ingest-dir", str(v), str(v)]))
        reingest = {
            "deleted_after_reingest": deleted_for(v, TARGET),
            "events_hash": out.get("events_hash"),
        }
    if stop == "term":
        # control: the removal is due before SIGTERM and must be taken then
        cls = "control_taken" if del_before_stop > 0 and hits_after == 0 else "control_failed"
    elif del_before_stop > 0:
        cls = "other"  # the removal was taken before the kill: not a probe of PQ2-1
    elif del_after_restart > 0 and hits_after == 0:
        cls = "recovered"
    elif del_after_restart == 0 and hits_after > 0:
        cls = "persistent"
    else:
        cls = "other"
    rec = {
        "op": op,
        "stop": stop,
        "rep": rep,
        "class": cls,
        "stop_after_op_s": round(stop_at, 3),
        "exit_code_stop": rc1,
        "exit_code_final": rc2,
        "ready_s": [round(ready1, 3), round(ready2, 3)],
        "target_blocks_deleted": {
            "before_op": del0,
            "before_stop": del_before_stop,
            "after_stop": del_after_stop,
            "after_restart_settle": del_after_restart,
        },
        "search_hits_target": {"before_op": hits0, "after_restart_settle": hits_after},
        "search_hits_moved_path": moved_hits,
        "events_jsonl_sha256": log_sha,
        "events_count": n_events,
        "workaround_rewrite_then_delete": workaround,
        "status_index_state": (status.get("index") or {}).get("state"),
        "reingest_with_server_down": reingest,
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
    ap.add_argument("--reps", type=int, default=10)
    a = ap.parse_args()
    os.environ["ROOTMARK_MODELS"] = a.models
    rm = a.rootmark
    work = Path(a.work)
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    base = work / "base"
    make_vault(base)
    t0 = time.time()
    ing = json.loads(run([rm, "ingest-dir", str(base), str(base)]))
    idx = json.loads(run([rm, "index", "build", str(base)]))
    runs = []
    for rep in range(a.reps):
        for op in ("delete", "move"):
            for stop in ("kill", "term"):
                r = one(rm, base, work, op, stop, rep)
                runs.append(r)
                print(
                    f"{a.platform} rep {rep} {op}/{stop}: {r['class']}", file=sys.stderr, flush=True
                )
    counts = {}
    for r in runs:
        k = f"{r['op']}/{r['stop']}"
        counts.setdefault(k, {})
        counts[k][r["class"]] = counts[k].get(r["class"], 0) + 1
    doc = {
        "probe": "D-0/R5 PQ2-1 (docs/rounds/D-0/progress/DESIGN.md §4)",
        "platform": a.platform,
        "machine": platform.machine(),
        "system": platform.system(),
        "rootmark_version": run([rm, "--version"]).strip(),
        "spec_version": run([rm, "--spec-version"]).strip(),
        "vault": {
            "notes": NOTES + 1,
            "synthetic": True,
            "base_events_hash": ing.get("events_hash"),
            "embedder_id": idx.get("embedder_id"),
        },
        "parameters": {
            "kill_after_op_s": KILL_AFTER_S,
            "term_after_op_s": TERM_AFTER_S,
            "settle_after_restart_s": SETTLE_S,
            "hold_default_s": 2.5,
            "reps": a.reps,
            "target": TARGET,
            "moved_to": MOVED,
        },
        "commands": [
            "rootmark ingest-dir V V",
            "rootmark index build V",
            "rootmark serve --vault V --http 127.0.0.1:PORT --watch",
            "op: unlink n00.md | rename n00.md -> sub/n00-moved.md",
            "kill: SIGKILL 0.5 s after op | term: SIGTERM 5 s after op",
            "rootmark serve --vault V --http 127.0.0.1:PORT --watch (restart), settle 10 s",
            "observe: BlockDeleted for n00.md in events.jsonl; POST /v1/search {query: zqprobe00}",
            "rep 0 of each kill case: SIGTERM, then rootmark ingest-dir V V (server down)",
        ],
        "duration_s": round(time.time() - t0, 1),
        "counts": counts,
        "runs": runs,
    }
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    print(json.dumps(counts))


if __name__ == "__main__":
    main()
