"""P2-0 measurements (GOAL P2-0 "Önceden kayıtlı eşikler"; TASKS Z5).

    python -m eval.run_p2_0 --phase tune --run <RUN> --rootmark <bin>
        [--obsidian <bin>] [--plugin <dir>]
    python -m eval.run_p2_0 --phase test --run <RUN> --rootmark <bin>
        [--obsidian <bin>] [--plugin <dir>]

`tune` runs the 100 D-B tuning sessions in the real Obsidian once per
window `W` ∈ {0.5, 1, 2, 3, 5} s and writes `p2_0/tuning.json` with the
chosen `W` (rule below, written before any measurement: Z5.md). `test` reads
the chosen `W` from `p2_0/tuning.json` of the same RUN and never looks at
another — the test sets are not read before `W` is fixed — and writes:

- `p2_0/d_e.json` (H-P2-0f): the three vaults written by `v0.1.0`
  (`research/data/p2_0_old_vaults/`) opened by this build: they open, every
  document replays to the snapshot `v0.1.0` recorded, `events_hash` is
  unchanged, and every block keeps the layer `v0.1.0` gave it (none becomes
  `unverified`; the mixed and scene vaults hold proposals, so "every block
  canonical" is read as "every block keeps its layer", ASSUMPTIONS Z5-3);
- `p2_0/d_a.json` (H-P2-0a, H-P2-0a′): 200 agent-write scenarios, each
  checked by `check_agent_write` (one function, every scenario);
  `p2_0/d_a/{events.jsonl, attestations.jsonl}` for the pair (H-P2-0g);
- `p2_0/d_c.json` (H-P2-0d): the 160 mixed ticks; every block written in
  them has a known author; accuracy = blocks whose layer is the author's
  (human `canonical`, agent `unverified`) over all of them;
  `p2_0/d_c/{events.jsonl, attestations.jsonl}` for the pair;
- `p2_0/d_b.json` (H-P2-0b, b′, c): the 200 D-B test sessions in the real
  Obsidian with the chosen `W` (`clients/obsidian/e2e/p2_0/sessions.ts`).
  Without an Obsidian binary D-B is recorded as skipped and H-P2-0b, c as
  not measured (GOAL P2-0 PROMPT).

Fail-fast: a server that does not start, an HTTP error, a step that does not
settle, a missing file — an exception, never a partial record.
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import re
import shutil
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

import blake3

from eval import runinfo
from eval.agent_fs_writer import apply_step
from eval.datasets import p2_0_sets as sets

REPO = Path(__file__).resolve().parents[2]
WINDOWS = [0.5, 1.0, 2.0, 3.0, 5.0]
OBSIDIAN = "/Applications/Obsidian.app/Contents/MacOS/Obsidian"
CHANGE_KINDS = {"BlockUpdated", "BlockDeleted", "BlockSplit", "BlockMerged", "BlockMoved"}


def content_hash(b: bytes) -> str:
    return blake3.blake3(b).hexdigest()[:32]


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def run(cmd: list[str], **kw) -> str:
    r = subprocess.run(cmd, check=False, capture_output=True, text=True, **kw)
    if r.returncode != 0:
        raise RuntimeError(f"{' '.join(cmd)}: exit {r.returncode}\n{r.stderr[-2000:]}")
    return r.stdout


# ---- a served vault --------------------------------------------------------------


class Served:
    """`rootmark serve --vault <v> --http 127.0.0.1 --watch` on a vault made
    from the mini vault, with the human id `editor` and `[attestation]`."""

    def __init__(self, rootmark: str, work: Path, window_s: float):
        self.rootmark = rootmark
        work.mkdir(parents=True, exist_ok=True)
        self.vault = work / "vault"
        shutil.copytree(sets.MINI, self.vault, ignore=shutil.ignore_patterns(".rootmark"))
        run([rootmark, "ingest-dir", str(self.vault), str(self.vault)])
        # the default recipe (`rootmark serve` writes it), then the port, id and policy
        subprocess.run(
            [rootmark, "serve", "--vault", str(self.vault), "--mcp", "stdio"],
            stdin=subprocess.DEVNULL,
            capture_output=True,
            check=True,
        )
        self.port = free_port()
        p = self.vault / ".rootmark/policy.toml"
        recipe = p.read_text(encoding="utf-8")
        recipe = recipe.replace("[server]\n", f"[server]\nhttp_port = {self.port}\n")
        recipe = recipe.replace(
            '# [identity]\n# human_id = "<your id>"\n', '[identity]\nhuman_id = "editor"\n'
        )
        recipe += f'\n[attestation]\nunattested = "unverified"\nwindow_s = {window_s}\n'
        p.write_text(recipe, encoding="utf-8")
        self.err = (work / "serve.err").open("w")
        self.proc = subprocess.Popen(
            [rootmark, "serve", "--vault", str(self.vault), "--http", "127.0.0.1", "--watch"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=self.err,
        )
        deadline = time.monotonic() + 120
        while "watching" not in (work / "serve.err").read_text():
            if self.proc.poll() is not None or time.monotonic() > deadline:
                raise RuntimeError(
                    f"rootmark serve did not start:\n{(work / 'serve.err').read_text()}"
                )
            time.sleep(0.05)
        self.token = (self.vault / ".rootmark/server.token").read_text().strip()
        self.session: tuple[str, bytes] | None = None

    def call(self, tool: str, body: dict) -> dict:
        req = urllib.request.Request(
            f"http://127.0.0.1:{self.port}/v1/{tool}",
            data=json.dumps(body).encode(),
            headers={
                "Authorization": f"Bearer {self.token}",
                "X-Consumer": "p2-0-measure",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                return json.loads(r.read())
        except urllib.error.HTTPError as e:
            raise RuntimeError(f"{tool}: HTTP {e.code}: {e.read()[:400]!r}") from e

    def blame(self, doc: str) -> list[dict] | None:
        try:
            return self.call("blame", {"doc_path": doc})["blocks"]
        except RuntimeError as e:
            if "HTTP 404" in str(e):
                return None
            raise

    def attest(self, doc: str, data: bytes) -> dict:
        """The plugin's protocol (contract §2.6): a session, then a MAC."""
        if self.session is None:
            s = self.call("attest_session", {})
            self.session = (s["session_id"], bytes.fromhex(s["key"]))
        sid, key = self.session
        h = content_hash(data)
        mac = hmac.new(key, f"rootmark-attest/1\n{doc}\n{h}\n".encode(), hashlib.sha256).hexdigest()
        return self.call(
            "attest", {"doc_path": doc, "content_hash": h, "session_id": sid, "mac": mac}
        )

    def quiesce(self, settle_s: float = 0.4, limit_s: float = 15.0) -> None:
        """Waits until the event log has not grown for `settle_s`."""
        p = self.vault / ".rootmark/events.jsonl"
        last, since = -1, time.monotonic()
        deadline = time.monotonic() + limit_s
        while True:
            n = p.stat().st_size
            if n != last:
                last, since = n, time.monotonic()
            elif time.monotonic() - since >= settle_s:
                return
            if time.monotonic() > deadline:
                raise RuntimeError("the vault did not settle (the watcher kept writing)")
            time.sleep(0.05)

    def stop(self) -> None:
        self.proc.send_signal(signal.SIGTERM)
        try:
            self.proc.wait(timeout=60)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            raise
        self.err.close()
        if self.proc.returncode != 0:
            raise RuntimeError(f"rootmark serve exit {self.proc.returncode}")


