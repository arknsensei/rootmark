"""v3 metric definitions (EVAL_DELTA §Metrikler). Frozen by `T23: freeze eval v3`.

K4 semantics (ARCHITECTURE §10): in a split the dominant (longest) part keeps
the old lineage and the other parts get new ones; a merge result keeps the
lineage of the longest old block. v2 functions are imported from
`eval.metrics` (never copied): the v2-defined S2 (information only) and the
H1-R functions. Every function is pure; an empty denominator is an error.
"""

from __future__ import annotations

from collections.abc import Iterable, Sequence

from eval import metrics as M
from eval.metrics import MetricError, f1, h1r_aggregate, h1r_pair_record  # noqa: F401 — re-exported

SPLIT_OPS = ("split", "split_edit", "split_three")
MERGE_OPS = ("merge", "merge_edit")
INSERT_OPS = ("insert",)
DUP_OPS = ("dup_delete", "dup_delete_adjacent")
# scored, but no H1 quantity is defined on them (as in v2 `metrics.py`)
NO_METRIC_OPS = ("delete",)
SAME_LINEAGE_OPS = (
    "rephrase_light",
    "rephrase_heavy",
    "rephrase_extreme",
    "move",
    "move_edit",
)
# H1-S1 (D1-easy) and H1-H1 (D1-hard) op sets, as listed in docs/v3/00_GOAL.md
RETENTION_OPS = {
    "easy": ("rephrase_light", "rephrase_heavy", "move"),
    "hard": (
        "rephrase_light",
        "rephrase_heavy",
        "rephrase_extreme",
        "move_edit",
        "split_edit",
        "merge_edit",
    ),
}
CODES = {
    "easy": ("H1-S1", "H1-S2", "H1-S3", "H1-S4"),
    "hard": ("H1-H1", "H1-H2", "H1-H3", "H1-H4"),
}
# v2 op names for the v2-defined S2 (information only)
V2_NEW_BLOCK_OP = {
    "insert": "insert",
    "split": "split",
    "split_edit": "split",
    "split_three": "split",
    "merge": "merge",
    "merge_edit": "merge",
}


def step_record(
    op: str,
    mapping: Sequence,
    dominant: int | None,
    prev_lineages: Iterable[str],
    events: Sequence[dict],
) -> dict:
    """One scored step.

    `mapping`: truth with references resolved to lineages,
    `[[old_lineage | None, [new_lineage, ...]], ...]` (old resolved before the
    ingest, new after). `dominant`: for a split, the index of the dominant part
    within the new list; for a merge, the index of the dominant old block
    within `mapping`; `None` otherwise. `prev_lineages`: lineages of the
    snapshot before the step; `events`: the step's ingest events.
    """
    prev = set(prev_lineages)
    rec: dict = {
        "op": op,
        "retained": [],
        "new_linked": [],
        "new_linked_v2def": [],
        "truth_events": [],
        "pred_events": [],
    }
    is_split, is_merge = op in SPLIT_OPS, op in MERGE_OPS
    if (is_split or is_merge) != (dominant is not None):
        raise MetricError(f"{op}: dominant {'missing' if dominant is None else 'unexpected'}")
    if op in SAME_LINEAGE_OPS or op in DUP_OPS:
        for old, news in mapping:
            for new in news:  # dup_delete*: the deleted copy has no new block
                rec["retained"].append(old == new)
    elif op in INSERT_OPS:
        for _old, news in mapping:
            rec["new_linked"].extend(n in prev for n in news)
    elif is_split:
        ((old, news),) = mapping
        if not 0 <= dominant < len(news) or len(news) < 2:
            raise MetricError(f"{op}: bad dominant {dominant} for {len(news)} parts")
        rec["retained"].append(news[dominant] == old)
        others = [n for k, n in enumerate(news) if k != dominant]
        rec["new_linked"].extend(n in prev for n in others)
        rec["truth_events"].append(("BlockSplit", frozenset([old]), frozenset([old, *others]), old))
    elif is_merge:
        olds = [o for o, _ in mapping]
        new = {n for _, ns in mapping for n in ns}
        if len(new) != 1 or not 0 <= dominant < len(olds):
            raise MetricError(f"{op}: merge truth must map to one block, got {new}")
        (n,) = new
        dom = olds[dominant]
        rec["retained"].append(n == dom)
        rec["truth_events"].append(("BlockMerged", frozenset(olds), frozenset([dom]), dom))
    elif op not in NO_METRIC_OPS:
        raise MetricError(f"unknown op {op!r}")
    if op in V2_NEW_BLOCK_OP:
        v2 = M.h1s_step_record(V2_NEW_BLOCK_OP[op], mapping, prev, events)
        rec["new_linked_v2def"] = v2["new_linked"]
    for e in events:
        if e["kind"] in ("BlockSplit", "BlockMerged"):
            rec["pred_events"].append(
                (
                    e["kind"],
                    frozenset(e["meta"]["from_lineages"]),
                    frozenset(e["lineage_ids"]),
                    e["meta"].get("dominant"),
                )
            )
    return rec


