"""Metric definitions (EVAL §Metrikler). Frozen by `T9: freeze eval`.

Every function is pure: it takes already-resolved lineage ids (see
`eval/truth.py`) and returns numbers. An empty denominator is an error, never
a silent zero.
"""

from __future__ import annotations

from collections.abc import Iterable, Sequence


class MetricError(RuntimeError):
    pass


def _ratio(num: int, den: int, what: str) -> float:
    if den <= 0:
        raise MetricError(f"{what}: empty denominator")
    return num / den


# ----------------------------------------------------------------------- H1

RETENTION_OPS_S1 = ("rephrase_light", "rephrase_heavy", "move")
RETENTION_OPS_S4 = ("dup_delete",)
NEW_BLOCK_OPS = ("insert", "split", "merge")
EVENT_OPS = ("split", "merge")


def h1s_step_record(
    op: str, mapping: Sequence, prev_lineages: Iterable[str], events: Sequence[dict]
) -> dict:
    """One scored D1 step.

    `mapping` is the truth mapping with references already resolved to
    lineage ids: `[[old_lineage | None, [new_lineage, ...]], ...]` (old ids
    resolved before ingest, new ids after). `prev_lineages` are the lineages
    of the snapshot before the step; `events` are the step's ingest events.
    """
    prev = set(prev_lineages)
    rec: dict = {"op": op, "retained": [], "new_linked": [], "truth_events": [], "pred_events": []}
    if op in RETENTION_OPS_S1 or op in RETENTION_OPS_S4:
        for old, news in mapping:
            for new in news:  # dup_delete: the deleted copy has no new block
                rec["retained"].append(old == new)
    if op == "insert":
        for _old, news in mapping:
            rec["new_linked"].extend(n in prev for n in news)
    if op == "split":
        ((old, news),) = mapping
        rec["new_linked"].extend(n in prev for n in news)
        rec["truth_events"].append(("BlockSplit", frozenset([old]), frozenset(news)))
    if op == "merge":
        olds = [o for o, _ in mapping]
        new = {n for _, ns in mapping for n in ns}
        if len(new) != 1:
            raise MetricError(f"merge truth must map to one block, got {new}")
        rec["new_linked"].extend(n in prev for n in new)
        rec["truth_events"].append(("BlockMerged", frozenset(olds), frozenset(new)))
    for e in events:
        if e["kind"] in ("BlockSplit", "BlockMerged"):
            rec["pred_events"].append(
                (e["kind"], frozenset(e["meta"]["from_lineages"]), frozenset(e["lineage_ids"]))
            )
    return rec


def f1(tp: int, n_pred: int, n_truth: int) -> dict:
    if n_truth <= 0:
        raise MetricError("F1: no truth events")
    p = tp / n_pred if n_pred else 0.0
    r = tp / n_truth
    f = 2 * p * r / (p + r) if p + r else 0.0
    return {"precision": p, "recall": r, "f1": f, "tp": tp, "n_pred": n_pred, "n_truth": n_truth}


def h1s_aggregate(records: Sequence[dict]) -> dict:
    """H1-S1..S4 plus per-op breakdown (EVAL §H1)."""
    s1 = [x for r in records if r["op"] in RETENTION_OPS_S1 for x in r["retained"]]
    s4 = [x for r in records if r["op"] in RETENTION_OPS_S4 for x in r["retained"]]
    linked = [x for r in records for x in r["new_linked"]]
    tp = n_pred = n_truth = 0
    for r in records:
        truth = list(r["truth_events"])
        n_truth += len(truth)
        n_pred += len(r["pred_events"])
        for p in r["pred_events"]:
            if p in truth:
                truth.remove(p)
                tp += 1
    per_op: dict = {}
    for op in sorted({r["op"] for r in records}):
        rs = [r for r in records if r["op"] == op]
        row: dict = {"steps": len(rs)}
        ret = [x for r in rs for x in r["retained"]]
        if ret:
            row["retention"] = sum(ret) / len(ret)
        lk = [x for r in rs for x in r["new_linked"]]
        if lk:
            row["false_match"] = sum(lk) / len(lk)
        if op in EVENT_OPS:
            t = sum(len(r["truth_events"]) for r in rs)
            ok = sum(1 for r in rs for p in r["pred_events"] if p in r["truth_events"])
            row["event_recall"] = ok / t if t else None
        per_op[op] = row
    return {
        "H1-S1": _ratio(sum(s1), len(s1), "H1-S1"),
        "H1-S2": _ratio(sum(linked), len(linked), "H1-S2"),
        "H1-S3": f1(tp, n_pred, n_truth)["f1"],
        "H1-S3_detail": f1(tp, n_pred, n_truth),
        "H1-S4": _ratio(sum(s4), len(s4), "H1-S4"),
        "n_s1": len(s1),
        "n_s2": len(linked),
        "n_s4": len(s4),
        "per_op": per_op,
    }