def events(vault: Path) -> list[dict]:
    return [
        json.loads(line) for line in (vault / ".rootmark/events.jsonl").read_text().splitlines()
    ]


# ---- D-A ---------------------------------------------------------------------------


def check_agent_write(
    sv: Served, sc: dict, checkpoint: dict, before: dict[str, list[dict] | None]
) -> dict:
    """THE check of one D-A scenario (H-P2-0a, H-P2-0a′), the same for all
    200: after the agent's file writes have settled,

    - leak (0a): every block of every document the scenario wrote whose
      version is new to that document (not in its blame before) must be
      `unverified`; a new or changed block that is `canonical` is a leak;
    - preserved (0a′): every `canonical` lineage of those documents before
      the scenario that the writes changed or removed (its version is gone
      from the document) still reads, as of the checkpoint taken before the
      scenario, with its old version; and its history holds a change or
      delete event after the checkpoint whose `meta.actor.kind` is
      `unattested_fs` (the actor `blame`/`history` show for the agent's write).
    """
    touched = sorted(
        {s["path"] for s in sc["steps"]} | {s.get("to") for s in sc["steps"] if s.get("to")}
    )
    leak, new_blocks, unverified = [], 0, 0
    lost, preserved = [], 0
    for doc in touched:
        after = sv.blame(doc) or []
        old = before.get(doc) or []
        old_versions = {b["version_id"] for b in old}
        for b in after:
            if b["version_id"] in old_versions:
                continue
            new_blocks += 1
            if b["layer"] == "canonical":
                leak.append({"doc": doc, "lineage_id": b["lineage_id"]})
            if b["layer"] == "unverified":
                unverified += 1
        live = {b["lineage_id"]: b["version_id"] for b in after}
        for b in old:
            if b["layer"] != "canonical" or live.get(b["lineage_id"]) == b["version_id"]:
                continue
            ok_read = False
            r = sv.call(
                "read_block",
                {"doc_path": doc, "lineage_id": b["lineage_id"], "at": checkpoint["checkpoint_id"]},
            )
            ok_read = r["block"]["block"]["version_id"] == b["version_id"]
            hist = sv.call("history", {"lineage_id": b["lineage_id"]})["events"]
            ok_actor = any(
                e["seq"] >= checkpoint["event_seq"]
                and e["kind"] in CHANGE_KINDS
                and (e["meta"].get("actor") or {}).get("kind") == "unattested_fs"
                for e in hist
            )
            if ok_read and ok_actor:
                preserved += 1
            else:
                lost.append(
                    {"doc": doc, "lineage_id": b["lineage_id"], "read": ok_read, "actor": ok_actor}
                )
    return {
        "id": sc["id"],
        "kind": sc["kind"],
        "documents": touched,
        "new_blocks": new_blocks,
        "unverified": unverified,
        "leak": leak,
        "canonical_changed": preserved + len(lost),
        "preserved": preserved,
        "lost": lost,
        "pass_0a": not leak and new_blocks > 0,
        "pass_0a_prime": not lost,
    }