def event_f1(records: Sequence[dict]) -> dict:
    """Pooled P/R/F1: a predicted split/merge is correct iff (kind, source
    lineages, result lineages, dominant) equals a truth event."""
    tp = n_pred = n_truth = 0
    for r in records:
        truth = list(r["truth_events"])
        n_truth += len(truth)
        n_pred += len(r["pred_events"])
        for p in r["pred_events"]:
            if p in truth:
                truth.remove(p)
                tp += 1
    return f1(tp, n_pred, n_truth)


def aggregate(records: Sequence[dict], which: str) -> dict:
    """`which` = "easy" (H1-S1..S4) or "hard" (H1-H1..H4)."""
    if which not in CODES:
        raise MetricError(f"unknown set {which!r}")
    c1, c2, c3, c4 = CODES[which]
    ret_ops = RETENTION_OPS[which]
    s1 = [x for r in records if r["op"] in ret_ops for x in r["retained"]]
    s4 = [x for r in records if r["op"] in DUP_OPS for x in r["retained"]]
    linked = [x for r in records for x in r["new_linked"]]
    linked_v2 = [x for r in records for x in r["new_linked_v2def"]]
    ev = event_f1(records)
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
        if op in SPLIT_OPS + MERGE_OPS:
            t = sum(len(r["truth_events"]) for r in rs)
            ok = 0
            for r in rs:
                truth = list(r["truth_events"])
                for p in r["pred_events"]:
                    if p in truth:
                        truth.remove(p)
                        ok += 1
            row["event_recall"] = ok / t
        per_op[op] = row
    return {
        c1: M._ratio(sum(s1), len(s1), c1),
        c2: M._ratio(sum(linked), len(linked), c2),
        c3: ev["f1"],
        f"{c3}_detail": ev,
        c4: M._ratio(sum(s4), len(s4), c4),
        f"{c2}_v2def": M._ratio(sum(linked_v2), len(linked_v2), f"{c2}_v2def"),
        "n_retention": len(s1),
        "n_false_match": len(linked),
        "n_false_match_v2def": len(linked_v2),
        "n_dup": len(s4),
        "per_op": per_op,
    }


def objective(m: dict, which: str = "hard") -> float:
    """SPEC_DELTA §5.3 tuning rule: S1 + (1 − S2) + S3 + S4 (H1-H* on D1-hard)."""
    c1, c2, c3, c4 = CODES[which]
    return m[c1] + (1 - m[c2]) + m[c3] + m[c4]


def distinct_objectives(values: Iterable[float]) -> int:
    """H1-G: number of distinct objective values over the grid (6 decimals)."""
    vals = [round(v, 6) for v in values]
    if not vals:
        raise MetricError("H1-G: empty grid")
    return len(set(vals))
