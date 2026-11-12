#!/usr/bin/env python3
"""P2-Q.3 P1/P2 (docs/rounds/P2-Q.3/progress/TESTS.md, DESIGN §7): the start's
reconciliation time of `rootmark serve --watch` on a copy of D-NOTES.

    python3 -I research/p2q3/perf_start.py --rootmark <bin> --notes <d_notes/notes> \
        --work <scratch dir> --out <record.json> --platform macos

P1: `ingest-dir` + `index build`, then `serve --watch`: the summary line's time
with nothing changed (threshold on Mac arm64: <= 10 s). P2: one note edited while
the server is down, then a restart (information). The record holds counts and
times only: no note content, no path, no host name.
"""

from __future__ import annotations

import argparse
import json
import platform
import re
import shutil
import signal
import socket
import subprocess
import time
from pathlib import Path

LINE = re.compile(
    r"watch: start: reconciled (\d+) files with the log "
    r"\((\d+) changed, (\d+) removals held, (\d+) errors\) in (\d+) ms"
)
# the default recipe is `canonical` with `window_s` 2.0; the human is added
RECIPE_TAIL = '\n[identity]\nhuman_id = "perf"\n'


def run(cmd):
    p = subprocess.run(cmd, capture_output=True, text=True, check=False, stdin=subprocess.DEVNULL)
    if p.returncode != 0:
        raise SystemExit(
            f"failed ({p.returncode}): {' '.join(map(str, cmd[:2]))}: {p.stderr[-400:]}"
        )
    return p.stdout


def port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    n = s.getsockname()[1]
    s.close()
    return n


def start(rm, v: Path, log: Path) -> dict:
    log.write_bytes(b"")
    t0 = time.monotonic()
    with open(log, "ab") as f:
        p = subprocess.Popen(
            [rm, "serve", "--vault", str(v), "--http", f"127.0.0.1:{port()}", "--watch"],
            stdout=subprocess.DEVNULL,
            stderr=f,
        )
    m = None
    while time.monotonic() - t0 < 600:
        if p.poll() is not None:
            raise SystemExit(f"serve exited {p.returncode}")
        m = LINE.search(log.read_text(errors="replace"))
        if m:
            break
        time.sleep(0.05)
    if not m:
        p.kill()
        raise SystemExit("no reconciliation line")
    wall = time.monotonic() - t0
    time.sleep(3.0)  # held removals (none expected) and quiet
    p.send_signal(signal.SIGTERM)
    rc = p.wait(timeout=120)
    files, changed, held, errors, ms = map(int, m.groups())
    return {
        "files": files,
        "changed": changed,
        "removals_held": held,
        "errors": errors,
        "reconcile_ms": ms,
        "start_to_line_s": round(wall, 3),
        "sigterm_rc": rc,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--notes", required=True, type=Path)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--platform", required=True)
    a = ap.parse_args()
    v = a.work / "vault"
    if v.exists():
        shutil.rmtree(v)
    shutil.copytree(a.notes, v)
    rm = a.rootmark
    t = time.monotonic()
    run([rm, "ingest-dir", str(v), str(v)])
    ingest_s = time.monotonic() - t
    # `rootmark serve` writes the default recipe and builds the index, then
    # ends at the end of stdin; then the identity is added
    t = time.monotonic()
    run([rm, "serve", "--vault", str(v), "--mcp", "stdio"])
    build_s = time.monotonic() - t
    pol = v / ".rootmark" / "policy.toml"
    pol.write_text(pol.read_text() + RECIPE_TAIL, encoding="utf-8")
    eh0 = json.loads(run([rm, "events-hash", str(v)]))["events_hash"]
    p1 = start(rm, v, a.work / "p1.log")
    eh1 = json.loads(run([rm, "events-hash", str(v)]))["events_hash"]
    # P2: one note edited while down
    notes = (
        x for x in v.rglob("*.md") if not any(c.startswith(".") for c in x.relative_to(v).parts)
    )
    first = min(notes, key=lambda x: str(x.relative_to(v)).encode())
    text = first.read_text(encoding="utf-8")
    first.write_text(text + "\n\nA paragraph added while the server was down.\n", encoding="utf-8")
    time.sleep(1.5)
    p2 = start(rm, v, a.work / "p2.log")
    eh2 = json.loads(run([rm, "events-hash", str(v)]))["events_hash"]
    rec = {
        "probe": "P2-Q.3 P1/P2: the start's reconciliation on a D-NOTES copy",
        "platform": a.platform,
        "machine": platform.machine(),
        "rootmark_version": run([rm, "--version"]).strip(),
        "documents": p1["files"],
        "ingest_dir_s": round(ingest_s, 3),
        "serve_first_start_with_index_build_s": round(build_s, 3),
        "P1_unchanged": p1,
        "P1_events_hash_equal": eh0 == eh1,
        "P1_threshold_s": 10.0,
        "P1_pass": p1["reconcile_ms"] <= 10000 and p1["changed"] == 0 and eh0 == eh1,
        "P2_one_edit": p2,
        "P2_events_hash_changed": eh2 != eh1,
    }
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(rec, indent=1) + "\n", encoding="utf-8")
    print(json.dumps({k: rec[k] for k in ("P1_unchanged", "P1_pass", "P2_one_edit")}))
    return 0 if rec["P1_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
