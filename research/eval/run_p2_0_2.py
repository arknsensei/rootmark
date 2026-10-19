"""P2-0.2 measurements (GOAL P2-0.2 "Önceden kayıtlı eşikler"; TASKS X2, X4).

    python -m eval.run_p2_0_2 --phase test --run <RUN> --rootmark <bin>
        [--obsidian <bin>] [--plugin <dir>] [--only d_c,...]

There is no tuning phase: `W` is P2-0's (`run_p2_0_1.window`, the committed
`docs/rounds/P2-0/progress/Z5_tuning.json`; without it the run stops).
Neither P2-0's nor P2-0.1's D-B test set is generated or read here
(ASSUMPTIONS P2-0.2 X0-3). Writes into `<RUN>/p2_0_2/`:

- `d_c.json` (H-P2-0.2d, i): P2-0's D-C ticks (`p2_0_sets.d_c`, unchanged),
  the human following the plugin's protocol (file, then `attest` with the
  saved bytes as `content`), with T3 (X0-9): in `apart` and
  `same_doc_after` ticks the agent writes only once the human's ingest is
  in the log (`wait_ingested`), so the log's order no longer depends on
  which of two files the watcher sees first; in `collide` ticks (ASSUMPTIONS
  X2-1) the human's attestation (with `content`) is sent before the human's
  write, so the two writes stay 2 ms apart with no HTTP round trip between
  them (the watcher sees them in one observation whatever the load);
  `d_c/{events,attestations}.jsonl` for the pair (compared byte for byte).

- `d_e.json` (H-P2-0.2h): P2-0.1's five old vaults (`run_p2_0_1.run_d_e`,
  unchanged: three `v0.1.0`, two `ce4cf04`) and the `21b16ea` vault
  (`research/data/p2_0_2_old_vaults/partial`, spec 2.0.1, with `partial`
  and `matched` lines; X0-11): they open, every document replays to what
  the old build recorded, `events_hash` and the bytes of `attestations.jsonl`
  are unchanged, every block keeps its layer (6/6);
- `d_a.json` (H-P2-0.2a, a′): P2-0's D-A (`run_p2_0.run_d_a`, unchanged);
- `d_a2.json` (H-P2-0.2a): P2-0.1's D-A2 (`run_p2_0_1.run_d_a2`, unchanged);
- `d_b2.json` (H-P2-0.2f): P2-0.1's D-B2 (`run_p2_0_1.run_d_b2`, unchanged;
  `--renamed-from` under the carry rule);
- `d_b.json` (H-P2-0.2b, b′, c): D-B test-3 (X0-4..X0-7, X0-15..X0-18) in
  the real Obsidian with the P2-0.2 copy of P2-0's session harness
  (`clients/obsidian/e2e/p2_0_2/sessions.ts`) on the test-3 vault; the
  harness's own totals, plus the rename sessions only (b′) and the
  per-format report of their new blocks (`p2_0_2_sets.classify`);
- `d_a_prime.json` (H-P2-0.2e), `d_c_prime.json` (H-P2-0.2d′): the P2-0.2
  copy of P2-0.1's real-Obsidian harness (`e2e/p2_0_2/real.ts`).

Without an Obsidian binary the real-Obsidian sets are recorded as skipped.
Fail-fast: a server that does not start, an HTTP error, a step that does not
settle, a missing file — an exception, never a partial record.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import tempfile
import time
from pathlib import Path

import blake3
import rootmark_core

from eval import run_p2_0 as p0
from eval import run_p2_0_1 as p1
from eval import runinfo
from eval.agent_fs_writer import apply_step
from eval.datasets import p2_0_1_sets as sets1
from eval.datasets import p2_0_2_sets as sets2
from eval.datasets import p2_0_sets as sets0
from eval.run_p2_0_1 import Served, copy_logs, window

REPO = Path(__file__).resolve().parents[2]
OLD_VAULTS_2 = REPO / "research/data/p2_0_2_old_vaults"
D_E_VAULTS_2 = ["partial"]


# ---- D-E ---------------------------------------------------------------------------


def run_d_e(rootmark: str, work: Path) -> dict:
    """P2-0.1's five old vaults, then the `21b16ea` vault, each checked as
    P2-0.1 checks its `ce4cf04` vaults (`run_p2_0_1.run_d_e`)."""
    old = p1.run_d_e(rootmark, work)
    rows = list(old["vaults"])
    for name in D_E_VAULTS_2:
        src = OLD_VAULTS_2 / name
        want = json.loads((src / "expected.json").read_text())
        v = work / f"d_e2-{name}"
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
        lines = [json.loads(x) for x in att1.decode().splitlines()]
        ok = {
            "opens": True,
            "events_hash_unchanged": got_hash == want["events_hash"],
            "log_bytes_unchanged": (v / ".rootmark/events.jsonl").read_bytes() == log0,
            "attestations_unchanged": att1 == att0
            and blake3.blake3(att1).hexdigest()[:32] == want["attestations_blake3"],
            "replay_equal": all(replay[d] == want["replay"][d] for d in want["docs"]),
            "layers_equal": all(layers[d] == want["layers"][d] for d in want["docs"]),
            # the 2.0.1 log read by the current reader holds the lines a
            # 2.0.0 reader refused (X0-11)
            "has_partial_and_matched": any(x.get("partial") is True for x in lines)
            and any(x.get("matched") is False for x in lines),
        }
        rows.append(
            {
                "vault": f"21b16ea/{name}",
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
        "pass": all(r["pass"] for r in rows) and len(rows) == 6,
    }


def block_versions(text: str, doc: str) -> list[str]:
    """The block versions of `text` as `doc` (spec §3), in order."""
    return [b["version_id"] for b in rootmark_core.parse_document(text, doc)[0]]


def wait_ingested(sv: Served, doc: str, data: bytes, limit_s: float = 15.0) -> None:
    """T3 (X0-9): waits until the server's `blame` of `doc` shows the blocks
    of `data` — the human's ingest is in the log; an error after `limit_s`."""
    want = sorted(block_versions(data.decode("utf-8"), doc))
    deadline = time.monotonic() + limit_s
    while True:
        bl = sv.blame(doc)
        if bl is not None and sorted(b["version_id"] for b in bl) == want:
            return
        if time.monotonic() > deadline:
            raise RuntimeError(f"D-C: the human's write of {doc} not ingested within {limit_s} s")
        time.sleep(0.01)


