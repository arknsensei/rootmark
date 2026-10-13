"""Metric definitions (EVAL §Metrikler). Frozen by `T9: freeze eval`.

Every function is pure: it takes already-resolved lineage ids (see
`eval/truth.py`) and returns numbers. An empty denominator is an error, never
a silent zero.
"""

from __future__ import annotations

import math
from collections.abc import Iterable, Sequence

import numpy as np


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


# ----------------------------------------------------------------------- H2

K = 10


def relevance(
    ranked_lineages: Sequence[Iterable[str]], groups: Sequence[Iterable[str]]
) -> list[set[int]]:
    """For each ranked chunk, the set of truth groups it touches.

    A chunk is relevant iff its `lineage_ids` intersect the truth; a single-hop
    question has one group, a multi-hop question two (A and B).
    """
    gs = [set(g) for g in groups]
    if not gs or any(not g for g in gs):
        raise MetricError("empty truth group")
    return [{i for i, g in enumerate(gs) if g & set(lins)} for lins in ranked_lineages]


def ndcg_at_k(rel: Sequence[set[int]], min_cover: int, k: int = K) -> float:
    """Binary gain, log2 discount; IDCG = `min_cover` relevant chunks at the top.

    The caller passes `min_cover = min(k, number of relevant chunks in the
    index)`, so a correct IDCG never yields more than 1; a value above 1 is a
    definition error and raises (no clipping).
    """
    if min_cover <= 0:
        raise MetricError("min_cover must be ≥ 1")
    dcg = sum(1.0 / math.log2(i + 2) for i, r in enumerate(rel[:k]) if r)
    idcg = sum(1.0 / math.log2(i + 2) for i in range(min(min_cover, k)))
    v = dcg / idcg
    if v > 1.0 + 1e-12:
        raise MetricError(f"nDCG {v} > 1: IDCG/min_cover inconsistent with the ranking")
    return v


def recall_at_k(
    rel_lineages: Sequence[Iterable[str]], groups: Sequence[Iterable[str]], k: int
) -> float:
    """v1 definition: mean over truth groups of the fraction of the group's
    lineages carried by the top-k chunks (multi-hop: A fully, B not at all → 0.5)."""
    gs = [set(g) for g in groups]
    if not gs or any(not g for g in gs):
        raise MetricError("empty truth group")
    covered = set().union(*[set(x) for x in rel_lineages[:k]]) if rel_lineages[:k] else set()
    return sum(len(g & covered) / len(g) for g in gs) / len(gs)


def mrr(rel: Sequence[set[int]]) -> float:
    """v1 definition: 1 / rank of the first relevant chunk of the given ranking, else 0."""
    for i, r in enumerate(rel):
        if r:
            return 1.0 / (i + 1)
    return 0.0


def ideal_relevant_count(
    corpus_lineages: Iterable[Iterable[str]], groups: Sequence[Iterable[str]], k: int = K
) -> int:
    """`min_cover` for nDCG: min(k, number of indexed chunks relevant to the truth).

    Differs from v1, where min_cover was the smallest set cover of the truth
    lineages; with that IDCG valid rankings exceed 1 and v1 clipped at 1.0.
    v2 forbids clipping, so the IDCG is the ideal ranking of all relevant
    chunks. Zero relevant chunks in the index is an error (truth not indexed).
    """
    truth = set().union(*[set(g) for g in groups])
    n = sum(1 for lins in corpus_lineages if truth & set(lins))
    if n == 0:
        raise MetricError("no indexed chunk carries any truth lineage")
    return min(k, n)


def full_cover_at_k(rel: Sequence[set[int]], n_groups: int, k: int = K) -> float:
    """Multi-hop: 1 iff the top-k touches every truth group (A and B); single-hop
    is reported as recall@k (see `query_metrics`)."""
    if n_groups <= 0:
        raise MetricError("no truth groups")
    covered = set().union(*rel[:k]) if rel[:k] else set()
    return 1.0 if len(covered) == n_groups else 0.0


