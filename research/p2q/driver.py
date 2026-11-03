"""P2-Q headless scene driver (GOAL Q2 (b), (d), Q3; scenario.json `headless`).

Runs steps H01–H23 of the pre-registered scenario on a fresh copy of one
corpus with the installed `rootmark` (command line, MCP over stdio, HTTP —
never the Python binding), the way docs/GUIDE.md §2–§4 prepares and runs a
vault. Python standard library only: the same file runs on the Mac, in the
linux/amd64 container and in CI.

    python3 -m p2q.driver --corpus qorbit|dnotes --snapshot <dir> --work <dir>
        --out <records dir> [--rootmark rootmark] [--label run1]

ROOTMARK_MODELS must be set (as the guide says). For a personal corpus
P2Q_PATH_KEY_FILE must name the path key (scenario.json `personal_paths`).
Records (no text of a personal corpus; a personal path only as `k2:`):
<out>/steps.json (every step: pass and details), queries.json (every
phase's hits), hashes.json (events_hash, index files, query results),
numbers.json (times, latencies, memory, disk), observe.json and
observe_compare.json. Logs and the vault copy stay in <work>.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import signal
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request
from pathlib import Path

from p2q.common import b2, corpus_summary, dump, fresh_copy, k2, list_md, load, path_key, tree_b2
from p2q.evaluate import check_phase, default_leaks

HERE = Path(__file__).resolve().parent
SYNTHETIC = ("toplayici/", "yorum/", "ajan-dogrudan/")
PROTO = "2025-06-18"


def cpu_model() -> str:
    """The processor's model name (Linux /proc/cpuinfo, macOS sysctl)."""
    ci = Path("/proc/cpuinfo")
    if ci.exists():
        m = re.search(r"^model name\s*:\s*(.+)$", ci.read_text(), re.MULTILINE)
        if m:
            return m.group(1).strip()
    r = subprocess.run(
        ["sysctl", "-n", "machdep.cpu.brand_string"], capture_output=True, text=True, check=False
    )
    return r.stdout.strip() or platform.processor()


def pct(xs: list[float], p: float) -> float | None:
    if not xs:
        return None
    s = sorted(xs)
    i = min(len(s) - 1, max(0, round(p / 100 * (len(s) - 1))))
    return round(s[i], 3)


class RSS:
    """Peak resident set size of a process: Linux VmHWM, else `ps` sampling."""

    def __init__(self, pid: int):
        self.pid, self.peak, self.stop = pid, 0, False
        self.t = threading.Thread(target=self._run, daemon=True)
        self.t.start()

    def _sample(self) -> int:
        st = Path(f"/proc/{self.pid}/status")
        if st.exists():
            m = re.search(r"VmHWM:\s+(\d+)", st.read_text())
            return int(m.group(1)) if m else 0
        r = subprocess.run(
            ["ps", "-o", "rss=", "-p", str(self.pid)], capture_output=True, text=True, check=False
        )
        return int(r.stdout.strip() or 0)

    def _run(self):
        while not self.stop:
            try:
                self.peak = max(self.peak, self._sample())
            except (OSError, ValueError):
                pass
            time.sleep(0.2)

    def done(self) -> int:
        self.stop = True
        self.t.join(timeout=2)
        return self.peak