def run_d_c(rootmark: str, window_s: float, out: Path, work: Path) -> dict:
    """P2-0's D-C with T3; the same check as `run_p2_0_1.run_d_c`."""
    data = sets0.d_c()
    sv = Served(rootmark, work, window_s)
    try:
        for t in data["ticks"]:
            h, a = t["human"], t["agent"]
            hb = h["text"].encode()
            if t["mode"] == "collide":
                # X2-1: the attestation first, then the two writes 2 ms
                # apart with nothing slow between them (one observation)
                sv.attest(h["path"], hb)
                (sv.vault / h["path"]).write_bytes(hb)
            else:
                (sv.vault / h["path"]).write_bytes(hb)
                sv.attest(h["path"], hb)
                wait_ingested(sv, h["path"], hb)
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
        "t3": "apart and same_doc_after: the agent writes once the human's ingest is in the log; collide: the attestation before the two writes (X2-1)",
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
        "records": blocks,
        "pass": bool(blocks) and right / len(blocks) >= 0.99,
    }


# ---- real Obsidian -----------------------------------------------------------------


def write_vault(texts: dict[str, str], to: Path) -> None:
    """The test-3 vault's notes (with their folders) under `to`."""
    for p, t in texts.items():
        f = to / p
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text(t, encoding="utf-8")


def format_report(records: list[dict]) -> dict:
    """H-P2-0.2b′ (X0-7, X0-18): the rename sessions' new blocks, by the
    format classes of the links in them naming the renamed note, and by
    session kind."""
    by_class: dict[str, dict[str, int]] = {}
    by_kind: dict[str, dict[str, int]] = {}
    for r in records:
        ren = r.get("rename")
        if ren is None:
            raise ValueError(f"{r['session']}: a rename session's record without its rename")
        k = by_kind.setdefault(r["kind"], {"records": 0, "blocks": 0, "unverified": 0})
        k["records"] += 1
        k["blocks"] += r["new_blocks"]
        k["unverified"] += r["unverified_after_window"]
        for b in r.get("blocks", []):
            for c in sets2.classify(b["text"], ren["to"], r["doc"] == ren["to"]):
                x = by_class.setdefault(c, {"blocks": 0, "unverified": 0})
                x["blocks"] += 1
                x["unverified"] += b["layer"] == "unverified"
    return {"by_class": dict(sorted(by_class.items())), "by_kind": dict(sorted(by_kind.items()))}


