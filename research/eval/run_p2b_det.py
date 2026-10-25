"""P2-B determinism record (GOAL P2-B B-9; `make p2b-checks`): the outputs
of the round's three surfaces written whole into the RUN, so that the pair
comparison (`eval.compare_p2b`) sees two chains' bytes:

- **locations:** `rootmark blame --loc` of every document of a vault made of
  D-NOTES' notes (every 16th note of the manifest without a hidden path, as
  the Obsidian e2e) and two dated notes: per document `loc_stale` and each
  block's `version_id` and `loc`; then one note changed on disk and not
  ingested: its `loc_stale` and locations (the ingested version's);
- **events:** over MCP (`rootmark serve --mcp stdio`), the whole log in pages
  of 1000 (the text content: each event its log line) — its sha256 and
  length, and that it equals `events.jsonl`; one filtered page (`kinds`,
  `path_prefix`) whole;
- **time filter:** a search with `date_from`/`date_to` (dates from `[index]
  date_from_path`) and `loc: true`, and one with a path prefix and dates,
  whole (checkpoint, query hash, hits, filter, `undated_excluded`), and the
  traces they wrote.

    python -m eval.run_p2b_det --run <RUN> --rootmark <binary>

Writes `p2b/det.json` (no time, no path). Exit 0, 1 on an error.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

from eval import runinfo
from eval.datasets import d_notes
from eval.datasets.common import dump_json

DATED = {
    "journal/2026/10/24.md": "# Dün\n\nÇay demlendi, kahve beklendi.\n",
    "journal/2026/10/25.md": "# Bugün\n\nÇay yine demlendi.\n\n    girintili not\n",
}
RECIPE = (
    '[promotion]\npolicy = "human_only"\n\n[edit]\npolicy = "append_only"\n\n'
    "[version_trigger]\nidle_ms = 30000\nmin_sim_change = 0.2\n\n[journal]\nretention_hours = 168\n\n"
    "[server]\ntoken_rotate_days = 30\nmax_propose_bytes = 1048576\n\n"
    '[index]\ncompact_every_s = 300\ndate_from_path = "{yyyy}/{mm}/{dd}"\n\n'
    '[identity]\nhuman_id = "ada"\n\n[attestation]\nunattested = "canonical"\n'
)


def run(cmd: list[str]) -> str:
    r = subprocess.run(cmd, capture_output=True, check=False)
    if r.returncode != 0:
        raise SystemExit(f"run_p2b_det: {' '.join(cmd[1:3])} failed: {r.stderr.decode()}")
    return r.stdout.decode("utf-8")


class Mcp:
    def __init__(self, rootmark: str, vault: Path) -> None:
        self.p = subprocess.Popen(
            [rootmark, "serve", "--vault", str(vault), "--mcp", "stdio"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
        self.n = 0
        self.rpc(
            "initialize",
            {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "p2b-det", "version": "1"},
            },
        )
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def send(self, m: dict) -> None:
        assert self.p.stdin is not None
        self.p.stdin.write((json.dumps(m) + "\n").encode())
        self.p.stdin.flush()

    def rpc(self, method: str, params: dict) -> dict:
        self.n += 1
        self.send({"jsonrpc": "2.0", "id": self.n, "method": method, "params": params})
        assert self.p.stdout is not None
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise SystemExit(f"run_p2b_det: serve exited during {method}")
            m = json.loads(line)
            if m.get("id") == self.n:
                return m.get("result", {"rpc_error": m.get("error")})

    def tool(self, name: str, args: dict) -> dict:
        r = self.rpc("tools/call", {"name": name, "arguments": args})
        if r.get("isError"):
            raise SystemExit(f"run_p2b_det: {name}: {r}")
        return r

    def close(self) -> None:
        assert self.p.stdin is not None
        self.p.stdin.close()
        self.p.wait(timeout=120)


def blame(rootmark: str, v: Path, doc: str) -> dict:
    b = json.loads(run([rootmark, "blame", "--loc", str(v), doc]))
    return {
        "loc_stale": b["loc_stale"],
        "blocks": [[x["version_id"], x.get("loc")] for x in b["blocks"]],
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--run", required=True)
    ap.add_argument("--rootmark", required=True)
    a = ap.parse_args(argv)
    rootmark = str(Path(a.rootmark).resolve())
    m = json.loads(d_notes.MANIFEST.read_text(encoding="utf-8"))
    notes = [
        n["path"] for n in m["notes"] if not any(c.startswith(".") for c in n["path"].split("/"))
    ]
    notes = [p for i, p in enumerate(notes) if i % 16 == 0]
    out: dict = {"notes": len(notes)}
    with tempfile.TemporaryDirectory(prefix="rootmark-p2b-det-") as tmp:
        v = Path(tmp) / "vault"
        for p in notes:
            f = v / p
            f.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(d_notes.NOTES / p, f)
        for p, t in DATED.items():
            f = v / p
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text(t, encoding="utf-8")
        run([rootmark, "ingest-dir", str(v), str(v)])
        (v / ".rootmark/policy.toml").write_text(RECIPE, encoding="utf-8")
        docs = sorted([*notes, *DATED])
        out["blame"] = {d: blame(rootmark, v, d) for d in docs}
        log = (v / ".rootmark/events.jsonl").read_bytes()
        s = Mcp(rootmark, v)
        loglines = log.decode("utf-8").splitlines()
        pages, since, equal = 0, 0, True
        while True:
            r = s.tool("events", {"since_seq": since, "limit": 1000})
            text, page = r["content"][0]["text"], r["structuredContent"]
            nxt, head = page["next_seq"], page["head_seq"]
            # the text content is the page with each event its log line
            want = (
                '{"events":['
                + ",".join(loglines[since:nxt])
                + f'],"head_seq":{head},"next_seq":{nxt}}}'
            )
            equal = equal and text == want
            pages += 1
            if nxt == head:
                break
            since = nxt
        out["events"] = {
            "count": len(loglines),
            "head_seq": head,
            "pages": pages,
            "log_sha256": hashlib.sha256(log).hexdigest(),
            "text_is_the_log_lines": equal,
        }
        out["events_filtered"] = s.tool(
            "events", {"kinds": ["SnapshotCommitted"], "path_prefix": ["journal"], "limit": 10}
        )["content"][0]["text"]
        out["search_dated"] = s.tool(
            "search",
            {
                "query": "çay demlendi",
                "mode": "lexical",
                "date_from": "2026-10-25",
                "date_to": "2026-10-25",
                "loc": True,
            },
        )["structuredContent"]
        out["search_prefixed_dated"] = s.tool(
            "search",
            {
                "query": "çay",
                "mode": "lexical",
                "path_prefix": ["journal"],
                "date_from": "2026-10-24",
                "loc": True,
            },
        )["structuredContent"]
        out["traces"] = s.tool("trace", {})["structuredContent"]
        s.close()
        # a note changed on disk, not ingested: the ingested version's locations
        stale = notes[0]
        (v / stale).write_bytes(b"Inserted first line.\n\n" + (v / stale).read_bytes())
        out["stale"] = {"doc": stale, **blame(rootmark, v, stale)}
    d = runinfo.run_dir(a.run, create=False) / "p2b"
    d.mkdir(parents=True, exist_ok=True)
    (d / "det.json").write_text(dump_json(out), encoding="utf-8")
    print(
        f"p2b-det: {len(out['blame'])} documents located, {out['events']['count']} events, "
        f"dated search {len(out['search_dated']['hits'])} hits, stale {out['stale']['loc_stale']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