def run_d_a(rootmark: str, window_s: float, out: Path, work: Path) -> dict:
    data = sets.d_a()
    sv = Served(rootmark, work, window_s)
    rows = []
    try:
        for sc in data["scenarios"]:
            cp = sv.call("checkpoint", {})
            paths = sorted(
                {s["path"] for s in sc["steps"]} | {s.get("to") for s in sc["steps"] if s.get("to")}
            )
            before = {p: sv.blame(p) for p in paths}
            for st in sc["steps"]:
                apply_step(sv.vault, st)
            sv.quiesce()
            rows.append(check_agent_write(sv, sc, cp, before))
    finally:
        sv.stop()
    (out / "d_a").mkdir(parents=True, exist_ok=True)
    for f in ["events.jsonl", "attestations.jsonl"]:
        src = sv.vault / ".rootmark" / f
        (out / "d_a" / f).write_bytes(src.read_bytes() if src.exists() else b"")
    leaks = sum(len(r["leak"]) for r in rows)
    res = {
        "set": "D-A",
        "scenarios": len(rows),
        "new_blocks": sum(r["new_blocks"] for r in rows),
        "canonical_leak": leaks,
        "pass_0a": leaks == 0 and all(r["pass_0a"] for r in rows),
        "scenarios_0a_prime": sum(r["pass_0a_prime"] for r in rows),
        "canonical_changed": sum(r["canonical_changed"] for r in rows),
        "pass_0a_prime": all(r["pass_0a_prime"] for r in rows) and len(rows) == 200,
        "by_kind": {k: sum(1 for r in rows if r["kind"] == k) for k in sets.D_A_KINDS},
        "rows": rows,
    }
    return res


# ---- D-C ---------------------------------------------------------------------------


