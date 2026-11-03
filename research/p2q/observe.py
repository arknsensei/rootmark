#!/usr/bin/env python3
"""rootmark observation summary (P2-Q, GOAL Q2 (e), Q-11; docs/GUIDE.md
"Collecting data for an evaluation").

Reads — and only reads — a vault's governance records and prints a period
summary as JSON: counts and ratios, never content (no text, no file path, no
query; a block is named only by the blake2b of its lineage id).

    python3 observe.py --vault <vault> [--attest-log <file>] [--status <file>]
                       [--since-seq N] [--out <file>]

  --vault       the vault directory (reads .rootmark/events.jsonl,
                .rootmark/traces.jsonl and the trace archive,
                .rootmark/checkpoints/)
  --attest-log  the file given to `rootmark serve --attest-log` (optional)
  --status      a saved `GET /v1/status` answer (optional; e.g. curl -o)
  --since-seq   count only events with seq >= N and traces with
                at_seq >= N (the period); layers are always the state at
                the log's end
  --out         write the JSON there (outside the vault) instead of stdout

Python 3.10+, standard library only; works on a running vault (it reads
complete lines only) and never writes into the vault. What rootmark does not
record (request durations, error counts, memory) is not in the summary.

Layer of a lineage (the kernel's rules as the event log states them): the
last event naming it decides — `Promoted`/`Demoted`/`LayerChanged` set
`meta.to`; a content event sets `meta.layer` if present, else `proposal` for
an agent's block and `canonical` for a human's; `BlockDeleted` and a
`Demoted` with `meta.prune` make it inactive.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from collections import Counter
from pathlib import Path

CONTENT_KINDS = {"BlockCreated", "BlockUpdated", "BlockMoved", "BlockSplit", "BlockMerged"}
LAYER_KINDS = {"Promoted", "Demoted", "LayerChanged"}
VERSION = "p2q-observe/1"


def _b2(s: str) -> str:
    return hashlib.blake2b(s.encode("utf-8"), digest_size=16).hexdigest()


def _lines(p: Path):
    """Complete JSON lines of a file opened read-only (a line still being
    written — no trailing newline — is left out)."""
    if not p.is_file():
        return
    with open(p, "rb") as f:
        data = f.read()
    end = data.rfind(b"\n")
    if end < 0:
        return
    for raw in data[: end + 1].split(b"\n"):
        if raw.strip():
            yield json.loads(raw)


def _actor_kind(ev: dict) -> str:
    meta = ev.get("meta") or {}
    a = meta.get("actor")
    if isinstance(a, dict) and a.get("kind"):
        return str(a["kind"])
    prov = meta.get("provenance") or {}
    if prov.get("source") == "unattested_fs":
        return "unattested_fs"
    return {"Agent": "agent", "Human": "human"}.get(prov.get("author_kind"), "unknown")


def _content_layer(ev: dict) -> str:
    meta = ev.get("meta") or {}
    if meta.get("layer"):
        return str(meta["layer"])
    prov = meta.get("provenance") or {}
    return "proposal" if prov.get("author_kind") == "Agent" else "canonical"


def summarize_events(events: list[dict], since: int) -> dict:
    layer: dict[str, str] = {}
    by_kind: Counter = Counter()
    writes = Counter()
    writers = Counter()
    agent_sources = Counter()
    promo = Counter()
    promo_rules = Counter()
    demo = Counter()
    late = 0
    checkpoints = named = 0
    unverified_at_checkpoint = []
    head = 0
    docs_written = set()
    for ev in events:
        seq = ev["seq"]
        head = seq + 1
        kind = ev["kind"]
        meta = ev.get("meta") or {}
        in_period = seq >= since
        if kind in CONTENT_KINDS or kind == "BlockDeleted":
            lay = "inactive" if kind == "BlockDeleted" else _content_layer(ev)
            for lid in ev.get("lineage_ids", []):
                layer[lid] = lay
        elif kind in LAYER_KINDS:
            to = meta.get("to")
            for lid in ev.get("lineage_ids", []):
                if kind == "Demoted" and meta.get("prune"):
                    layer[lid] = "inactive"
                elif to:
                    layer[lid] = str(to)
        elif kind == "CheckpointCreated":
            unverified_at_checkpoint.append(sum(1 for v in layer.values() if v == "unverified"))
        if not in_period:
            continue
        by_kind[kind] += 1
        if kind in CONTENT_KINDS or kind == "BlockDeleted":
            short = kind.removeprefix("Block").lower()
            writes[short] += len(ev.get("lineage_ids", []))
            who = _actor_kind(ev)
            writers[who] += 1
            if who == "agent":
                src = (meta.get("provenance") or {}).get("source") or ""
                agent_sources[src.split("/")[0]] += 1
            docs_written.add(_b2(ev.get("doc_path", "")))
        elif kind == "Promoted":
            ak = _actor_kind(ev)
            promo[ak] += len(ev.get("lineage_ids", []))
            if ak == "policy":
                rid = str((meta.get("actor") or {}).get("id", ""))
                promo_rules[rid] += len(ev.get("lineage_ids", []))
        elif kind == "Demoted":
            n = len(ev.get("lineage_ids", []))
            demo["pruned" if meta.get("prune") else _actor_kind(ev)] += n
        elif kind == "LayerChanged" and meta.get("cause") == "late_attestation":
            late += len(ev.get("lineage_ids", []))
        elif kind == "CheckpointCreated":
            checkpoints += 1
        elif kind == "CheckpointNamed":
            named += 1
    layers = Counter(layer.values())
    agent_created = writers.get("agent", 0)
    return {
        "head_seq": head,
        "since_seq": since,
        "events": {"total": sum(by_kind.values()), "by_kind": dict(sorted(by_kind.items()))},
        "writes": {
            "lineages": dict(sorted(writes.items())),
            "events_by_writer": dict(sorted(writers.items())),
            "agent_events_by_client": dict(sorted(agent_sources.items())),
            "documents_touched": len(docs_written),
        },
        "promotions": {
            "lineages_by_actor": dict(sorted(promo.items())),
            "lineages_by_rule": dict(sorted(promo_rules.items())),
            "late_attestation_raises": late,
        },
        "demotions": {"lineages_by_actor": dict(sorted(demo.items()))},
        "checkpoints": {"created": checkpoints, "named": named},
        "layers": {
            "lineages": dict(sorted(layers.items())),
            "unverified_at_each_checkpoint": unverified_at_checkpoint,
        },
        "ratios": {
            "promoted_human_per_agent_write_event": (
                round(promo.get("human", 0) / agent_created, 4) if agent_created else None
            ),
            "unverified_share_of_live": (
                round(layers.get("unverified", 0) / live, 4)
                if (live := sum(v for k, v in layers.items() if k != "inactive"))
                else None
            ),
        },
    }


def summarize_traces(rows: list[dict], since: int, top: int = 10) -> dict:
    consumers = Counter()
    returned = 0
    blocks = Counter()
    searches = reads = 0
    for t in rows:
        if t.get("at_seq", 0) < since:
            continue
        consumers[str(t.get("consumer", ""))] += 1
        if t.get("query_hash"):
            searches += 1
        else:
            reads += 1
        for r in t.get("returned", []):
            returned += 1
            blocks[_b2(r.get("lineage_id", ""))] += 1
    return {
        "total": sum(consumers.values()),
        "searches": searches,
        "reads": reads,
        "by_consumer": dict(sorted(consumers.items())),
        "returned_blocks": returned,
        "distinct_blocks": len(blocks),
        "most_relied_on": [
            {"lineage_b2": k, "count": v}
            for k, v in sorted(blocks.items(), key=lambda kv: (-kv[1], kv[0]))[:top]
        ],
    }


def summarize_attest(rows: list[dict]) -> dict:
    attest = Counter()
    refused = Counter()
    observe = Counter()
    unmatched = 0
    for r in rows:
        if r.get("event") == "attest":
            attest[f"{r.get('kind', '?')}:{r.get('result', '?')}"] += 1
            if r.get("result") == "refused":
                refused[f"{r.get('kind', '?')}:{r.get('code', '?')}"] += 1
            if r.get("matched") is False:
                unmatched += 1
        elif r.get("event") == "observe":
            observe[str(r.get("ingest", "?"))] += 1
            for b in r.get("accepted_before") or []:
                observe[f"unverified_because:{b.get('why', '?')}"] += 1
    return {
        "attestations": dict(sorted(attest.items())),
        "refusal_classes": dict(sorted(refused.items())),
        "accepted_unmatched": unmatched,
        "observations": dict(sorted(observe.items())),
    }


def summarize_status(st: dict) -> dict:
    idx = st.get("index") or {}
    mig = st.get("migrations") or {}
    att = st.get("attestation") or {}
    pol = st.get("policy") or {}
    return {
        "mdk_version": st.get("mdk_version"),
        "spec_version": st.get("spec_version"),
        "embedder": st.get("embedder"),
        "index_state": idx.get("state"),
        "migrations_required": len(mig.get("required") or []),
        "migrations_recommended": len(mig.get("recommended") or []),
        "attestation": {k: att.get(k) for k in ("unattested", "pending", "unmatched", "stage")},
        "promotion": pol.get("promotion"),
        "promotion_rules": len((pol.get("promotion_rules") or {}).get("rules") or []),
        "last_checkpoint_named": bool((st.get("last_checkpoint") or {}).get("name")),
    }


def observe(vault: Path, attest_log: Path | None, status: Path | None, since: int) -> dict:
    store = vault / ".rootmark"
    if not (store / "events.jsonl").is_file():
        raise SystemExit(f"observe: {store / 'events.jsonl'} not found (not a rootmark vault)")
    events = list(_lines(store / "events.jsonl"))
    traces = []
    arch = store / "traces" / "archive"
    if arch.is_dir():
        for f in sorted(os.listdir(arch)):
            traces += list(_lines(arch / f))
    traces += list(_lines(store / "traces.jsonl"))
    out = {"version": VERSION, **summarize_events(events, since)}
    out["traces"] = summarize_traces(traces, since)
    if attest_log is not None:
        out["attest"] = summarize_attest(list(_lines(attest_log)))
    if status is not None:
        out["status"] = summarize_status(json.loads(status.read_text(encoding="utf-8")))
    return out


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--vault", required=True, type=Path)
    ap.add_argument("--attest-log", type=Path)
    ap.add_argument("--status", type=Path)
    ap.add_argument("--since-seq", type=int, default=0)
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    vault = a.vault.resolve()
    if a.out is not None:
        o = a.out.resolve()
        if o == vault or vault in o.parents:
            raise SystemExit("observe: --out must lie outside the vault")
    text = json.dumps(observe(vault, a.attest_log, a.status, a.since_seq), indent=1, sort_keys=True)
    if a.out is not None:
        a.out.write_text(text + "\n", encoding="utf-8")
    else:
        print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
