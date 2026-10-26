"""P2-R D-VOL (GOAL P2-R 4, R-7; pre-registered before the measurement):
the volume of `.rootmark` before and after the explicit prune.

Scenario (deterministic, `eval.datasets.d_vol` from `SEED`), against a real
`rootmark serve --http` (its HTTP API) and the `rootmark` CLI:

- 200 human notes, ingested (`rootmark ingest-dir`) and indexed;
- 40 rounds; each round: 25 notes edited (the file written, then `rootmark
  index watch` — a human edit, indexed), 10 `propose` (HTTP; agent
  proposals `agents/r<r>/p<j>.md`), then 50 `search` (half with
  `include_proposal`) and 10 `read_block` (a block of a note picked by the
  generator, through `blame`), then one `checkpoint`; at the start of round
  2 both lineages of round 1's first 5 proposals (10 lineages) are promoted
  to `canonical` (HTTP `promote`; they must never be pruned);
- recipe: `[retention] keep_checkpoints = 5`, `[[prune.rules]]` `proposal`
  `idle_checkpoints = 10`;
- `V` = bytes of `traces.jsonl` + the live checkpoint files
  (`checkpoints/*.json`) + `loc/` + `journal/`; `events.jsonl`, `objects/`,
  `snapshots/`, `attestations.jsonl`, `trace_filters.jsonl` and the archive
  are reported apart (never pruned);
- `V_before` after the 40 rounds; `rootmark prune` (CLI, `--now-ms` fixed;
  the server stopped) → `V_after`; 10 more rounds and `rootmark prune` →
  `V_after2`. The live traces joined after the archive must be the
  `traces.jsonl` of `V_before`, byte for byte.

R-7: (a) `V_after / V_before` ≤ 0.35, (b) `V_after2 / V_after` ≤ 1.10.

    python -m eval.p2r_volume --run <RUN> --rootmark <binary>

Writes `p2r/volume.json` (no time, no path). Exit 0, 3 when R-7 fails
(recorded, not optimized: GOAL decision rule), 1 on an error.
"""

from __future__ import annotations

import argparse
import json
import signal
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
from pathlib import Path

from eval import runinfo
from eval.datasets import d_vol
from eval.datasets.common import dump_json

ROUNDS = 40
ROUNDS2 = 10
EDITS = 25
PROPOSALS = 10
SEARCHES = 50
READS = 10
PROMOTED_DOCS = 5
KEEP_CHECKPOINTS = 5
IDLE_CHECKPOINTS = 10
NOW_MS = 1_800_000_000_000
A_MAX = 0.35
B_MAX = 1.10

RECIPE = (
    '[promotion]\npolicy = "human_only"\n\n[edit]\npolicy = "append_only"\n\n'
    "[version_trigger]\nidle_ms = 30000\nmin_sim_change = 0.2\n\n[journal]\nretention_hours = 24\n\n"
    "[server]\ntoken_rotate_days = 30\nmax_propose_bytes = 1048576\n\n"
    "[index]\ncompact_every_s = 86400\n\n"
    '[identity]\nhuman_id = "vol"\n\n[attestation]\nunattested = "canonical"\n\n'
    f"[retention]\nkeep_checkpoints = {KEEP_CHECKPOINTS}\n\n"
    '[prune]\nversion = "d-vol"\n\n'
    f'[[prune.rules]]\nid = "stale"\nlayer = "proposal"\nidle_checkpoints = {IDLE_CHECKPOINTS}\n'
)


def run(cmd: list[str]) -> dict:
    r = subprocess.run(cmd, capture_output=True, check=False)
    if r.returncode != 0:
        raise SystemExit(f"p2r_volume: {' '.join(cmd[1:3])} failed: {r.stderr.decode()}")
    return json.loads(r.stdout or b"{}")