def query_metrics(
    ranked_lineages: Sequence[Iterable[str]], groups: Sequence[Iterable[str]], min_cover: int
) -> dict:
    """All H2 per-query metrics for one ranking (top-10 chunks' lineage lists)."""
    rel = relevance(ranked_lineages, groups)
    n = len(groups)
    return {
        "ndcg@10": ndcg_at_k(rel, min_cover),
        "recall@5": recall_at_k(ranked_lineages, groups, 5),
        "recall@10": recall_at_k(ranked_lineages, groups, 10),
        "mrr": mrr(rel[:K]),
        "full_cover@10": full_cover_at_k(rel, n)
        if n > 1
        else recall_at_k(ranked_lineages, groups, 10),
    }


def mean(xs: Sequence[float]) -> float:
    if not xs:
        raise MetricError("mean of nothing")
    return float(sum(xs) / len(xs))


def percentile(xs: Sequence[float], q: float) -> float:
    if not xs:
        raise MetricError("percentile of nothing")
    return float(np.percentile(np.asarray(xs, dtype=np.float64), q))


def latency_summary(ms: Sequence[float], warmup: int = 10) -> dict:
    """p50/p95 in ms after dropping the first `warmup` measurements (the
    runner issues the warm-up queries as extra calls before the measured set)."""
    rest = list(ms[warmup:])
    return {"p50": percentile(rest, 50), "p95": percentile(rest, 95), "n": len(rest)}


def paired_bootstrap(
    a: Sequence[float], b: Sequence[float], n: int = 10_000, seed: int = 20261013
) -> dict:
    """Mean of (a − b) with a paired percentile bootstrap 95 % CI."""
    if len(a) != len(b) or not a:
        raise MetricError("paired bootstrap needs equal, non-empty samples")
    d = np.asarray(a, dtype=np.float64) - np.asarray(b, dtype=np.float64)
    rng = np.random.default_rng(seed)
    idx = rng.integers(0, len(d), size=(n, len(d)))
    means = d[idx].mean(axis=1)
    return {
        "delta": float(d.mean()),
        "ci95": [float(np.percentile(means, 2.5)), float(np.percentile(means, 97.5))],
        "n": len(d),
        "samples": n,
    }


# ----------------------------------------------------------------------- H3


def lineage_loss(before: Iterable[str], after: Iterable[str]) -> dict:
    """Re-chunk lineage loss (H3-2): |A − B| lost, |B − A| gained."""
    a, b = set(before), set(after)
    return {"lost": len(a - b), "gained": len(b - a), "A": len(a), "B": len(b)}


def h3_summary(latencies_ms: Sequence[float], lost: int, gained: int, log_equal: bool) -> dict:
    s = latency_summary(latencies_ms, warmup=0)
    return {
        "p50": s["p50"],
        "p95": s["p95"],
        "n": s["n"],
        "lost": lost,
        "gained": gained,
        "log_equal": log_equal,
    }


# ----------------------------------------------------------------------- H4


def max_relative_difference(a: dict, b: dict, fields: Sequence[str]) -> dict:
    """Largest |a−b| / max(|a|, |b|) over dotted `fields` (0 when both are 0).

    A missing field, NaN or infinity is an error; values of different types,
    or unequal non-numeric values, count as an infinite difference.
    """
    worst = 0.0
    diffs = {}
    for f in fields:
        x, y = _get(a, f), _get(b, f)
        num = (int, float)
        if (
            type(x) is not type(y)
            and not (isinstance(x, num) and isinstance(y, num))
            or isinstance(x, bool) != isinstance(y, bool)
        ):
            r = math.inf
        elif isinstance(x, num) and not isinstance(x, bool):
            if not (math.isfinite(x) and math.isfinite(y)):
                raise MetricError(f"field {f} is not finite: {x!r} vs {y!r}")
            den = max(abs(x), abs(y))
            r = 0.0 if den == 0 else abs(x - y) / den
        else:
            r = 0.0 if x == y else math.inf
        if r:
            diffs[f] = r
        worst = max(worst, r)
    return {"max_relative_difference": worst, "differing_fields": diffs, "n_fields": len(fields)}


def _get(d: dict, dotted: str):
    cur = d
    for part in dotted.split("."):
        if not isinstance(cur, dict) or part not in cur:
            raise MetricError(f"field {dotted} missing")
        cur = cur[part]
    return cur
