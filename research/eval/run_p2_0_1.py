"""P2-0.1 measurements (GOAL P2-0.1 "Önceden kayıtlı eşikler"; TASKS Y5).

    python -m eval.run_p2_0_1 --phase test --run <RUN> --rootmark <bin>
        [--obsidian <bin>] [--plugin <dir>] [--only d_e,d_a,...]

There is no tuning phase: `W` is P2-0's, read from the committed
`docs/rounds/P2-0/progress/Z5_tuning.json` (0.5 s); without that file the
run stops (GOAL: W is fixed, never tuned again). P2-0's D-B test set is
never generated or read here: D-B test-2 is P2-0's generator with the new
seed (`p2_0_1_sets.d_b_test2`, ASSUMPTIONS Y0-4). Writes into
`<RUN>/p2_0_1/`:

- `d_e.json` (H-P2-0.1h): P2-0's three `v0.1.0` vaults (`run_p2_0.run_d_e`,
  unchanged) and the two `ce4cf04` vaults (`research/data/p2_0_1_old_vaults`):
  they open, every document replays to what the old build recorded,
  `events_hash` and the bytes of `attestations.jsonl` are unchanged, every
  block keeps its layer;
- `d_a.json` (H-P2-0.1a, a′): P2-0's D-A (`run_p2_0.run_d_a`, unchanged);
  `d_a/{events,attestations}.jsonl` for the pair;
- `d_a2.json` (H-P2-0.1g): 120 agent writes through six file-system paths
  (`eval.agent_fs_paths`), each checked by P2-0's `check_agent_write`;
  `d_a2/{events,attestations}.jsonl` for the pair;
- `d_c.json` (H-P2-0.1d): P2-0's D-C ticks; the human follows the plugin's
  current protocol (file, then `attest` with the saved bytes as `content`,
  T2); `d_c/{events,attestations}.jsonl` for the pair;
- `d_b2.json` (H-P2-0.1f): 150 editor-emulation sessions, each followed by
  `rootmark attest` (CLI, a separate process), each measured by
  `measure_session` (one function, every session);
- `d_b.json` (H-P2-0.1b, b′, c): D-B test-2 in the real Obsidian with P2-0's
  frozen harness (`run_p2_0.run_d_b`);
- `d_a_prime.json` (H-P2-0.1e), `d_c_prime.json` (H-P2-0.1d′): the
  real-Obsidian harness `clients/obsidian/e2e/p2_0_1/real.ts`.

Without an Obsidian binary the real-Obsidian sets are recorded as skipped.
Fail-fast: a server that does not start, an HTTP error, a step that does not
settle, a missing file — an exception, never a partial record.
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import os
import re
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

import blake3

from eval import agent_fs_paths, runinfo
from eval import run_p2_0 as p0
from eval.agent_fs_writer import apply_step
from eval.datasets import p2_0_1_sets as sets1
from eval.datasets import p2_0_sets as sets0

REPO = Path(__file__).resolve().parents[2]
TUNING = REPO / "docs/rounds/P2-0/progress/Z5_tuning.json"
OLD_VAULTS_1 = REPO / "research/data/p2_0_1_old_vaults"
D_E_VAULTS_1 = ["attested", "bulk"]


def window() -> float:
    """W of P2-0 (Z5_tuning.json); the run stops without it."""
    if not TUNING.is_file():
        raise FileNotFoundError(f"{TUNING} missing: W is P2-0's, never tuned again (GOAL P2-0.1)")
    return float(json.loads(TUNING.read_text())["window_s"])


class Served(p0.Served):
    """P2-0's served vault; the human's attestation carries the saved bytes
    as `content` (the plugin's protocol since P2-0.1, T2)."""

    def attest(self, doc: str, data: bytes, content: bool = True) -> dict:
        if self.session is None:
            s = self.call("attest_session", {})
            self.session = (s["session_id"], bytes.fromhex(s["key"]))
        sid, key = self.session
        h = p0.content_hash(data)
        mac = hmac.new(key, f"rootmark-attest/1\n{doc}\n{h}\n".encode(), hashlib.sha256).hexdigest()
        body = {"doc_path": doc, "content_hash": h, "session_id": sid, "mac": mac}
        if content:
            body["content"] = data.decode("utf-8")
        return self.call("attest", body)


def copy_logs(sv: p0.Served, out: Path, name: str, attested: bool) -> None:
    """The vault's `events.jsonl` and `attestations.jsonl` into `<out>/<name>/`;
    a set whose human attested must have an attestation log (an error
    otherwise, as P2-0's D-C); a set without attestations gets an empty one."""
    (out / name).mkdir(parents=True, exist_ok=True)
    (out / name / "events.jsonl").write_bytes((sv.vault / ".rootmark/events.jsonl").read_bytes())
    att = sv.vault / ".rootmark/attestations.jsonl"
    if attested and not att.is_file():
        raise FileNotFoundError(f"{name}: {att} missing (the human attested)")
    (out / name / "attestations.jsonl").write_bytes(att.read_bytes() if att.is_file() else b"")


# ---- D-E ---------------------------------------------------------------------------


def run_d_e(rootmark: str, work: Path) -> dict:
    old = p0.run_d_e(rootmark, work)
    rows = list(old["vaults"])
    for name in D_E_VAULTS_1:
        src = OLD_VAULTS_1 / name
        want = json.loads((src / "expected.json").read_text())
        v = work / f"d_e1-{name}"
        shutil.copytree(src / "vault", v)
        shutil.copytree(src / "store", v / ".rootmark")
        log0 = (v / ".rootmark/events.jsonl").read_bytes()
        att0 = (v / ".rootmark/attestations.jsonl").read_bytes()
        got_hash = json.loads(p0.run([rootmark, "events-hash", str(v)]))["events_hash"]
        replay, layers = {}, {}
        for doc in want["docs"]:
            replay[doc] = json.loads(p0.run([rootmark, "replay", str(v), doc]))
            bl = json.loads(p0.run([rootmark, "blame", str(v), doc]))
            rows_b = bl["blocks"] if isinstance(bl, dict) else bl
            layers[doc] = [[b["lineage_id"], b["layer"]] for b in rows_b]
        att1 = (v / ".rootmark/attestations.jsonl").read_bytes()
        ok = {
            "opens": True,
            "events_hash_unchanged": got_hash == want["events_hash"],
            "log_bytes_unchanged": (v / ".rootmark/events.jsonl").read_bytes() == log0,
            "attestations_unchanged": att1 == att0
            and blake3.blake3(att1).hexdigest()[:32] == want["attestations_blake3"],
            "replay_equal": all(replay[d] == want["replay"][d] for d in want["docs"]),
            "layers_equal": all(layers[d] == want["layers"][d] for d in want["docs"]),
        }
        rows.append(
            {
                "vault": f"ce4cf04/{name}",
                "documents": len(want["docs"]),
                "blocks": sum(len(x) for x in layers.values()),
                "attestation_lines": want["attestation_lines"],
                **ok,
                "pass": all(ok.values()),
            }
        )
    return {
        "set": "D-E",
        "vaults": rows,
        "passed": sum(r["pass"] for r in rows),
        "pass": all(r["pass"] for r in rows) and len(rows) == 5,
    }


# ---- D-A2 --------------------------------------------------------------------------


def run_d_a2(rootmark: str, window_s: float, out: Path, work: Path) -> dict:
    data = sets1.d_a2()
    sv = Served(rootmark, work, window_s)
    rows = []
    try:
        for sc in data["scenarios"]:
            cp = sv.call("checkpoint", {})
            paths = sorted({s["path"] for s in sc["steps"]})
            before = {p: sv.blame(p) for p in paths}
            for st in sc["steps"]:
                agent_fs_paths.apply_step(sv.vault, st, work / "agent-prep")
            sv.quiesce()
            r = p0.check_agent_write(sv, sc, cp, before)
            r["path"] = sc["steps"][0]["op"]
            rows.append(r)
    finally:
        sv.stop()
    copy_logs(sv, out, "d_a2", attested=False)
    if (out / "d_a2" / "attestations.jsonl").read_bytes() != b"":
        raise RuntimeError("D-A2: the vault holds attestations, but nothing attested")
    leaks = sum(len(r["leak"]) for r in rows)
    return {
        "set": "D-A2",
        "scenarios": len(rows),
        "new_blocks": sum(r["new_blocks"] for r in rows),
        "canonical_leak": leaks,
        "by_path": {
            p: {
                "scenarios": sum(1 for r in rows if r["path"] == p),
                "new_blocks": sum(r["new_blocks"] for r in rows if r["path"] == p),
                "canonical_leak": sum(len(r["leak"]) for r in rows if r["path"] == p),
            }
            for p in agent_fs_paths.PATHS
        },
        "scenarios_0a_prime": sum(r["pass_0a_prime"] for r in rows),
        "pass": leaks == 0 and all(r["pass_0a"] for r in rows) and len(rows) == 120,
        "rows": rows,
    }


# ---- D-C ---------------------------------------------------------------------------


def run_d_c(rootmark: str, window_s: float, out: Path, work: Path) -> dict:
    """P2-0's D-C (`p2_0_sets.d_c`, the same ticks and the same check as
    `run_p2_0.run_d_c`); the human's attestation carries `content`."""
    data = sets0.d_c()
    sv = Served(rootmark, work, window_s)
    try:
        for t in data["ticks"]:
            h, a = t["human"], t["agent"]
            hb = h["text"].encode()
            (sv.vault / h["path"]).write_bytes(hb)
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
            bl = sv.blame(d)
            if bl is None:
                raise RuntimeError(f"D-C: the server does not know {d}")
            for b in bl:
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
    copy_logs(sv, out, "d_c", attested=True)
    right = sum(b["right"] for b in blocks)
    tick_mode = {t["id"]: t["mode"] for t in data["ticks"]}
    by_mode: dict[str, dict[str, int]] = {}
    for b in blocks:
        m = by_mode.setdefault(tick_mode[b["tick"]], {"blocks": 0, "right": 0})
        m["blocks"] += 1
        m["right"] += b["right"]
    partial = [
        json.loads(x)
        for x in (out / "d_c" / "attestations.jsonl").read_text().splitlines()
        if '"partial":true' in x
    ]
    return {
        "set": "D-C",
        "ticks": len(data["ticks"]),
        "modes": {
            m: sum(1 for t in data["ticks"] if t["mode"] == m)
            for m in ["apart", "same_doc_after", "collide"]
        },
        "blocks": len(blocks),
        "right": right,
        "accuracy": right / len(blocks) if blocks else None,
        "agent_canonical": sum(
            1 for b in blocks if b["author"] == "agent" and b["layer"] == "canonical"
        ),
        "accuracy_without_collide": (
            sum(b["right"] for b in blocks if tick_mode[b["tick"]] != "collide")
            / max(1, sum(1 for b in blocks if tick_mode[b["tick"]] != "collide"))
        ),
        "by_mode": by_mode,
        "partial_lines": len(partial),
        "wrong": [b for b in blocks if not b["right"]],
        "pass": bool(blocks) and right / len(blocks) >= 0.99,
    }


# ---- D-B2 --------------------------------------------------------------------------


def save(vault: Path, doc: str, how: str, add: str) -> None:
    """One editor save of `doc` with `add` appended to its text (Y0-6)."""
    p = vault / doc
    old = p.read_text(encoding="utf-8")
    full = old.rstrip("\n") + add
    if how == "atomic":
        tmp = p.parent / f".{p.name}.rmtmp"
        with open(tmp, "wb") as f:
            f.write(full.encode("utf-8"))
            f.flush()
            os.fsync(f.fileno())
        os.rename(tmp, p)
    elif how == "inplace":
        with open(p, "wb") as f:
            f.write(full.encode("utf-8"))
            f.flush()
            os.fsync(f.fileno())
    elif how == "append":
        with open(p, "ab") as f:
            f.write(full[len(old) :].encode("utf-8") if full.startswith(old) else add.encode())
            f.flush()
            os.fsync(f.fileno())
    else:
        raise ValueError(f"unknown save {how!r}")


def cli_attest(rootmark: str, vault: Path, doc: str, renamed_from: str | None = None) -> None:
    cmd = [rootmark, "attest", str(vault), doc]
    if renamed_from is not None:
        cmd += ["--renamed-from", renamed_from]
    r = subprocess.run(cmd, check=False, capture_output=True, text=True, timeout=60)
    if r.returncode != 0:
        raise RuntimeError(f"rootmark attest {doc}: exit {r.returncode}: {r.stderr[-400:]}")


def measure_session(sv: Served, doc: str, before: set[str], t_last: float, window_s: float) -> dict:
    """THE check of one D-B2 session (Y0-6): the new blocks of `doc` (versions
    not in `before`); latency = from the end of the last save to the first
    `blame` answer, asked after it, whose new blocks are all `canonical`;
    then, W + 1.5 s after the last save, the new blocks still `unverified`."""
    latency = None
    deadline = t_last + window_s + 1.5
    while time.monotonic() < deadline:
        bl = sv.blame(doc) or []
        fresh = [b for b in bl if b["version_id"] not in before]
        if fresh and all(b["layer"] == "canonical" for b in fresh):
            latency = (time.monotonic() - t_last) * 1000
            break
        time.sleep(0.025)
    rest = deadline - time.monotonic()
    if rest > 0:
        time.sleep(rest)
    bl = sv.blame(doc) or []
    fresh = [b for b in bl if b["version_id"] not in before]
    return {
        "doc": doc,
        "new_blocks": len(fresh),
        "unverified_after_window": sum(1 for b in fresh if b["layer"] == "unverified"),
        "latency_ms": None if latency is None else round(latency, 1),
    }


def run_d_b2(rootmark: str, window_s: float, out: Path, work: Path) -> dict:
    data = sets1.d_b2()
    sv = Served(rootmark, work, window_s)
    rows = []
    try:
        for s in data["sessions"]:
            doc = s["doc"]
            if s["kind"] == "mv":
                os.rename(sv.vault / doc, sv.vault / s["to"])
                t_last = time.monotonic()
                cli_attest(rootmark, sv.vault, s["to"], renamed_from=doc)
                r = measure_session(sv, s["to"], set(), t_last, window_s)
            else:
                before = {b["version_id"] for b in (sv.blame(doc) or [])}
                t_last = 0.0
                for i, sv_save in enumerate(s["saves"]):
                    if i > 0:
                        time.sleep(sv_save.get("gap_s", 0))
                    save(sv.vault, doc, sv_save["how"], sv_save["text"])
                    t_last = time.monotonic()
                    cli_attest(rootmark, sv.vault, doc)
                r = measure_session(sv, doc, before, t_last, window_s)
            rows.append({"id": s["id"], "kind": s["kind"], **r})
            sv.quiesce()
    finally:
        sv.stop()
    # the server's logs (the cause of any false unverified, ASSUMPTIONS Y1-1)
    copy_logs(sv, out, "d_b2", attested=True)
    blocks = sum(r["new_blocks"] for r in rows)
    unv = sum(r["unverified_after_window"] for r in rows)
    # ASSUMPTIONS Y3-4: a `mv` session counts every block of the new path
    # (as P2-0's D-B counts a renamed note); the rate without them, reported
    nb_mv = sum(r["new_blocks"] for r in rows if r["kind"] != "mv")
    unv_mv = sum(r["unverified_after_window"] for r in rows if r["kind"] != "mv")
    lat = sorted(float("inf") if r["latency_ms"] is None else r["latency_ms"] for r in rows)
    p95 = lat[min(len(lat) - 1, -(-len(lat) * 95 // 100) - 1)] if lat else None
    return {
        "set": "D-B2",
        "sessions": len(rows),
        "new_blocks": blocks,
        "unverified_after_window": unv,
        "false_unverified_rate": None if blocks == 0 else unv / blocks,
        "false_unverified_rate_without_mv": None if nb_mv == 0 else unv_mv / nb_mv,
        "latency_p95_ms": None if p95 is None or p95 == float("inf") else p95,
        "latency_unsettled": sum(1 for r in rows if r["latency_ms"] is None),
        "by_kind": {
            k: {
                "sessions": sum(1 for r in rows if r["kind"] == k),
                "new_blocks": sum(r["new_blocks"] for r in rows if r["kind"] == k),
                "unverified_after_window": sum(
                    r["unverified_after_window"] for r in rows if r["kind"] == k
                ),
            }
            for k in ["atomic", "inplace", "append", "mv", "rapid"]
        },
        "pass": blocks > 0
        and unv / blocks <= 0.01
        and p95 is not None
        and p95 < 500
        and len(rows) == 150,
        "rows": rows,
    }


# ---- real Obsidian -----------------------------------------------------------------


def run_real(
    name: str, rootmark: str, obsidian: str, plugin: Path, window_s: float, out: Path, work: Path
) -> dict:
    if not Path(obsidian).is_file():
        return {"skipped": True, "reason": f"no Obsidian binary at {obsidian}", "pass": None}
    data = {"d_a_prime": sets1.d_a_prime, "d_c_prime": sets1.d_c_prime}[name]()
    work.mkdir(parents=True, exist_ok=True)
    f = work / f"{name}.json"
    f.write_text(json.dumps(data), encoding="utf-8")
    harness = REPO / "clients/obsidian/e2e/p2_0_1/.build/real.mjs"
    if not harness.is_file():
        raise FileNotFoundError(f"{harness} missing (make p2-0-1-harness)")
    o = work / f"{name}-out"
    p0.run(
        [
            "node",
            str(harness),
            "--set",
            name,
            "--data",
            str(f),
            "--rootmark",
            rootmark,
            "--obsidian",
            obsidian,
            "--plugin",
            str(plugin),
            "--vault-src",
            str(sets1.MINI),
            "--window",
            str(window_s),
            "--out",
            str(o),
        ],
        cwd=REPO / "clients/obsidian",
        timeout=4 * 3600,
    )
    # the vault's logs (the cause of any false unverified, ASSUMPTIONS Y1-1)
    (out / name).mkdir(parents=True, exist_ok=True)
    for f in ["events.jsonl", "attestations.jsonl"]:
        src = o / f
        if not src.is_file():
            raise FileNotFoundError(f"{name}: the harness wrote no {f}")
        (out / name / f).write_bytes(src.read_bytes())
    return json.loads((o / f"{name}.json").read_text())


SETS = ["d_e", "d_a", "d_a2", "d_c", "d_b2", "d_b", "d_a_prime", "d_c_prime"]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", required=True, choices=["test"])
    ap.add_argument("--run", required=True)
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--obsidian", default=p0.OBSIDIAN)
    ap.add_argument("--plugin", type=Path, default=REPO / "clients/obsidian")
    ap.add_argument("--only", default="", help="comma list of sets for a partial (development) run")
    a = ap.parse_args(argv)
    window_s = window()
    out = runinfo.run_dir(a.run) / "p2_0_1"
    out.mkdir(parents=True, exist_ok=True)
    rootmark = str(Path(a.rootmark).resolve())
    only = set(filter(None, a.only.split(",")))
    if only - set(SETS):
        raise ValueError(f"unknown sets {sorted(only - set(SETS))}")
    todo = [s for s in SETS if not only or s in only]
    with tempfile.TemporaryDirectory(prefix="rootmark-p2-0-1-") as w:
        work = Path(w)
        for s in todo:
            if s == "d_e":
                r = run_d_e(rootmark, work)
            elif s == "d_a":
                r = p0.run_d_a(rootmark, window_s, out, work / "a")
            elif s == "d_a2":
                r = run_d_a2(rootmark, window_s, out, work / "a2")
            elif s == "d_c":
                r = run_d_c(rootmark, window_s, out, work / "c")
            elif s == "d_b2":
                r = run_d_b2(rootmark, window_s, out, work / "b2")
            elif s == "d_b":
                r = p0.run_d_b(
                    rootmark, a.obsidian, a.plugin, sets1.d_b_test2(), window_s, out, work
                )
            else:
                r = run_real(s, rootmark, a.obsidian, a.plugin, window_s, out, work / s)
            (out / f"{s}.json").write_text(json.dumps(r, indent=1) + "\n")
    runinfo.record_phase(
        a.run,
        "p2_0_1_test" + ("" if not only else "_" + "_".join(sorted(only))),
        {"window_s": window_s, "sets": todo, "obsidian": a.obsidian},
        [f"p2_0_1/{t}.json" for t in todo],
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