def h1s_objective(m: dict) -> float:
    """SPEC §5.3 tuning rule: S1 + (1 − S2) + S3 + S4."""
    return m["H1-S1"] + (1 - m["H1-S2"]) + m["H1-S3"] + m["H1-S4"]


# ------------------------------------------------------------------- H1-R

LOSS_CATEGORIES = ("dup_content", "hunk_adjacent", "type_change", "other")


def touched_lines(hunks: Sequence[dict]) -> set[int]:
    """Old-side lines that count as touched (EVAL §D2).

    A hunk `-a,b` covers old lines a..a+b-1; the lines directly bordering it
    (a-1 and a+b) are touched too. A pure insertion `-a,0` sits between old
    lines a and a+1, so both of those are touched.
    """
    out: set[int] = set()
    for h in hunks:
        a, b = h["old"]
        if b > 0:
            out.update(range(a - 1, a + b + 1))  # a-1 .. a+b (adjacent lines included)
        else:
            out.update((a, a + 1))
    return out


def is_touched(line_start: int, line_end: int, touched: set[int]) -> bool:
    return any(ln in touched for ln in range(line_start, line_end + 1))


def h1r_pair_record(
    old_blocks: Sequence[dict],
    hunks: Sequence[dict],
    new_lineages: Iterable[str],
    new_blocks: Sequence[dict],
) -> dict:
    """One (commit, file) pair.

    `old_blocks`: old-snapshot blocks in order with `lineage_id`, `version_id`,
    `block_type`, `normalized_text`, `line_start`, `line_end`.
    `new_blocks`: new-snapshot blocks with `version_id`, `block_type`,
    `normalized_text`. `new_lineages`: lineages of the new snapshot.
    """
    live = set(new_lineages)
    t = touched_lines(hunks)
    flags = [is_touched(b["line_start"], b["line_end"], t) for b in old_blocks]
    old_vids = [b["version_id"] for b in old_blocks]
    new_vids = [b["version_id"] for b in new_blocks]
    new_texts = {(b["normalized_text"]) for b in new_blocks}
    new_typed = {(b["normalized_text"], b["block_type"]) for b in new_blocks}
    rec = {"untouched": 0, "untouched_kept": 0, "touched": 0, "touched_kept": 0, "losses": []}
    for i, (b, touched) in enumerate(zip(old_blocks, flags)):
        kept = b["lineage_id"] in live
        if touched:
            rec["touched"] += 1
            rec["touched_kept"] += kept
            continue
        rec["untouched"] += 1
        rec["untouched_kept"] += kept
        if kept:
            continue
        if old_vids.count(b["version_id"]) > 1 or new_vids.count(b["version_id"]) > 1:
            cat = "dup_content"
        elif (
            b["normalized_text"] in new_texts
            and (b["normalized_text"], b["block_type"]) not in new_typed
        ):
            cat = "type_change"
        elif (i > 0 and flags[i - 1]) or (i + 1 < len(flags) and flags[i + 1]):
            cat = "hunk_adjacent"
        else:
            cat = "other"
        rec["losses"].append({"category": cat, "version_id": b["version_id"]})
    return rec


def h1r_aggregate(records: Sequence[dict], pairs: Sequence[dict]) -> dict:
    """R1 (untouched retention), R2 (all old blocks), touched_retention, loss categories."""
    u = sum(r["untouched"] for r in records)
    uk = sum(r["untouched_kept"] for r in records)
    t = sum(r["touched"] for r in records)
    tk = sum(r["touched_kept"] for r in records)
    cats = {c: 0 for c in LOSS_CATEGORIES}
    other = []
    for r, p in zip(records, pairs, strict=True):
        for loss in r["losses"]:
            cats[loss["category"]] += 1
            if loss["category"] == "other":
                other.append(
                    {"commit": p["commit"], "doc": p["path"], "version_id": loss["version_id"]}
                )
    return {
        "H1-R1": _ratio(uk, u, "H1-R1"),
        "H1-R2": _ratio(uk + tk, u + t, "H1-R2"),
        "touched_retention": _ratio(tk, t, "touched_retention"),
        "blocks": u + t,
        "untouched": u,
        "touched": t,
        "untouched_lost": u - uk,
        "loss_categories": cats,
        "other_examples": other[:5] if len(other) >= 5 else other,
        "other_count": len(other),
    }