def run_d_b(rootmark: str, obsidian: str, plugin: Path, window_s: float, work: Path) -> dict:
    """D-B test-3 (X0-6) with the P2-0.2 session harness on the test-3
    vault; the harness's totals (H-P2-0.2b, c), the rename sessions only
    and their per-format report (H-P2-0.2b′, information)."""
    if not Path(obsidian).is_file():
        return {"skipped": True, "reason": f"no Obsidian binary at {obsidian}", "pass": None}
    harness = REPO / "clients/obsidian/e2e/p2_0_2/.build/sessions.mjs"
    if not harness.is_file():
        raise FileNotFoundError(f"{harness} missing (make p2-0-2-harness)")
    cov = sets2.coverage()
    if not cov["ok"]:
        raise RuntimeError(f"X0-16: a pre-registered count is 0: {cov['zero']}")
    work.mkdir(parents=True, exist_ok=True)
    src = work / "vault3"
    write_vault(sets2.vault3()["texts"], src)
    sc = work / "d_b3.json"
    data = sets2.d_b3()
    sc.write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")
    o = work / "d_b-out"
    p0.run(
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
            str(src),
            "--scenarios",
            str(sc),
            "--window",
            str(window_s),
            "--out",
            str(o),
        ],
        cwd=REPO / "clients/obsidian",
        timeout=6 * 3600,
    )
    res = json.loads((o / "sessions.json").read_text())
    ren = [r for r in res["records"] if r["kind"] in sets2.RENAMES]
    blocks = sum(r["new_blocks"] for r in ren)
    unv = sum(r["unverified_after_window"] for r in ren)
    res["set"] = "D-B test-3"
    res["seed"] = data["seed"]
    res["coverage"] = cov
    res["renames"] = {
        "sessions": sum(1 for s in data["sessions"] if s["kind"] in sets2.RENAMES),
        "records": len(ren),
        "new_blocks": blocks,
        "unverified_after_window": unv,
        "false_unverified_rate": unv / blocks if blocks else None,
        **format_report(ren),
    }
    rate, p95 = res["false_unverified_rate"], res["latency_p95_ms"]
    res["pass"] = (
        rate is not None
        and rate <= 0.01
        and p95 is not None
        and p95 < 500
        and res["sessions"] == 200
    )
    return res


def run_real(
    name: str, rootmark: str, obsidian: str, plugin: Path, window_s: float, out: Path, work: Path
) -> dict:
    """D-A′ (X0-8, the test-3 vault inside the data) and D-C′ (P2-0.1's
    ticks, the mini vault) with the P2-0.2 copy of P2-0.1's harness."""
    if not Path(obsidian).is_file():
        return {"skipped": True, "reason": f"no Obsidian binary at {obsidian}", "pass": None}
    data = {"d_a_prime": sets2.d_a_prime, "d_c_prime": sets1.d_c_prime}[name]()
    work.mkdir(parents=True, exist_ok=True)
    f = work / f"{name}.json"
    f.write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")
    harness = REPO / "clients/obsidian/e2e/p2_0_2/.build/real.mjs"
    if not harness.is_file():
        raise FileNotFoundError(f"{harness} missing (make p2-0-2-harness)")
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
    (out / name).mkdir(parents=True, exist_ok=True)
    for g in ["events.jsonl", "attestations.jsonl"]:
        s = o / g
        if not s.is_file():
            raise FileNotFoundError(f"{name}: the harness wrote no {g}")
        (out / name / g).write_bytes(s.read_bytes())
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
    out = runinfo.run_dir(a.run) / "p2_0_2"
    out.mkdir(parents=True, exist_ok=True)
    rootmark = str(Path(a.rootmark).resolve())
    only = set(filter(None, a.only.split(",")))
    if only - set(SETS):
        raise ValueError(f"unknown sets {sorted(only - set(SETS))}")
    todo = [s for s in SETS if not only or s in only]
    with tempfile.TemporaryDirectory(prefix="rootmark-p2-0-2-") as w:
        work = Path(w)
        for s in todo:
            if s == "d_e":
                r = run_d_e(rootmark, work)
            elif s == "d_a":
                r = p0.run_d_a(rootmark, window_s, out, work / "a")
            elif s == "d_a2":
                r = p1.run_d_a2(rootmark, window_s, out, work / "a2")
            elif s == "d_c":
                r = run_d_c(rootmark, window_s, out, work / "c")
            elif s == "d_b2":
                r = p1.run_d_b2(rootmark, window_s, out, work / "b2")
            elif s == "d_b":
                r = run_d_b(rootmark, a.obsidian, a.plugin, window_s, work / "b")
            else:
                r = run_real(s, rootmark, a.obsidian, a.plugin, window_s, out, work / s)
            (out / f"{s}.json").write_text(json.dumps(r, indent=1, ensure_ascii=False) + "\n")
    runinfo.record_phase(
        a.run,
        "p2_0_2_test" + ("" if not only else "_" + "_".join(sorted(only))),
        {"window_s": window_s, "sets": todo, "obsidian": a.obsidian},
        [f"p2_0_2/{t}.json" for t in todo],
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