class Server:
    """`rootmark serve --mcp stdio --http 127.0.0.1:0 --watch` with an MCP
    client on its stdio and an HTTP client on its port."""

    def __init__(self, rootmark: str, vault: Path, log: Path, attest_log: Path, client: dict):
        self.err = open(log, "ab")  # noqa: SIM115 (open while the server runs)
        self.p = subprocess.Popen(
            [
                rootmark,
                "serve",
                "--vault",
                str(vault),
                "--mcp",
                "stdio",
                "--http",
                "127.0.0.1:0",
                "--watch",
                "--attest-log",
                str(attest_log),
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self.err,
        )
        self.vault, self.log, self.n = vault, log, 0
        self.rss = RSS(self.p.pid)
        self.port = None
        for _ in range(1200):
            m = re.search(rb"http listening on 127\.0\.0\.1:(\d+)", log.read_bytes())
            if m:
                self.port = int(m.group(1))
                break
            if self.p.poll() is not None:
                raise RuntimeError(f"serve exited {self.p.returncode}")
            time.sleep(0.1)
        if self.port is None:
            raise RuntimeError("serve did not listen")
        self.token = (vault / ".rootmark/server.token").read_text().strip()
        self.rpc("initialize", {"protocolVersion": PROTO, "capabilities": {}, "clientInfo": client})
        self._send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def _send(self, obj):
        self.p.stdin.write((json.dumps(obj) + "\n").encode())
        self.p.stdin.flush()

    def rpc(self, method, params):
        self.n += 1
        self._send({"jsonrpc": "2.0", "id": self.n, "method": method, "params": params})
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError("MCP stream closed")
            msg = json.loads(line)
            if msg.get("id") == self.n:
                if "error" in msg:
                    raise RuntimeError(f"MCP {method}: {msg['error']}")
                return msg["result"]

    def mcp(self, tool, args):
        r = self.rpc("tools/call", {"name": tool, "arguments": args})
        sc = r.get("structuredContent")
        if r.get("isError"):
            return {"_error": sc if sc is not None else r.get("content")}
        return sc

    def http(self, tool, body=None, consumer="pilot"):
        url = f"http://127.0.0.1:{self.port}/v1/{tool}"
        data = None if body is None else json.dumps(body).encode()
        req = urllib.request.Request(url, data=data, method="GET" if body is None else "POST")
        req.add_header("Authorization", f"Bearer {self.token}")
        req.add_header("X-Consumer", consumer)
        req.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(req, timeout=300) as r:
                return json.loads(r.read())
        except urllib.error.HTTPError as e:
            return {"_status": e.code, "_error": json.loads(e.read() or b"{}")}

    def close(self, sig: int | None = None, timeout: float = 60) -> dict:
        t0 = time.perf_counter()
        if sig is None:
            self.p.stdin.close()
        else:
            self.p.send_signal(sig)
        try:
            rc = self.p.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            self.p.kill()
            rc = "timeout"
        peak = self.rss.done()
        self.err.close()
        return {"rc": rc, "seconds": round(time.perf_counter() - t0, 3), "peak_rss_kib": peak}


class Driver:
    def __init__(self, a):
        self.a = a
        self.scen = load(HERE / "scenario.json")
        self.corpus = a.corpus
        self.meta = self.scen["corpora"][a.corpus]
        self.h = self.scen["headless"]["per_corpus"][a.corpus]
        self.personal = self.meta["kind"] == "personal"
        self.key = path_key() if self.personal else None
        self.work = a.work.resolve()
        self.out = a.out.resolve()
        self.vault = self.work / "vault"
        self.steps: list[dict] = []
        self.numbers: dict = {
            "cpu": cpu_model(),
            "platform": f"{platform.system()} {platform.machine()}",
            "python": platform.python_version(),
        }
        self.client = self.scen["agent"]["client"]
        self.calls = {"search": 0, "consumers": {}}
        self.queries: dict = {}

    # ---- helpers
    def ref(self, path: str) -> str:
        """A document as it may appear in a record."""
        if not self.personal or path.startswith(SYNTHETIC):
            return path
        return k2(path, self.key)

    def resolve_ref(self, ref: str, paths: list[str]) -> str:
        if not ref.startswith("k2:"):
            return ref
        for p in paths:
            if k2(p, self.key) == ref:
                return p
        raise RuntimeError(f"{ref}: no document of the copy has this key hash")

    def step(self, sid, ok, **details):
        rec = {"id": sid, "pass": bool(ok), **details}
        self.steps.append(rec)
        print(f"driver: {sid} {'ok' if ok else 'FAIL'}", file=sys.stderr, flush=True)
        return ok

    def run(self, *args, timed=False):
        t0 = time.perf_counter()
        r = subprocess.run([self.a.rootmark, *args], capture_output=True, text=True, check=False)
        dt = round(time.perf_counter() - t0, 3)
        with open(self.work / "cli.log", "a", encoding="utf-8") as f:
            f.write(f"$ rootmark {' '.join(args)}\nrc={r.returncode} {dt}s\n{r.stderr}\n")
        return r, dt

    def hits(self, res: dict) -> list[dict]:
        return [
            {
                "doc": self.ref(h["doc_path"]),
                "layer": h.get("layer"),
                "score": h.get("score"),
                "chunk_id": h.get("chunk_id"),
                "lineage_ids": h.get("lineage_ids"),
            }
            for h in res.get("hits", [])
        ]

    def search(self, srv, q: dict, include: bool, at: str | None = None, lat: list | None = None):
        body = {"query": q["query"], "mode": q["mode"], "k": 10, "include_proposal": include}
        if at:
            body["at"] = at
        t0 = time.perf_counter()
        res = srv.http("search", body, consumer="pilot-queries")
        if lat is not None:
            lat.append((time.perf_counter() - t0) * 1000)
        self.calls["search"] += 1
        self.calls["consumers"]["pilot-queries"] = (
            self.calls["consumers"].get("pilot-queries", 0) + 1
        )
        if "_error" in res:
            return {"error": res}
        return {"hits": self.hits(res), "checkpoint_id": res.get("checkpoint_id")}

    def phase(self, srv, phase: str, lat: list, at: str | None = None) -> dict:
        out = {}
        for q in self.h["queries"]:
            d = self.search(srv, q, False, at, lat)
            ip = self.search(srv, q, True, at, lat)
            out[q["id"]] = {
                "default": d.get("hits"),
                "include_proposal": ip.get("hits"),
                "checkpoint_id": d.get("checkpoint_id"),
                "error": d.get("error") or ip.get("error"),
            }
        return out

    def blame(self, srv, path: str) -> list[dict]:
        r = srv.http("blame", {"doc_path": path})
        return r.get("blocks", []) if "_error" not in r else []

    # ---- the scene
    def go(self) -> int:
        a = self.a
        self.work.mkdir(parents=True, exist_ok=True)
        self.out.mkdir(parents=True, exist_ok=True)
        if self.vault.exists():
            raise SystemExit(f"{self.vault} exists: use a fresh --work")
        # H01
        fresh_copy(a.snapshot, self.vault)
        summ = corpus_summary(self.vault)
        self.step("H01", summ == self.meta["snapshot"], summary=summ)
        paths = list_md(self.vault)
        # H02
        r, dt = self.run("ingest-dir", str(self.vault), str(self.vault))
        ing = json.loads(r.stdout) if r.returncode == 0 else {}
        n_ing = len(ing.get("ingested", []))
        self.numbers["ingest_s"] = dt
        self.numbers["documents"] = n_ing
        self.step(
            "H02",
            r.returncode == 0 and n_ing == len(paths),
            rc=r.returncode,
            documents=n_ing,
            events_hash=ing.get("events_hash"),
        )
        # H03
        recipe = self.scen["recipes"][self.meta["recipe"]]
        (self.vault / ".rootmark/policy.toml").write_text(recipe, encoding="utf-8")
        r, _ = self.run("policy", "lint", str(self.vault))
        lint = json.loads(r.stdout) if r.returncode == 0 else {}
        self.step(
            "H03",
            r.returncode == 0 and len(lint.get("rules", [])) == 1,
            rc=r.returncode,
            rules=[x.get("id") for x in lint.get("rules", [])],
        )
        # H04
        r, dt = self.run("index", "build", str(self.vault))
        ib = json.loads(r.stdout) if r.returncode == 0 else {}
        r2, _ = self.run("index", "status", str(self.vault))
        st = json.loads(r2.stdout) if r2.returncode == 0 else {}
        self.numbers["index_build_s"] = dt
        self.numbers["index_records"] = ib.get("records")
        self.numbers["index_embedded"] = ib.get("embedded")
        self.numbers["index_chunks_per_s"] = (
            round(ib["embedded"] / dt, 3) if ib.get("embedded") and dt else None
        )
        self.numbers["embedder_id"] = ib.get("embedder_id")
        self.step(
            "H04",
            r.returncode == 0 and r2.returncode == 0 and st.get("stale_docs") == [],
            embedder_id=ib.get("embedder_id"),
            records=ib.get("records"),
            stale_docs=len(st.get("stale_docs") or []),
        )
        # H05
        log = self.work / "serve.log"
        att = self.work / "attest.log"
        t0 = time.perf_counter()
        srv = Server(a.rootmark, self.vault, log, att, self.client)
        self.numbers["serve_start_s"] = round(time.perf_counter() - t0, 3)
        status = srv.http("status")
        ok5 = (
            status.get("attestation", {}).get("unattested") == "unverified"
            and status.get("server_policy", {}).get("identity", {}).get("human_id") == "pilot"
            and status.get("index", {}).get("state") == "ok"
        )
        self.step(
            "H05",
            ok5,
            mdk_version=status.get("mdk_version"),
            spec_version=status.get("spec_version"),
            index=status.get("index", {}).get("state"),
            embedder=status.get("embedder"),
        )
        cps = self.scen["headless"]["checkpoints"]
        # H06
        c = srv.http("checkpoint", {"name": cps[0]})
        self.step("H06", c.get("name") == cps[0], checkpoint_id=c.get("checkpoint_id"))
        # H07, H08
        prop_lat, srch_lat, fails, promoted_by_rule = [], [], [], 0
        layer_fail = []
        for w in self.h["propose"]:
            t0 = time.perf_counter()
            res = srv.mcp(
                "propose",
                {
                    "doc_path": w["doc_path"],
                    "content": w["content"],
                    "reason": w["reason"],
                    "mode": "create",
                },
            )
            t1 = time.perf_counter()
            prop_lat.append((t1 - t0) * 1000)
            if res is None or "_error" in res:
                fails.append({"i": w["i"], "error": res})
                continue
            promoted_by_rule += sum(
                len(e.get("lineage_ids", [])) for e in (res.get("promoted") or [])
            )
            want = "canonical" if w["rule"] else "proposal"
            got = [x.get("layer") for x in res.get("layers", [])]
            if not got or any(g != want for g in got):
                layer_fail.append({"i": w["i"], "layers": got})
            if w["rule"]:
                found = False
                while time.perf_counter() - t1 < 10:
                    s = srv.http(
                        "search",
                        {"query": w["code"], "mode": "lexical", "k": 10},
                        consumer="pilot-h08",
                    )
                    self.calls["search"] += 1
                    self.calls["consumers"]["pilot-h08"] = (
                        self.calls["consumers"].get("pilot-h08", 0) + 1
                    )
                    if any(
                        h["doc_path"] == w["doc_path"] and h.get("layer") == "canonical"
                        for h in s.get("hits", [])
                    ):
                        found = True
                        break
                srch_lat.append((time.perf_counter() - t1) * 1000 if found else None)
        self.numbers["propose_ms"] = {
            "p50": pct(prop_lat, 50),
            "p95": pct(prop_lat, 95),
            "n": len(prop_lat),
        }
        good = [x for x in srch_lat if x is not None]
        self.numbers["propose_to_searchable_ms"] = {
            "p50": pct(good, 50),
            "p95": pct(good, 95),
            "n": len(good),
        }
        blame_layers_ok = True
        for w in self.h["propose"]:
            want = "canonical" if w["rule"] else "proposal"
            bl = self.blame(srv, w["doc_path"])
            if not bl or any(b["layer"] != want for b in bl):
                blame_layers_ok = False
        self.step(
            "H07",
            not fails and not layer_fail and blame_layers_ok,
            calls=len(prop_lat),
            failures=fails[:5],
            layer_failures=layer_fail[:5],
            lineages_promoted_by_rule=promoted_by_rule,
        )
        self.step("H08", len(good) == 50 and None not in srch_lat, found=len(good), of=50)
        self.driver_counts = {"promoted_by_rule": promoted_by_rule}
        # H09 direct writes
        stage = self.work / "stage"
        stage.mkdir(exist_ok=True)
        direct_seqs: set[int] = set()
        w_lat, wfail = [], []
        for d in self.h["direct"]:
            path = d.get("doc_path") or self.resolve_ref(d["doc_ref"], paths)
            target = self.vault / path
            head = srv.http("events", {"since_seq": 0, "limit": 1}).get("head_seq", 0)
            data = d["content"].encode()
            if d["op"] == "append":
                old = target.read_bytes()
                if not old.endswith(b"\n"):
                    old += b"\n"
                data = old + data
            tmp = stage / f"w{d['j']}"
            tmp.write_bytes(data)
            target.parent.mkdir(parents=True, exist_ok=True)
            t0 = time.perf_counter()
            os.replace(tmp, target)
            seen = None
            while time.perf_counter() - t0 < 30:
                ev = srv.http("events", {"since_seq": head, "path_prefix": [path], "limit": 1000})
                kinds = [e["kind"] for e in ev.get("events", [])]
                if "SnapshotCommitted" in kinds:
                    seen = ev["events"]
                    break
                time.sleep(0.05)
            if seen is None:
                wfail.append(d["j"])
                continue
            w_lat.append((time.perf_counter() - t0) * 1000)
            direct_seqs |= {e["seq"] for e in seen if e["kind"].startswith("Block")}
        self.numbers["direct_write_to_ingested_ms"] = {
            "p50": pct(w_lat, 50),
            "p95": pct(w_lat, 95),
            "n": len(w_lat),
        }
        blames = {}
        for d in self.h["direct"]:
            path = d.get("doc_path") or self.resolve_ref(d["doc_ref"], paths)
            blames[path] = self.blame(srv, path)
        new_blocks = [
            b for bl in blames.values() for b in bl if b.get("seq_created") in direct_seqs
        ]
        canon = sum(1 for b in new_blocks if b["layer"] == "canonical")
        unver = sum(1 for b in new_blocks if b["layer"] == "unverified")
        self.step(
            "H09",
            not wfail and canon == 0 and unver == len(new_blocks) and new_blocks,
            writes=len(self.h["direct"]),
            not_ingested=wfail,
            new_blocks=len(new_blocks),
            canonical=canon,
            unverified=unver,
        )
        self.driver_counts["direct_blocks"] = len(new_blocks)
        # H10, H11
        c = srv.http("checkpoint", {"name": cps[1]})
        lat = []
        res1 = self.phase(srv, "ajan-yazdi", lat)
        self.queries["ajan-yazdi"] = res1
        bl_at = {w["doc_path"]: self.blame(srv, w["doc_path"]) for w in self.h["propose"][:10]}
        self.step("H10", c.get("name") == cps[1], checkpoint_id=c.get("checkpoint_id"))
        f11 = check_phase(self.h["queries"], "ajan-yazdi", res1)
        leaks1 = sum(len(default_leaks(v["default"] or [])) for v in res1.values())
        self.step(
            "H11",
            not f11 and leaks1 == 0,
            failures=f11[:20],
            n_failures=len(f11),
            default_leaks=leaks1,
        )
        # H12, H13
        promote_n = demote_n = 0
        bad = []
        for doc in self.h["promote_docs"]:
            for b in self.blame(srv, doc):
                r = srv.http("promote", {"lineage_id": b["lineage_id"], "reason": "p2q onay"})
                promote_n += 1
                if r.get("decision") != "promoted":
                    bad.append(r)
        self.step("H12", not bad and promote_n >= 10, promotes=promote_n, failures=bad[:3])
        bad = []
        for doc in self.h["demote_docs"]:
            for b in self.blame(srv, doc):
                r = srv.http("demote", {"lineage_id": b["lineage_id"], "reason": "p2q geri al"})
                demote_n += 1
                if r.get("decision") != "demoted":
                    bad.append(r)
        self.step("H13", not bad and demote_n >= 2, demotes=demote_n, failures=bad[:3])
        self.driver_counts.update({"promoted_by_human": promote_n, "demoted_by_human": demote_n})
        # H14, H15
        c = srv.http("checkpoint", {"name": cps[2]})
        self.step("H14", c.get("name") == cps[2], checkpoint_id=c.get("checkpoint_id"))
        res2 = self.phase(srv, "son", lat)
        self.queries["son"] = res2
        f15 = check_phase(self.h["queries"], "son", res2)
        leaks2 = sum(len(default_leaks(v["default"] or [])) for v in res2.values())
        self.step(
            "H15",
            not f15 and leaks2 == 0,
            failures=f15[:20],
            n_failures=len(f15),
            default_leaks=leaks2,
        )
        self.numbers["search_ms"] = {"p50": pct(lat, 50), "p95": pct(lat, 95), "n": len(lat)}
        # H16
        wrong = []
        for w in self.h["propose"]:
            want = (
                ("proposal" if w["doc_path"] in self.h["demote_docs"] else "canonical")
                if w["rule"]
                else ("canonical" if w["doc_path"] in self.h["promote_docs"] else "proposal")
            )
            got = sorted({b["layer"] for b in self.blame(srv, w["doc_path"])})
            if got != [want]:
                wrong.append({"doc": w["doc_path"], "got": got, "want": want})
        for d in self.h["direct"]:
            path = d.get("doc_path") or self.resolve_ref(d["doc_ref"], paths)
            got = sorted(
                {b["layer"] for b in self.blame(srv, path) if b.get("seq_created") in direct_seqs}
            )
            if got != ["unverified"]:
                wrong.append({"doc": self.ref(path), "got": got, "want": "unverified"})
        self.step("H16", not wrong, wrong=wrong[:10], n_wrong=len(wrong))
        # H17 traces over MCP
        tq_ok, details = True, []
        since = srv.mcp("trace", {}).get("traces", [])
        n_before = len(since)
        for tq in self.scen["headless"]["trace_queries"]:
            s = srv.mcp("search", {"query": tq["query"], "k": 10})
            self.calls["search"] += 1
            cons = f"{self.client['name']}/{self.client['version']}"
            self.calls["consumers"][cons] = self.calls["consumers"].get(cons, 0) + 1
            tr = srv.mcp("trace", {"consumer": cons}).get("traces", [])
            mine = [t for t in tr if t["trace_id"] == s.get("trace_id")]
            hit_l = sorted({lid for h in s.get("hits", []) for lid in h.get("lineage_ids", [])})
            tr_l = sorted({r["lineage_id"] for t in mine for r in t.get("returned", [])})
            same = len(mine) == 1 and hit_l == tr_l and len(hit_l) > 0
            tq_ok &= same
            details.append(
                {
                    "id": tq["id"],
                    "trace": len(mine),
                    "hit_lineages": len(hit_l),
                    "trace_lineages": len(tr_l),
                    "equal": same,
                }
            )
        self.step("H17", tq_ok, traces=details, traces_before=n_before)
        # H18 checkpoint return
        lat18 = []
        back = self.phase(srv, "ajan-yazdi@at", lat18, at=cps[1])
        diff = [
            qid
            for qid in res1
            if {k: v for k, v in res1[qid].items() if k != "checkpoint_id"}
            != {k: v for k, v in back[qid].items() if k != "checkpoint_id"}
        ]
        bl_back = {
            p: srv.http("blame", {"doc_path": p, "at": cps[1]}).get("blocks", []) for p in bl_at
        }
        bdiff = [
            p
            for p in bl_at
            if [(b["lineage_id"], b["layer"]) for b in bl_at[p]]
            != [(b["lineage_id"], b["layer"]) for b in bl_back[p]]
        ]
        self.step("H18", not diff and not bdiff, query_diffs=diff, blame_diffs=bdiff)
        # H19
        sh, sm = srv.http("status"), srv.mcp("status", {})
        keys = ("index", "policy", "attestation", "last_checkpoint")
        neq = [k for k in keys if sh.get(k) != sm.get(k)]
        self.step("H19", not neq, differing=neq)
        # H20 prune dry run
        before = tree_b2(self.vault / ".rootmark", skip={"server.token"})
        pr = srv.http("prune", {"dry_run": True})
        after = tree_b2(self.vault / ".rootmark", skip={"server.token"})
        changed = sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))
        self.step(
            "H20",
            "_error" not in pr and pr.get("dry_run") is True and not changed,
            changed_files=len(changed),
            events_hash=pr.get("events_hash"),
        )
        # H21 SIGTERM and reopen
        eh_before = pr.get("events_hash")
        q0 = self.h["queries"][0]
        r_before = self.search(srv, q0, False)
        stop = srv.close(signal.SIGTERM)
        self.numbers["serve_peak_rss_kib"] = stop["peak_rss_kib"]
        r, _ = self.run("events-hash", str(self.vault))
        eh_mid = r.stdout.strip().strip('"')
        srv = Server(a.rootmark, self.vault, log, att, self.client)
        st2 = srv.http("status")
        r_after = self.search(srv, q0, False)
        self.step(
            "H21",
            stop["rc"] == 0
            and st2.get("index", {}).get("state") == "ok"
            and r_before.get("hits") == r_after.get("hits")
            and eh_mid
            and (eh_before is None or eh_mid == eh_before),
            sigterm_rc=stop["rc"],
            sigterm_s=stop["seconds"],
            events_hash=eh_mid,
        )
        # H23 needs the counts while the server runs: blame every document
        counts = {"canonical": 0, "proposal": 0, "unverified": 0, "derived": 0}
        for p in list_md(self.vault):
            for b in self.blame(srv, p):
                if not b.get("inactive"):
                    counts[b["layer"]] = counts.get(b["layer"], 0) + 1
        self.driver_counts["blame_layers"] = {k: v for k, v in counts.items() if v}
        cps_list = srv.http("checkpoints", {}).get("checkpoints", [])
        self.driver_counts["checkpoints_named"] = sum(1 for c in cps_list if c.get("name"))
        status_final = srv.http("status")
        # H22
        stop2 = srv.close()
        self.numbers["serve_peak_rss_kib"] = max(
            self.numbers["serve_peak_rss_kib"], stop2["peak_rss_kib"]
        )
        r, _ = self.run("index", "compact", str(self.vault))
        r2, _ = self.run("events-hash", str(self.vault))
        events_hash = r2.stdout.strip().strip('"')
        idx = tree_b2(self.vault / ".rootmark/index")
        qres = json.dumps({"ajan-yazdi": res1, "son": res2}, sort_keys=True).encode()
        hashes = {
            "events_hash": events_hash,
            "index_files": idx,
            "vectors": idx.get("vectors/vectors.f32"),
            "manifest": idx.get("manifest.json"),
            "query_results_b2": b2(qres, 32),
            "rankings_b2": b2(
                json.dumps(
                    {
                        ph: {
                            q: [
                                (h["doc"], h["chunk_id"])
                                for h in (v["default"] or []) + (v["include_proposal"] or [])
                            ]
                            for q, v in res.items()
                        }
                        for ph, res in (("ajan-yazdi", res1), ("son", res2))
                    },
                    sort_keys=True,
                ).encode(),
                32,
            ),
        }
        self.step(
            "H22",
            r.returncode == 0 and stop2["rc"] == 0 and bool(events_hash),
            compact_rc=r.returncode,
            stop_rc=stop2["rc"],
            events_hash=events_hash,
        )
        sizes = 0
        for dp, _dn, fn in os.walk(self.vault / ".rootmark"):
            sizes += sum((Path(dp) / f).stat().st_size for f in fn)
        self.numbers["rootmark_dir_bytes"] = sizes
        # H23 observe.py
        stf = self.work / "status.json"
        stf.write_text(json.dumps(status_final), encoding="utf-8")
        snap0 = tree_b2(self.vault)
        a0 = b2(att.read_bytes()) if att.exists() else None
        r = subprocess.run(
            [
                sys.executable,
                str(HERE / "observe.py"),
                "--vault",
                str(self.vault),
                "--attest-log",
                str(att),
                "--status",
                str(stf),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        obs = json.loads(r.stdout) if r.returncode == 0 else {}
        ro = tree_b2(self.vault) == snap0 and (b2(att.read_bytes()) if att.exists() else None) == a0
        cmp = self.compare_observe(obs)
        dump(obs, self.out / "observe.json")
        dump(cmp, self.out / "observe_compare.json")
        self.step(
            "H23",
            r.returncode == 0 and ro and all(x["equal"] for x in cmp),
            read_only=ro,
            compared=len(cmp),
            unequal=[x for x in cmp if not x["equal"]],
        )
        # records
        dump(self.steps, self.out / "steps.json")
        dump({"ajan-yazdi": res1, "son": res2, "ajan-yazdi@at": back}, self.out / "queries.json")
        dump(hashes, self.out / "hashes.json")
        dump(self.numbers, self.out / "numbers.json")
        passed = sum(s["pass"] for s in self.steps)
        print(
            json.dumps(
                {
                    "corpus": self.corpus,
                    "passed": passed,
                    "total": len(self.steps),
                    "events_hash": events_hash,
                }
            )
        )
        return 0 if passed == len(self.steps) else 1

    def compare_observe(self, obs: dict) -> list[dict]:
        dc = self.driver_counts
        layers = {
            k: v for k, v in obs.get("layers", {}).get("lineages", {}).items() if k != "inactive"
        }
        pairs = [
            (
                "promotions by rule (lineages)",
                dc["promoted_by_rule"],
                obs.get("promotions", {}).get("lineages_by_rule", {}).get("toplayici", 0),
            ),
            (
                "promotions by the human (lineages)",
                dc["promoted_by_human"],
                obs.get("promotions", {}).get("lineages_by_actor", {}).get("human", 0),
            ),
            (
                "demotions by the human (lineages)",
                dc["demoted_by_human"],
                obs.get("demotions", {}).get("lineages_by_actor", {}).get("human", 0),
            ),
            ("live lineages per layer", dc["blame_layers"], layers),
            ("named checkpoints", dc["checkpoints_named"], obs.get("checkpoints", {}).get("named")),
            ("traces (searches made)", self.calls["search"], obs.get("traces", {}).get("total")),
            (
                "traces per consumer",
                self.calls["consumers"],
                obs.get("traces", {}).get("by_consumer"),
            ),
            (
                "unverified observations (direct writes)",
                len(self.h["direct"]),
                obs.get("attest", {}).get("observations", {}).get("unverified", 0),
            ),
        ]
        return [{"what": w, "driver": d, "observe": o, "equal": d == o} for w, d, o in pairs]


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", required=True, choices=["qorbit", "dnotes"])
    ap.add_argument("--snapshot", required=True, type=Path)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--rootmark", default="rootmark")
    a = ap.parse_args(argv)
    if not os.environ.get("ROOTMARK_MODELS"):
        raise SystemExit("ROOTMARK_MODELS is not set (docs/GUIDE.md §1.1)")
    return Driver(a).go()


if __name__ == "__main__":
    sys.exit(main())
