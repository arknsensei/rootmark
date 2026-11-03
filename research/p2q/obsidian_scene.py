"""P2-Q Obsidian scene (GOAL Q2 (c), Q-7; scenario.json `obsidian`, O01–O14).

A copy of the user's vault, the installed `rootmark serve --watch`, and the
real Obsidian in its own profile (`--user-data-dir`, CDP; the P1-E.1 /
K-E1.3 harness's method), the plugin installed from the release zip by
docs/GUIDE.md §5.1 (the shell block as written, run by guide_run; the two
clicks of steps 1–2 through Obsidian's own API, as the real e2e does).
Edits, renames and promotions go through Obsidian (its vault API, the
plugin's views); the agent proposes over MCP. Python standard library only
(a minimal WebSocket client for CDP). macOS only.

    python3 -m p2q.obsidian_scene --snapshot <Obs snapshot> --work <dir> --out <records>
        --guide-home <guide_run home with ~/rootmark-install> [--obsidian <binary>]

Records (no note text; personal paths as k2): <out>/steps.json, edits.json,
queries.json, numbers.json, attest.json, observe.json, observe_compare.json.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import re
import shutil
import signal
import socket
import struct
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

from p2q.common import b2, corpus_summary, dump, fresh_copy, k2, list_md, load, path_key, tree_b2
from p2q.driver import Server, pct
from p2q.evaluate import check_obsidian_query

HERE = Path(__file__).resolve().parent
SYNTHETIC = ("pilot/", "ajan/")
OBSIDIAN = "/Applications/Obsidian.app/Contents/MacOS/Obsidian"


class WS:
    """A minimal RFC 6455 client (text frames, client masking)."""

    def __init__(self, url: str):
        m = re.match(r"ws://([^:/]+):(\d+)(/.*)", url)
        host, port, path = m.group(1), int(m.group(2)), m.group(3)
        self.s = socket.create_connection((host, port), timeout=120)
        key = base64.b64encode(os.urandom(16)).decode()
        req = (
            f"GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nUpgrade: websocket\r\n"
            f"Connection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        )
        self.s.sendall(req.encode())
        buf = b""
        while b"\r\n\r\n" not in buf:
            buf += self.s.recv(4096)
        head, self.rest = buf.split(b"\r\n\r\n", 1)
        if b" 101 " not in head.split(b"\r\n")[0]:
            raise RuntimeError(f"websocket handshake: {head[:80]!r}")

    def _read(self, n: int) -> bytes:
        while len(self.rest) < n:
            chunk = self.s.recv(65536)
            if not chunk:
                raise RuntimeError("websocket closed")
            self.rest += chunk
        out, self.rest = self.rest[:n], self.rest[n:]
        return out

    def send(self, text: str):
        data = text.encode()
        hdr = bytearray([0x81])
        n = len(data)
        if n < 126:
            hdr.append(0x80 | n)
        elif n < 65536:
            hdr += bytes([0x80 | 126]) + struct.pack(">H", n)
        else:
            hdr += bytes([0x80 | 127]) + struct.pack(">Q", n)
        mask = os.urandom(4)
        self.s.sendall(bytes(hdr) + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(data)))

    def recv(self) -> str:
        parts = []
        while True:
            b0, b1 = self._read(2)
            op, n = b0 & 0x0F, b1 & 0x7F
            if n == 126:
                n = struct.unpack(">H", self._read(2))[0]
            elif n == 127:
                n = struct.unpack(">Q", self._read(8))[0]
            payload = self._read(n)
            if op == 0x9:  # ping
                continue
            if op == 0x8:
                raise RuntimeError("websocket closed by Obsidian")
            parts.append(payload)
            if b0 & 0x80:
                return b"".join(parts).decode()


class Page:
    def __init__(self, port: int):
        url = None
        for _ in range(240):
            try:
                with urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list", timeout=2) as r:
                    for t in json.loads(r.read()):
                        if t["type"] == "page" and t["url"].startswith(
                            "app://obsidian.md/index.html"
                        ):
                            url = t["webSocketDebuggerUrl"]
            except OSError:
                pass
            if url:
                break
            time.sleep(0.5)
        if not url:
            raise RuntimeError("no Obsidian window on the CDP port")
        self.ws, self.n = WS(url), 0
        self.wait(
            "typeof app !== 'undefined' && app.workspace.layoutReady === true", "workspace", 90
        )

    def ev(self, expr: str):
        self.n += 1
        self.ws.send(
            json.dumps(
                {
                    "id": self.n,
                    "method": "Runtime.evaluate",
                    "params": {
                        "expression": f"(async () => ({expr}))()",
                        "awaitPromise": True,
                        "returnByValue": True,
                    },
                }
            )
        )
        while True:
            msg = json.loads(self.ws.recv())
            if msg.get("id") == self.n:
                res = msg.get("result", {})
                if res.get("exceptionDetails"):
                    ex = res["exceptionDetails"]
                    raise RuntimeError(
                        "in Obsidian: "
                        + str(ex.get("exception", {}).get("description") or ex.get("text"))[:300]
                    )
                return res.get("result", {}).get("value")

    def wait(self, expr: str, what: str, s: float = 30):
        t0 = time.time()
        while time.time() - t0 < s:
            try:
                if self.ev(expr):
                    return True
            except RuntimeError:
                pass
            time.sleep(0.2)
        raise TimeoutError(f"timed out waiting for {what}")


HELPERS = """(window.__p2q = {
  view(type) { const el = document.querySelector('.workspace-leaf-content[data-type="' + type + '"] .view-content'); if (!el) throw new Error('view ' + type); return el; },
  button(root, text) { const b = [...root.querySelectorAll('button')].find((x) => x.textContent === text); if (!b) throw new Error('no button ' + text); return b; },
  statusBar() { const it = [...document.querySelectorAll('.status-bar-item')].find((e) => (e.textContent ?? '').startsWith('rootmark')); return it ? { text: it.textContent, title: it.getAttribute('title') } : null; },
  attestBar() { const it = [...document.querySelectorAll('.status-bar-item')].find((e) => (e.textContent ?? '').startsWith('attested')); return it ? it.textContent : null; },
  notices() { return [...document.querySelectorAll('.notice')].map((n) => n.textContent); },
  type(input, value) { input.value = value; input.dispatchEvent(new Event('input', { bubbles: true })); input.dispatchEvent(new Event('change', { bubbles: true })); },
}, true)"""


class Scene:
    def __init__(self, a):
        self.a = a
        self.scen = load(HERE / "scenario.json")
        self.o = self.scen["obsidian"]
        self.meta = self.scen["corpora"]["obsidian"]
        self.key = path_key()
        self.work, self.out = a.work.resolve(), a.out.resolve()
        self.vault = self.work / "vault"
        self.steps, self.numbers, self.edits = [], {}, []
        self.md_ok = []  # (operation, files it may change)

    def ref(self, p: str) -> str:
        return p if p.startswith(SYNTHETIC) else k2(p, self.key)

    def resolve(self, ref: str) -> str:
        for p in self.paths:
            if k2(p, self.key) == ref:
                return p
        raise RuntimeError(f"{ref}: not in the copy")

    def step(self, sid, ok, **d):
        self.steps.append({"id": sid, "pass": bool(ok), **d})
        print(f"obsidian_scene: {sid} {'ok' if ok else 'FAIL'}", file=sys.stderr, flush=True)

    def md(self) -> dict:
        return {p: b2((self.vault / p).read_bytes()) for p in list_md(self.vault)}

    def head(self) -> int:
        return self.srv.http("events", {"since_seq": 0, "limit": 1}).get("head_seq", 0)

    def wait_events(self, since: int, paths: list[str], s: float = 30, quiet: float = 0) -> list:
        """Events since `since` on `paths` until a SnapshotCommitted (or
        BlockDeleted) appears for each; then `quiet` seconds without new
        events in the vault (link updates of other notes)."""
        t0, got = time.perf_counter(), []
        need = set(paths)
        while time.perf_counter() - t0 < s:
            ev = self.srv.http("events", {"since_seq": since, "limit": 1000}).get("events", [])
            done = {e["doc_path"] for e in ev if e["kind"] in ("SnapshotCommitted", "BlockDeleted")}
            if need <= done:
                got = ev
                break
            time.sleep(0.05)
        else:
            return None
        lat = (time.perf_counter() - t0) * 1000
        if quiet:
            last, n = time.perf_counter(), len(got)
            while time.perf_counter() - last < quiet:
                ev = self.srv.http("events", {"since_seq": since, "limit": 1000}).get("events", [])
                if len(ev) != n:
                    n, last, got = len(ev), time.perf_counter(), ev
                time.sleep(0.2)
        return {"events": got, "ms": lat}

    def layers_since(self, path: str, since: int) -> dict:
        bl = self.srv.http("blame", {"doc_path": path}).get("blocks", [])
        c = {}
        for b in bl:
            if b.get("seq_last", -1) >= since or b.get("seq_created", -1) >= since:
                c[b["layer"]] = c.get(b["layer"], 0) + 1
        return c

    def go(self) -> int:
        a = self.a
        self.work.mkdir(parents=True, exist_ok=True)
        self.out.mkdir(parents=True, exist_ok=True)
        fresh_copy(a.snapshot, self.vault)
        s = corpus_summary(self.vault)
        self.step("O01", s == self.meta["snapshot"], summary=s)
        self.paths = list_md(self.vault)
        rm = a.rootmark
        t0 = time.perf_counter()
        r = subprocess.run(
            [rm, "ingest-dir", str(self.vault), str(self.vault)],
            capture_output=True,
            text=True,
            check=False,
        )
        self.numbers["ingest_s"] = round(time.perf_counter() - t0, 3)
        self.step("O02", r.returncode == 0, rc=r.returncode)
        (self.vault / ".rootmark/policy.toml").write_text(self.scen["recipes"]["obsidian"])
        r = subprocess.run(
            [rm, "policy", "lint", str(self.vault)], capture_output=True, text=True, check=False
        )
        self.step("O03", r.returncode == 0, rc=r.returncode)
        t0 = time.perf_counter()
        r = subprocess.run(
            [rm, "index", "build", str(self.vault)], capture_output=True, text=True, check=False
        )
        ib = json.loads(r.stdout) if r.returncode == 0 else {}
        self.numbers["index_build_s"] = round(time.perf_counter() - t0, 3)
        self.numbers["index_embedded"] = ib.get("embedded")
        att = self.work / "attest.log"
        self.srv = Server(
            rm,
            self.vault,
            self.work / "serve.log",
            att,
            {"name": "yardimci", "version": "1.0"},
            http="127.0.0.1",
        )
        st = self.srv.http("status")
        self.step(
            "O04",
            st.get("index", {}).get("state") == "ok" and self.srv.port == 27357,
            port=self.srv.port,
            index=st.get("index", {}).get("state"),
        )
        md_before = self.md()
        # O05: the plugin by GUIDE §5.1 (the block as written) and Obsidian
        env = dict(os.environ, HOME=str(a.guide_home), VAULT=str(self.vault))
        r = subprocess.run(
            [
                sys.executable,
                "-m",
                "p2q.guide_run",
                "--guide",
                str(a.guide),
                "--work",
                str(a.guide_home.parent),
                "--mode",
                "release",
                "--keep-home",
                "--only",
                "5.1#1",
            ],
            cwd=HERE.parent,
            env=env,
            capture_output=True,
            text=True,
            check=False,
        )
        gr = json.loads(r.stdout.strip().splitlines()[-1]) if r.stdout.strip() else {}
        data = self.work / "obsidian-data"
        data.mkdir(exist_ok=True)
        (data / "obsidian.json").write_text(
            json.dumps(
                {"vaults": {"0123456789abcdef": {"path": str(self.vault), "ts": 1, "open": True}}}
            )
        )
        port = 9333
        self.obs = subprocess.Popen(
            [a.obsidian, f"--user-data-dir={data}", f"--remote-debugging-port={port}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        try:
            return self.scene(port, gr, md_before, att)
        finally:
            self.obs.send_signal(signal.SIGTERM)
            try:
                self.obs.wait(timeout=20)
            except subprocess.TimeoutExpired:
                self.obs.kill()

    def scene(self, port, gr, md_before, att) -> int:
        pg = Page(port)
        self.pg = pg
        pg.ev(
            "app.plugins.setEnable(true).then(() => app.plugins.loadManifests())"
            ".then(() => app.plugins.enablePluginAndSave('rootmark')).then(() => true)"
        )
        pg.wait("!!app.plugins.plugins.rootmark", "the plugin", 60)
        pg.ev(HELPERS)
        pg.wait(
            "(() => { const s = __p2q.statusBar(); return s && s.text !== 'rootmark: offline' || false; })()",
            "the plugin to connect",
            60,
        )
        sb = pg.ev("__p2q.statusBar()")
        pg.ev("(app.vault.setConfig('alwaysUpdateLinks', true), true)")
        self.step(
            "O05",
            gr.get("failed") == [] and sb["text"] == "rootmark 0.3.0 · human:pilot",
            guide_block=gr,
            status_bar=sb["text"],
        )
        md_cur = md_before
        # O06 edits
        lat, false_unv, recs = [], 0, []
        pg.ev(
            "app.vault.getAbstractFileByPath('pilot') ? true : app.vault.createFolder('pilot').then(() => true)"
        )
        for e in self.o["edits"]:
            path = e.get("doc_path") or self.resolve(e["doc_ref"])
            h = self.head()
            J = json.dumps(path)
            if e["op"] == "create":
                pg.ev(f"app.vault.create({J}, {json.dumps(e['content'])}).then(() => true)")
            elif e["op"] == "append":
                pg.ev(
                    f"(async () => {{ const f = app.vault.getAbstractFileByPath({J}); "
                    f"await app.vault.modify(f, (await app.vault.read(f)) + {json.dumps(e['content'])}); return true; }})()"
                )
            elif e["op"] == "replace":
                pg.ev(
                    f"(async () => {{ const f = app.vault.getAbstractFileByPath({J}); "
                    f"await app.vault.modify(f, {json.dumps(e['content'])}); return true; }})()"
                )
            else:
                pg.ev(f"app.vault.delete(app.vault.getAbstractFileByPath({J})).then(() => true)")
            w = self.wait_events(h, [path])
            rec = {"e": e["e"], "op": e["op"], "doc": self.ref(path), "ingested": w is not None}
            if w:
                lat.append(w["ms"])
                rec["ms"] = round(w["ms"], 1)
                if e["op"] != "delete":
                    lay = self.layers_since(path, h)
                    rec["layers"] = lay
                    false_unv += lay.get("unverified", 0)
            recs.append(rec)
            new = self.md()
            changed = {p for p in set(new) | set(md_cur) if new.get(p) != md_cur.get(p)}
            self.md_ok.append(
                {"op": f"edit {e['e']}", "changed": len(changed), "only_target": changed <= {path}}
            )
            md_cur = new
        self.numbers["obsidian_save_to_layer_ms"] = {
            "p50": pct(lat, 50),
            "p95": pct(lat, 95),
            "n": len(lat),
        }
        canon = sum(r.get("layers", {}).get("canonical", 0) for r in recs)
        self.step(
            "O06",
            all(r["ingested"] for r in recs),
            edits=len(recs),
            not_ingested=[r["e"] for r in recs if not r["ingested"]],
            canonical_blocks=canon,
            false_unverified_blocks=false_unv,
        )
        # O07 renames
        rn = []
        for r in self.o["renames"]:
            old = self.resolve(r["doc_ref"])
            new_path = str(Path(old).parent / (r["new_name"] + ".md")).removeprefix("./")
            h = self.head()
            pg.ev(
                f"app.fileManager.renameFile(app.vault.getAbstractFileByPath({json.dumps(old)}), "
                f"{json.dumps(new_path)}).then(() => true)"
            )
            w = self.wait_events(h, [new_path], s=60, quiet=4)
            docs = sorted({ev["doc_path"] for ev in (w or {}).get("events", []) if ev["doc_path"]})
            lay = {
                self.ref(d) if d != new_path else "renamed": self.layers_since(d, h)
                for d in docs
                if (self.vault / d).exists()
            }
            rn.append(
                {
                    "r": r["r"],
                    "ingested": w is not None,
                    "ms": round(w["ms"], 1) if w else None,
                    "docs_changed": len(docs),
                    "layers": lay,
                }
            )
            new = self.md()
            changed = {p for p in set(new) | set(md_cur) if new.get(p) != md_cur.get(p)}
            self.md_ok.append(
                {
                    "op": f"rename {r['r']}",
                    "changed": len(changed),
                    "only_target": changed <= set(docs) | {old, new_path},
                }
            )
            md_cur = new
            self.paths = list_md(self.vault)
        self.step("O07", all(x["ingested"] for x in rn), renames=rn)
        # O08 agent proposals
        lin = {}
        okp = True
        for p in self.o["proposals"]:
            res = self.srv.mcp(
                "propose",
                {
                    "doc_path": p["doc_path"],
                    "content": p["content"],
                    "reason": p["reason"],
                    "mode": "create",
                },
            )
            lay = [x["layer"] for x in (res or {}).get("layers", [])]
            okp &= bool(lay) and set(lay) == {"proposal"}
            lin[p["doc_path"]] = [x["lineage_id"] for x in (res or {}).get("layers", [])]
        md_cur = self.md()
        self.step("O08", okp, proposals=len(lin))
        # O09 promote 4 from Obsidian (the plugin's proposals view), demote 2 (HTTP)
        promoted_ui, notes = 0, []
        for op in self.o["layer_ops"]:
            d = op["doc_path"]
            if op["op"] == "promote":
                try:
                    pg.wait(
                        f"!!app.vault.getAbstractFileByPath({json.dumps(d)})",
                        f"{d} in Obsidian",
                        30,
                    )
                    pg.ev("(app.commands.executeCommandById('rootmark:proposals'), true)")
                    pg.wait(
                        "!!document.querySelector('.workspace-leaf-content[data-type=\"rootmark-proposals\"] table')",
                        "the proposals table",
                        30,
                    )
                    n = pg.ev(f"""(() => {{ const v = __p2q.view('rootmark-proposals'); let n = 0;
                      v.querySelectorAll('tr').forEach((tr) => {{ const c = tr.querySelector('input[type="checkbox"]');
                        if (!c) return; const on = tr.textContent.includes({json.dumps(d)});
                        if (c.checked !== on) c.click(); if (on) n++; }});
                      __p2q.button(v, 'Promote selected').click(); return n; }})()""")
                    title = f"Promote {n} block(s)"
                    modal = f"[...document.querySelectorAll('.modal')].find((m) => m.querySelector('.modal-title')?.textContent === {json.dumps(title)})"
                    pg.wait(f"!!{modal}", "the reason modal", 15)
                    pg.ev(
                        f"(__p2q.type({modal}.querySelector('textarea.rootmark-reason'), 'p2q obsidian onay'), true)"
                    )
                    pg.ev(f"(__p2q.button({modal}, 'Promote').click(), true)")
                    pg.wait(
                        f"__p2q.notices().some((x) => x.includes('{n} promoted'))",
                        "the promoted notice",
                        30,
                    )
                    promoted_ui += n
                    notes.append({"op": "promote", "doc": d, "blocks": n, "via": "plugin"})
                except (RuntimeError, TimeoutError) as ex:
                    notes.append(
                        {"op": "promote", "doc": d, "via": "plugin", "error": str(ex)[:200]}
                    )
            else:
                dis = pg.ev(
                    "(() => { try { return __p2q.button(__p2q.view('rootmark-proposals'), 'Demote').disabled; } catch (e) { return null; } })()"
                )
                n = 0
                for lid in lin[d]:
                    r = self.srv.http(
                        "demote",
                        {"lineage_id": lid, "reason": "p2q geri al"},
                        consumer="obsidian-pilot",
                    )
                    n += r.get("decision") == "demoted"
                notes.append(
                    {
                        "op": "demote",
                        "doc": d,
                        "blocks": n,
                        "via": "http",
                        "plugin_demote_disabled": dis,
                    }
                )
        final = {}
        for d in self.o["expect"]["proposal_final_layers"]:
            final[d] = sorted(
                {b["layer"] for b in self.srv.http("blame", {"doc_path": d}).get("blocks", [])}
            )
        okl = all(final[d] == [w] for d, w in self.o["expect"]["proposal_final_layers"].items())
        self.step(
            "O09", okl and all("error" not in x for x in notes), ops=notes, final_layers=final
        )
        self.promoted_human = sum(x["blocks"] for x in notes if x["op"] == "promote")
        self.demoted_human = sum(x["blocks"] for x in notes if x["op"] == "demote")
        # O10 queries
        qres, fails, qlat = {}, [], []
        for q in self.o["queries"]:
            t0 = time.perf_counter()
            r = self.srv.http(
                "search",
                {"query": q["query"], "mode": q["mode"], "k": 10},
                consumer="obsidian-pilot",
            )
            qlat.append((time.perf_counter() - t0) * 1000)
            hits = [
                {
                    "doc": self.ref(h["doc_path"]),
                    "layer": h.get("layer"),
                    "chunk_id": h.get("chunk_id"),
                }
                for h in r.get("hits", [])
            ]
            qres[q["id"]] = hits
            fails += check_obsidian_query(q, hits)
        self.numbers["search_ms"] = {"p50": pct(qlat, 50), "p95": pct(qlat, 95), "n": len(qlat)}
        self.step("O10", not fails, failures=fails[:20], n_failures=len(fails))
        # O11 blame of touched documents
        bl = {}
        for p in list_md(self.vault):
            if p.startswith(SYNTHETIC) or self.ref(p) in {x["doc"] for x in recs}:
                c = {}
                for b in self.srv.http("blame", {"doc_path": p}).get("blocks", []):
                    c[b["layer"]] = c.get(b["layer"], 0) + 1
                bl[self.ref(p)] = c
        self.step("O11", bool(bl), documents=len(bl))
        # O12 md bytes
        final_md = self.md()
        rootmark_md = sum(
            1 for p in set(final_md) | set(md_cur) if final_md.get(p) != md_cur.get(p)
        )
        ok12 = rootmark_md == 0 and all(x["only_target"] for x in self.md_ok)
        self.step(
            "O12",
            ok12,
            md_changes_not_by_obsidian_or_agent=rootmark_md,
            operations=len(self.md_ok),
            outside_target=[x for x in self.md_ok if not x["only_target"]],
        )
        # counts for O14 while the server runs
        counts = {}
        for p in list_md(self.vault):
            for b in self.srv.http("blame", {"doc_path": p}).get("blocks", []):
                if not b.get("inactive"):
                    counts[b["layer"]] = counts.get(b["layer"], 0) + 1
        status = self.srv.http("status")
        sb2 = pg.ev("__p2q.attestBar()")
        # O13 stop
        stop = self.srv.close(signal.SIGTERM)
        log = (self.work / "serve.log").read_text(errors="replace")
        panics = len(re.findall(r"panicked|panic", log))
        self.numbers["serve_peak_rss_kib"] = stop["peak_rss_kib"]
        attest = self.attest_summary(att)
        self.step(
            "O13",
            stop["rc"] == 0 and panics == 0,
            sigterm_rc=stop["rc"],
            panics=panics,
            attest_bar=sb2,
            attest=attest,
        )
        # O14 observe
        stf = self.work / "status.json"
        stf.write_text(json.dumps(status))
        snap0, a0 = tree_b2(self.vault), b2(att.read_bytes())
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
        ro = tree_b2(self.vault) == snap0 and b2(att.read_bytes()) == a0
        layers = {
            k: v for k, v in obs.get("layers", {}).get("lineages", {}).items() if k != "inactive"
        }
        cmp = [
            {
                "what": "promotions by the human (lineages)",
                "driver": self.promoted_human,
                "observe": obs.get("promotions", {}).get("lineages_by_actor", {}).get("human", 0),
            },
            {
                "what": "demotions by the human (lineages)",
                "driver": self.demoted_human,
                "observe": obs.get("demotions", {}).get("lineages_by_actor", {}).get("human", 0),
            },
            {"what": "live lineages per layer", "driver": counts, "observe": layers},
        ]
        for c in cmp:
            c["equal"] = c["driver"] == c["observe"]
        self.step(
            "O14",
            r.returncode == 0 and ro and all(c["equal"] for c in cmp),
            read_only=ro,
            unequal=[c for c in cmp if not c["equal"]],
        )
        sizes = sum(
            (Path(d) / f).stat().st_size
            for d, _, fs in os.walk(self.vault / ".rootmark")
            for f in fs
        )
        self.numbers["rootmark_dir_bytes"] = sizes
        dump(self.steps, self.out / "steps.json")
        dump({"edits": recs, "renames": rn, "md_operations": self.md_ok}, self.out / "edits.json")
        dump(qres, self.out / "queries.json")
        dump(self.numbers, self.out / "numbers.json")
        dump(attest, self.out / "attest.json")
        dump(obs, self.out / "observe.json")
        dump(cmp, self.out / "observe_compare.json")
        dump(bl, self.out / "blame.json")
        passed = sum(s["pass"] for s in self.steps)
        print(json.dumps({"passed": passed, "total": len(self.steps)}))
        return 0 if passed == len(self.steps) else 1

    def attest_summary(self, att: Path) -> dict:
        """Accepted/refused by kind, refusals with their reason (personal
        paths replaced by their k2), observations by outcome."""
        out = {"attest": {}, "refused": [], "observe": {}}
        names = sorted(list_md(self.vault) + self.paths, key=len, reverse=True)
        for line in att.read_text(errors="replace").splitlines():
            try:
                r = json.loads(line)
            except ValueError:
                continue
            if r.get("event") == "attest":
                k = f"{r.get('kind')}:{r.get('result')}"
                out["attest"][k] = out["attest"].get(k, 0) + 1
                if r.get("result") == "refused":
                    reason = str(r.get("reason", ""))
                    for n in names:
                        if n in reason:
                            reason = reason.replace(n, self.ref(n))
                    out["refused"].append(
                        {
                            "kind": r.get("kind"),
                            "code": r.get("code"),
                            "doc": self.ref(r.get("doc_path", "")),
                            "reason": reason[:400],
                        }
                    )
            elif r.get("event") == "observe":
                k = str(r.get("ingest"))
                out["observe"][k] = out["observe"].get(k, 0) + 1
        return out


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--snapshot", required=True, type=Path)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--guide", required=True, type=Path)
    ap.add_argument("--guide-home", required=True, type=Path)
    ap.add_argument("--rootmark", default="rootmark")
    ap.add_argument("--obsidian", default=OBSIDIAN)
    a = ap.parse_args(argv)
    if not os.environ.get("ROOTMARK_MODELS"):
        raise SystemExit("ROOTMARK_MODELS is not set")
    if shutil.which(a.obsidian) is None and not Path(a.obsidian).exists():
        raise SystemExit(f"no Obsidian at {a.obsidian}")
    return Scene(a).go()


if __name__ == "__main__":
    sys.exit(main())