def run_d_c(rootmark: str, window_s: float, out: Path, work: Path) -> dict:
    data = sets.d_c()
    sv = Served(rootmark, work, window_s)
    try:
        for t in data["ticks"]:
            h, a = t["human"], t["agent"]
            hb = h["text"].encode()
            p = sv.vault / h["path"]
            p.write_bytes(hb)
            sv.attest(h["path"], hb)
            time.sleep(t["agent_delay_ms"] / 1000)
            apply_step(sv.vault, {"op": "write", "path": a["path"], "text": a["text"]})
            sv.quiesce()
        time.sleep(window_s + 0.5)
        docs = sorted(
            {t["human"]["path"] for t in data["ticks"]}
            | {t["agent"]["path"] for t in data["ticks"]}
        )
        blocks = []
        for d in docs:
            for b in sv.blame(d) or []:
                text = sv.call("read_block", {"doc_path": d, "lineage_id": b["lineage_id"]})[
                    "block"
                ]["block"]["text"]
                m = re.search(r"\b(Agent|Human) (c\d{3})", text)
                if m is None:
                    continue
                author, tick = m.group(1).lower(), m.group(2)
                want = "canonical" if author == "human" else "unverified"
                blocks.append(
                    {
                        "doc": d,
                        "lineage_id": b["lineage_id"],
                        "author": author,
                        "tick": tick,
                        "layer": b["layer"],
                        "right": b["layer"] == want,
                    }
                )
    finally:
        sv.stop()
    (out / "d_c").mkdir(parents=True, exist_ok=True)
    for f in ["events.jsonl", "attestations.jsonl"]:
        src = sv.vault / ".rootmark" / f
        (out / "d_c" / f).write_bytes(src.read_bytes() if src.exists() else b"")
    right = sum(b["right"] for b in blocks)
    tick_mode = {t["id"]: t["mode"] for t in data["ticks"]}
    by_mode: dict[str, dict[str, int]] = {}
    for b in blocks:
        m = by_mode.setdefault(tick_mode[b["tick"]], {"blocks": 0, "right": 0})
        m["blocks"] += 1
        m["right"] += b["right"]
    res = {
        "set": "D-C",
        "ticks": len(data["ticks"]),
        "modes": {
            m: sum(1 for t in data["ticks"] if t["mode"] == m)
            for m in ["apart", "same_doc_after", "collide"]
        },
        "blocks": len(blocks),
        "right": right,
        "accuracy": right / len(blocks) if blocks else None,
        "by_mode": by_mode,
        "wrong": [b for b in blocks if not b["right"]],
        "pass": bool(blocks) and right / len(blocks) >= 0.99,
    }
    return res


# ---- D-E ---------------------------------------------------------------------------


def run_d_e(rootmark: str, work: Path) -> dict:
    rows = []
    for name in sets.D_E_VAULTS:
        src = sets.OLD_VAULTS / name
        want = json.loads((src / "expected.json").read_text())
        v = work / f"d_e-{name}"
        shutil.copytree(src / "vault", v)
        shutil.copytree(src / "store", v / ".rootmark")
        log0 = (v / ".rootmark/events.jsonl").read_bytes()
        got_hash = json.loads(run([rootmark, "events-hash", str(v)]))["events_hash"]
        replay, layers, unverified = {}, {}, 0
        for doc in want["docs"]:
            replay[doc] = json.loads(run([rootmark, "replay", str(v), doc]))
            bl = json.loads(run([rootmark, "blame", str(v), doc]))
            rows_b = bl["blocks"] if isinstance(bl, dict) else bl
            layers[doc] = [[b["lineage_id"], b["layer"]] for b in rows_b]
            unverified += sum(1 for b in rows_b if b["layer"] == "unverified")
        ok = {
            "opens": True,
            "events_hash_unchanged": got_hash == want["events_hash"],
            "log_bytes_unchanged": (v / ".rootmark/events.jsonl").read_bytes() == log0,
            "replay_equal": all(replay[d] == want["replay"][d] for d in want["docs"]),
            "layers_equal": all(layers[d] == want["layers"][d] for d in want["docs"]),
            "no_unverified": unverified == 0,
        }
        rows.append(
            {
                "vault": name,
                "documents": len(want["docs"]),
                "blocks": sum(len(x) for x in layers.values()),
                "canonical": sum(1 for x in layers.values() for _, lay in x if lay == "canonical"),
                **ok,
                "pass": all(ok.values()),
            }
        )
    return {
        "set": "D-E",
        "vaults": rows,
        "passed": sum(r["pass"] for r in rows),
        "pass": all(r["pass"] for r in rows) and len(rows) == 3,
    }


# ---- D-B ---------------------------------------------------------------------------


def run_d_b(
    rootmark: str,
    obsidian: str,
    plugin: Path,
    scenarios: dict,
    window_s: float,
    out: Path,
    work: Path,
) -> dict:
    if not Path(obsidian).is_file():
        return {"skipped": True, "reason": f"no Obsidian binary at {obsidian}", "pass": None}
    sc = work / f"d_b-{window_s}.json"
    sc.write_text(json.dumps(scenarios), encoding="utf-8")
    harness = REPO / "clients/obsidian/e2e/p2_0/.build/sessions.mjs"
    if not harness.is_file():
        raise FileNotFoundError(f"{harness} missing (build the harness: make p2-0-harness)")
    o = work / f"d_b-out-{window_s}"
    run(
        [
            "node",
            str(harness),
            "--rootmark",
            rootmark,
            "--obsidian",
            obsidian,
            "--plugin",
            str(plugin),
            "--vault-src",
            str(sets.MINI),
            "--scenarios",
            str(sc),
            "--window",
            str(window_s),
            "--out",
            str(o),
        ],
        cwd=REPO / "clients/obsidian",
        timeout=4 * 3600,
    )
    return json.loads((o / "sessions.json").read_text())