class Server:
    def __init__(self, rootmark: str, vault: Path) -> None:
        self.p = subprocess.Popen(
            [rootmark, "serve", "--vault", str(vault), "--http", "127.0.0.1:0"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        assert self.p.stderr is not None
        prefix = "rootmark serve: http listening on 127.0.0.1:"
        while True:
            line = self.p.stderr.readline().decode()
            if not line:
                raise SystemExit("p2r_volume: rootmark serve did not start")
            if line.startswith(prefix):
                self.port = int(line[len(prefix) :].strip())
                break
        self.token = (vault / ".rootmark" / "server.token").read_text().strip()

    def call(self, tool: str, body: dict) -> dict:
        req = urllib.request.Request(
            f"http://127.0.0.1:{self.port}/v1/{tool}",
            data=json.dumps(body).encode(),
            headers={
                "Authorization": f"Bearer {self.token}",
                "X-Consumer": "d-vol",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        try:
            with urllib.request.urlopen(req) as r:
                return json.loads(r.read())
        except urllib.error.HTTPError as e:
            raise SystemExit(f"p2r_volume: {tool} {body}: {e.code} {e.read().decode()}") from e

    def stop(self) -> None:
        self.p.send_signal(signal.SIGTERM)
        if self.p.wait(timeout=600) != 0:
            raise SystemExit("p2r_volume: rootmark serve did not stop cleanly")


def size(p: Path) -> int:
    if not p.exists():
        return 0
    if p.is_file():
        return p.stat().st_size
    return sum(f.stat().st_size for f in p.rglob("*") if f.is_file())


def volume(v: Path) -> dict:
    m = v / ".rootmark"
    live_cp = sum(f.stat().st_size for f in (m / "checkpoints").glob("*.json"))
    parts = {
        "traces": size(m / "traces.jsonl"),
        "checkpoints": live_cp,
        "loc": size(m / "loc"),
        "journal": size(m / "journal"),
    }
    apart = {
        "events": size(m / "events.jsonl"),
        "objects": size(m / "objects"),
        "snapshots": size(m / "snapshots"),
        "attestations": size(m / "attestations.jsonl"),
        "trace_filters": size(m / "trace_filters.jsonl"),
        "archive_traces": size(m / "traces" / "archive"),
        "archive_checkpoints": size(m / "checkpoints" / "archive"),
    }
    return {"V": sum(parts.values()), "parts": parts, "apart": apart}


def rounds(rootmark: str, v: Path, gen: d_vol.Gen, first: int, last: int, st: dict) -> None:
    srv = Server(rootmark, v)
    try:
        for r in range(first, last + 1):
            if r == 2:
                for res in st["round1"][:PROMOTED_DOCS]:
                    for b in res["blocks"]:
                        out = srv.call(
                            "promote", {"lineage_id": b["lineage_id"], "reason": "d-vol keeps it"}
                        )
                        if out.get("decision") != "promoted":
                            raise SystemExit(f"p2r_volume: promote refused: {out}")
                        st["promoted"].append(b["lineage_id"])
            for i in gen.edited(r, EDITS):
                p, text = gen.edit(i, r)
                (v / p).write_text(text, encoding="utf-8")
                run([rootmark, "index", "watch", str(v), p, "--source", "d-vol-human"])
            for j in range(PROPOSALS):
                p, text = gen.proposal(r, j)
                res = srv.call("propose", {"doc_path": p, "content": text, "reason": "d-vol"})
                if r == 1:
                    st["round1"].append(res)
            for j in range(SEARCHES):
                srv.call("search", {"query": gen.query(), "include_proposal": j % 2 == 0})
            for _ in range(READS):
                doc = gen.note_path(gen.pick(d_vol.NOTES))
                blocks = srv.call("blame", {"doc_path": doc})["blocks"]
                b = blocks[gen.pick(len(blocks))]
                srv.call("read_block", {"doc_path": doc, "lineage_id": b["lineage_id"]})
            srv.call("checkpoint", {})
    finally:
        srv.stop()


def prune(rootmark: str, v: Path) -> dict:
    out = run([rootmark, "prune", str(v), "--now-ms", str(NOW_MS)])
    r = out["retention"]
    return {
        "events_hash_before": out["events_hash_before"],
        "events_hash": out["events_hash"],
        "pruned_lineages": len(out["rules"]["pruned"]),
        "trace_lines_archived": r["traces"]["lines"],
        "trace_base": r["traces"]["base"],
        "filter_lines_archived": r["traces"]["filter_lines"],
        "checkpoints_archived": len(r["checkpoints"]),
        "loc_removed": len(r["loc"]["removed"]),
        "loc_kept": r["loc"]["kept"],
        "journal_records_removed": out["journal"]["records_removed"],
        "bound": r["bound"],
    }


def joined_traces(v: Path) -> bytes:
    m = v / ".rootmark"
    files = sorted(
        (
            p
            for p in (m / "traces" / "archive").glob("*.jsonl")
            if not p.name.endswith(".filters.jsonl")
        ),
        key=lambda p: int(p.name.split("-")[0]),
    )
    return b"".join(p.read_bytes() for p in files) + (m / "traces.jsonl").read_bytes()


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--run", required=True)
    ap.add_argument("--rootmark", required=True)
    a = ap.parse_args(argv)
    rootmark = str(Path(a.rootmark).resolve())
    out_dir = runinfo.run_dir(a.run, create=False) / "p2r"
    out_dir.mkdir(parents=True, exist_ok=True)
    gen = d_vol.Gen()
    st: dict = {"round1": [], "promoted": []}
    with tempfile.TemporaryDirectory(prefix="rootmark-p2r-vol-") as tmp:
        v = Path(tmp) / "vault"
        for p, text in gen.initial().items():
            (v / p).parent.mkdir(parents=True, exist_ok=True)
            (v / p).write_text(text, encoding="utf-8")
        run([rootmark, "ingest-dir", str(v), str(v)])
        (v / ".rootmark" / "policy.toml").write_text(RECIPE, encoding="utf-8")
        run([rootmark, "index", "build", str(v)])
        rounds(rootmark, v, gen, 1, ROUNDS, st)
        before = volume(v)
        traces_before = (v / ".rootmark" / "traces.jsonl").read_bytes()
        p1 = prune(rootmark, v)
        after = volume(v)
        join_equal = joined_traces(v) == traces_before
        rounds(rootmark, v, gen, ROUNDS + 1, ROUNDS + ROUNDS2, st)
        mid = volume(v)
        p2 = prune(rootmark, v)
        after2 = volume(v)
        canon = []
        for lid in st["promoted"]:
            h = run([rootmark, "history", str(v), lid])
            pruned = any(e["kind"] == "Demoted" and e["meta"].get("prune") for e in h["events"])
            canon.append(h["layer"] == "canonical" and not pruned)
    ra = after["V"] / before["V"]
    rb = after2["V"] / after["V"]
    res = {
        "scenario": {
            "notes": d_vol.NOTES,
            "rounds": ROUNDS,
            "rounds_phase2": ROUNDS2,
            "edits_per_round": EDITS,
            "proposals_per_round": PROPOSALS,
            "searches_per_round": SEARCHES,
            "reads_per_round": READS,
            "promoted_lineages": len(st["promoted"]),
            "keep_checkpoints": KEEP_CHECKPOINTS,
            "idle_checkpoints": IDLE_CHECKPOINTS,
            "now_ms": NOW_MS,
        },
        "before": before,
        "prune1": p1,
        "after": after,
        "phase2_before_prune": mid,
        "prune2": p2,
        "after2": after2,
        "traces_joined_equal": join_equal,
        "promoted_never_pruned": len(canon) == 2 * PROMOTED_DOCS and all(canon),
        "V_before": before["V"],
        "V_after": after["V"],
        "V_after2": after2["V"],
        "ratio_a": ra,
        "ratio_b": rb,
    }
    ok = ra <= A_MAX and rb <= B_MAX
    res["criteria"] = {
        "R-7": {"a": ra, "a_max": A_MAX, "b": rb, "b_max": B_MAX, "pass": ok},
    }
    res["pass"] = ok and join_equal and res["promoted_never_pruned"]
    (out_dir / "volume.json").write_text(dump_json(res), encoding="utf-8")
    print(
        f"p2r-volume: V_before {before['V']}, V_after {after['V']} (a {ra:.4f}, "
        f"{'pass' if ra <= A_MAX else 'FAIL'}), V_after2 {after2['V']} (b {rb:.4f}, "
        f"{'pass' if rb <= B_MAX else 'FAIL'}); joined traces equal {join_equal}; "
        f"promoted never pruned {res['promoted_never_pruned']}"
    )
    if not (join_equal and res["promoted_never_pruned"]):
        return 1
    return 0 if ok else 3


if __name__ == "__main__":
    sys.exit(main())
