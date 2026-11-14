"""D-1/R4, R6 — the container image's quick start, as the guide writes it.

On a synthetic vault (a copy of spec/fixtures/mini_vault in a scratch
directory, never personal content), with a given image:
  1. `docker run --rm IMG conformance`                       → 39/39
  2. `docker run --rm --user U:G -v V:/vault IMG ingest-dir /vault /vault`
  3. `docker run --rm --user U:G -v V:/vault IMG index build /vault`
  4. the image's default command as an agent starts it
     (`docker run -i --rm --user U:G -v V:/vault IMG`): an MCP stdio session
     — initialize, tools/list, `propose` (a new note), `search` with
     include_proposal finds it — and, while that server runs, `--watch`:
     a note of the mounted vault is edited on the host and `search` returns
     the new text (polled, at most --watch-timeout seconds).
Also: the process's uid (`id -u` ≠ 0), the model licence and attribution
files, THIRD_PARTY_LICENSES in the image, the image size. Writes a record
with no host value (no path, user or host name) and exits 1 unless every
step passed.

    python3 -I research/d1/image_quickstart.py --image IMG --platform-label linux \
        --vault-src spec/fixtures/mini_vault --work <scratch> --out <record.json>
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import threading
import time
from pathlib import Path

WORD = "zebracoffeeprobe"  # the --watch probe word (in no fixture)
PROPOSE_WORD = "quokkaroastprobe"


def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, check=False, **kw)


class Mcp:
    """A line-delimited JSON-RPC session over the container's stdio."""

    def __init__(self, cmd):
        self.p = subprocess.Popen(
            cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )
        self.next_id = 0
        self.stderr: list[str] = []
        threading.Thread(target=self._drain, daemon=True).start()

    def _drain(self):
        for line in self.p.stderr:
            self.stderr.append(line.rstrip("\n"))

    def send(self, msg):
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()

    def call(self, method, params, timeout=300):
        self.next_id += 1
        mid = self.next_id
        self.send({"jsonrpc": "2.0", "id": mid, "method": method, "params": params})
        t0 = time.monotonic()
        while time.monotonic() - t0 < timeout:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError(f"{method}: the server closed stdout")
            msg = json.loads(line)
            if msg.get("id") == mid:
                return msg
        raise RuntimeError(f"{method}: no answer in {timeout}s")

    def tool(self, name, args):
        r = self.call("tools/call", {"name": name, "arguments": args})
        res = r.get("result") or {}
        if res.get("isError") or "error" in r:
            return False, res.get("structuredContent") or r.get("error")
        return True, res.get("structuredContent")

    def close(self):
        try:
            self.p.stdin.close()
            self.p.wait(timeout=60)
        except Exception:  # noqa: BLE001
            self.p.kill()
        return self.p.returncode


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--image", required=True)
    ap.add_argument("--platform-label", required=True)
    ap.add_argument("--docker-platform", default="linux/amd64")
    ap.add_argument("--vault-src", required=True, type=Path)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--watch-timeout", type=float, default=120.0)
    ap.add_argument("--watch-reps", type=int, default=3)
    a = ap.parse_args()
    if a.work.exists():
        shutil.rmtree(a.work)
    vault = a.work / "vault"
    shutil.copytree(a.vault_src, vault)
    for junk in vault.rglob(".rootmark"):
        shutil.rmtree(junk)
    ug = f"{os.getuid()}:{os.getgid()}"
    base = ["docker", "run", "--rm", "--platform", a.docker_platform]
    mount = ["--user", ug, "-v", f"{vault}:/vault"]
    steps = {}

    def step(name, ok, **detail):
        steps[name] = {"pass": bool(ok), **detail}
        print(json.dumps({name: steps[name]}), flush=True)

    t0 = time.monotonic()
    r = run([*base, a.image, "conformance"])
    try:
        c = json.loads(r.stdout)
        step(
            "1_conformance",
            r.returncode == 0 and c["passed"] == c["total"] == 39 and c["level_claimed"] == 3,
            passed=c["passed"],
            total=c["total"],
            level=c["level_claimed"],
            s=round(time.monotonic() - t0, 1),
        )
    except (ValueError, KeyError):
        step("1_conformance", False, exit=r.returncode)
    t0 = time.monotonic()
    r = run([*base, *mount, a.image, "ingest-dir", "/vault", "/vault"])
    try:
        ing = json.loads(r.stdout)
        step(
            "2_ingest_dir",
            r.returncode == 0,
            documents=len(ing.get("ingested", [])),
            events_hash=ing.get("events_hash"),
            s=round(time.monotonic() - t0, 1),
        )
    except ValueError:
        step("2_ingest_dir", False, exit=r.returncode)
    t0 = time.monotonic()
    r = run([*base, *mount, a.image, "index", "build", "/vault"])
    step("3_index_build", r.returncode == 0, exit=r.returncode, s=round(time.monotonic() - t0, 1))

    # 4: the default command (serve --vault /vault --mcp stdio --watch), as an agent starts it
    t0 = time.monotonic()
    m = Mcp(["docker", "run", "-i", "--rm", "--platform", a.docker_platform, *mount, a.image])
    try:
        init = m.call(
            "initialize",
            {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "d1-image-quickstart", "version": "1.0"},
            },
        )
        m.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
        tools = sorted(t["name"] for t in m.call("tools/list", {})["result"]["tools"])
        ok_p, _prop = m.tool(
            "propose",
            {
                "doc_path": "agent/probe.md",
                "reason": "D-1 image quick start",
                "content": f"# Probe\n\nThe agent notes a {PROPOSE_WORD} here.\n",
            },
        )
        ok_s, srch = m.tool(
            "search", {"query": PROPOSE_WORD, "include_proposal": True, "k": 5, "mode": "lexical"}
        )
        found = ok_s and any(h["doc_path"] == "agent/probe.md" for h in srch.get("hits", []))
        ok_d, dflt = m.tool("search", {"query": PROPOSE_WORD, "k": 5, "mode": "lexical"})
        hidden = ok_d and not any(h["doc_path"] == "agent/probe.md" for h in dflt.get("hits", []))
        step(
            "4_mcp_stdio",
            bool(init.get("result")) and len(tools) == 11 and ok_p and found and hidden,
            server=init.get("result", {}).get("serverInfo", {}).get("name"),
            tools=len(tools),
            propose=ok_p,
            search_include_proposal_finds=found,
            default_search_hides_proposal=hidden,
            s=round(time.monotonic() - t0, 1),
        )
        # --watch: a human edit of a mounted note while the server runs,
        # repeated on different notes (the word is absent before the edit)
        reps = []
        for i, note in enumerate(sorted(vault.glob("*.md"))[: a.watch_reps]):
            word = f"{WORD}{i}"
            ok_b, before = m.tool("search", {"query": word, "k": 5, "mode": "lexical"})
            absent = ok_b and not before.get("hits")
            time.sleep(2)
            with note.open("a", encoding="utf-8") as fh:
                fh.write(f"\n## Watch probe {i}\n\nA {word} was added while the server ran.\n")
            t1 = time.monotonic()
            seen = False
            while time.monotonic() - t1 < a.watch_timeout:
                ok_w, w = m.tool("search", {"query": word, "k": 5, "mode": "lexical"})
                if ok_w and any(h["doc_path"] == note.name for h in w.get("hits", [])):
                    seen = True
                    break
                time.sleep(0.5)
            reps.append(
                {"absent_before": absent, "found": seen, "after_s": round(time.monotonic() - t1, 2)}
            )
        step(
            "5_watch_edit_found",
            len(reps) == a.watch_reps and all(r["absent_before"] and r["found"] for r in reps),
            reps=reps,
        )
    except Exception as e:  # noqa: BLE001
        step("4_mcp_stdio", False, error=str(e)[:200])
    finally:
        rc = m.close()
    steps["4_mcp_stdio"]["exit_after_close"] = rc

    r = run([*base, "--entrypoint", "id", a.image, "-u"])
    step(
        "6_non_root", r.returncode == 0 and r.stdout.strip() not in ("", "0"), uid=r.stdout.strip()
    )
    r = run(
        [
            *base,
            "--entrypoint",
            "ls",
            a.image,
            "/usr/share/rootmark/models/minilm-l6-v2",
            "/usr/share/doc/rootmark",
        ]
    )
    names = r.stdout.split()
    step(
        "7_licences_in_image",
        all(n in names for n in ("LICENSE", "ATTRIBUTION", "NOTICE", "THIRD_PARTY_LICENSES")),
        files=sorted(
            set(names) & {"LICENSE", "ATTRIBUTION", "NOTICE", "THIRD_PARTY_LICENSES", "model.onnx"}
        ),
    )
    r = run([*base, a.image, "--version"])
    step("8_version", r.returncode == 0, version=r.stdout.strip())
    r = run(["docker", "image", "inspect", a.image, "--format", "{{.Size}}"])
    size = int(r.stdout.strip() or 0)
    rec = {
        "what": "D-1 image quick start (research/d1/image_quickstart.py)",
        "platform": a.platform_label,
        "docker_platform": a.docker_platform,
        "image": a.image,
        "vault": "copy of spec/fixtures/mini_vault (synthetic)",
        "image_size_mb": round(size / 1e6, 1),
        "steps": steps,
        "pass": all(s["pass"] for s in steps.values()),
    }
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(rec, indent=1) + "\n", encoding="utf-8")
    print(json.dumps({"pass": rec["pass"]}))
    return 0 if rec["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