def choose_window(rows: list[dict]) -> float:
    """The rule (Z5.md, written before the tuning run): the smallest `W`
    whose false-unverified rate is the lowest of the five, provided its
    latency p95 is below 500 ms; if none meets the latency, the smallest `W`
    with the lowest rate (and the report says so)."""
    best = min(r["false_unverified_rate"] for r in rows)
    ok = [
        r for r in rows if r["false_unverified_rate"] == best and (r["latency_p95_ms"] or 1e9) < 500
    ]
    pick = ok or [r for r in rows if r["false_unverified_rate"] == best]
    return min(r["window_s"] for r in pick)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", required=True, choices=["tune", "test"])
    ap.add_argument("--run", required=True)
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--obsidian", default=OBSIDIAN)
    ap.add_argument("--plugin", type=Path, default=REPO / "clients/obsidian")
    ap.add_argument("--only", default="", help="comma list of sets for a partial (development) run")
    a = ap.parse_args(argv)
    out = runinfo.run_dir(a.run) / "p2_0"
    out.mkdir(parents=True, exist_ok=True)
    rootmark = str(Path(a.rootmark).resolve())
    tune, test = sets.d_b()
    with tempfile.TemporaryDirectory(prefix="rootmark-p2-0-") as w:
        work = Path(w)
        if a.phase == "tune":
            if not Path(a.obsidian).is_file():
                rec = {
                    "skipped": True,
                    "reason": f"no Obsidian binary at {a.obsidian}",
                    "window_s": 2.0,
                    "chosen_by": "default (D-B not measured)",
                }
            else:
                rows = []
                for wv in WINDOWS:
                    r = run_d_b(rootmark, a.obsidian, a.plugin, tune, wv, out, work)
                    rows.append(
                        {
                            k: r[k]
                            for k in [
                                "window_s",
                                "saves",
                                "new_blocks",
                                "unverified_after_window",
                                "false_unverified_rate",
                                "raised_late",
                                "latency_p95_ms",
                                "latency_measured",
                                "no_write",
                            ]
                        }
                    )
                rec = {
                    "set": "D-B tune",
                    "sessions": len(tune["sessions"]),
                    "windows": rows,
                    "window_s": choose_window(rows),
                    "chosen_by": "choose_window (Z5.md)",
                }
            rec["tune_set_blake3"] = blake3.blake3(
                json.dumps(tune, sort_keys=True).encode()
            ).hexdigest()
            (out / "tuning.json").write_text(json.dumps(rec, indent=1) + "\n")
            runinfo.record_phase(
                a.run,
                "p2_0_tune",
                {"windows": WINDOWS, "obsidian": a.obsidian},
                ["p2_0/tuning.json"],
            )
            print(json.dumps({k: v for k, v in rec.items() if k != "windows"}))
            return 0
        tuning = out / "tuning.json"
        if not tuning.is_file():
            raise FileNotFoundError(
                f"{tuning} missing: run --phase tune first (W is fixed before the test sets are read)"
            )
        window_s = float(json.loads(tuning.read_text())["window_s"])
        only = set(filter(None, a.only.split(",")))
        todo = only or {"d_e", "d_a", "d_c", "d_b"}
        if "d_e" in todo:
            (out / "d_e.json").write_text(json.dumps(run_d_e(rootmark, work), indent=1) + "\n")
        if "d_a" in todo:
            (out / "d_a.json").write_text(
                json.dumps(run_d_a(rootmark, window_s, out, work / "a"), indent=1) + "\n"
            )
        if "d_c" in todo:
            (out / "d_c.json").write_text(
                json.dumps(run_d_c(rootmark, window_s, out, work / "c"), indent=1) + "\n"
            )
        if "d_b" in todo:
            (out / "d_b.json").write_text(
                json.dumps(
                    run_d_b(rootmark, a.obsidian, a.plugin, test, window_s, out, work), indent=1
                )
                + "\n"
            )
        runinfo.record_phase(
            a.run,
            "p2_0_test" + ("" if not only else "_" + "_".join(sorted(only))),
            {"window_s": window_s, "sets": sorted(todo), "obsidian": a.obsidian},
            [f"p2_0/{t}.json" for t in sorted(todo)],
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
