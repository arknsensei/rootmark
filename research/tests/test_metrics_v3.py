"""metrics_v3 (EVAL_DELTA §Metrikler): K4 S2 denominator, dominant equality in
the event F1, H1-G counting, op sets per set, v2 functions reused."""

import pytest

from eval import metrics as M
from eval import metrics_v3 as M3


def ev(kind, frm, to, dom=None):
    meta = {"from_lineages": frm}
    if dom is not None:
        meta["dominant"] = dom
    return {"kind": kind, "meta": meta, "lineage_ids": to}


def base_records():
    """Minimal records so every easy/hard denominator is non-empty."""
    return [
        M3.step_record("rephrase_light", [("L", ["L"])], None, ["L"], []),
        M3.step_record("dup_delete", [("A", ["A"]), ("B", [])], None, ["A", "B"], []),
        M3.step_record("dup_delete_adjacent", [("A", ["A"]), ("B", [])], None, ["A", "B"], []),
        M3.step_record("insert", [(None, ["N"])], None, ["L"], []),
    ]


# ------------------------------------------------------------- S2 (K4)


def test_merge_result_is_not_in_the_s2_denominator():
    # merge whose result keeps an existing lineage (correct K4): S2 must stay 0
    merge = M3.step_record(
        "merge",
        [("P", ["Q"]), ("Q", ["Q"])],
        1,
        ["P", "Q"],
        [ev("BlockMerged", ["P", "Q"], ["Q"], "Q")],
    )
    assert merge["new_linked"] == []
    m = M3.aggregate([*base_records(), merge], "easy")
    assert m["H1-S2"] == 0.0
    assert m["n_false_match"] == 1  # only the insert
    # the v2 definition counts the merge result as linked (information only)
    assert m["H1-S2_v2def"] == 0.5


def test_split_non_dominant_part_linked_to_old_lineage_is_a_false_match():
    # matcher kept S on the non-dominant part (index 0) instead of the dominant (index 1)
    split = M3.step_record(
        "split", [("S", ["S", "N"])], 1, ["S"], [ev("BlockSplit", ["S"], ["S", "N"], "S")]
    )
    assert split["new_linked"] == [True]
    assert split["retained"] == [False]
    m = M3.aggregate([*base_records(), split], "easy")
    assert m["H1-S2"] > 0
    assert m["H1-S2"] == 0.5


def test_correct_k4_split_is_not_a_false_match():
    split = M3.step_record(
        "split_edit", [("S", ["N", "S"])], 1, ["S"], [ev("BlockSplit", ["S"], ["N", "S"], "S")]
    )
    assert split["new_linked"] == [False]
    assert split["retained"] == [True]


# ------------------------------------------------------------- event F1


def test_event_f1_requires_dominant_equality():
    truth_ok = M3.step_record(
        "split", [("S", ["N", "S"])], 1, ["S"], [ev("BlockSplit", ["S"], ["N", "S"], "S")]
    )
    assert M3.event_f1([truth_ok])["tp"] == 1
    # same kind, sources and results, but the event names another dominant
    wrong_dom = M3.step_record(
        "split", [("S", ["N", "S"])], 1, ["S"], [ev("BlockSplit", ["S"], ["N", "S"], "N")]
    )
    assert M3.event_f1([wrong_dom])["tp"] == 0
    # a v2-style event without dominant never matches
    no_dom = M3.step_record(
        "merge", [("P", ["Q"]), ("Q", ["Q"])], 1, ["P"], [ev("BlockMerged", ["P", "Q"], ["Q"])]
    )
    assert M3.event_f1([no_dom])["tp"] == 0


def test_event_f1_split_truth_keeps_old_lineage_in_results():
    # dominant part got a new lineage: results {N1, N2} ≠ truth {S, N1}
    r = M3.step_record(
        "split", [("S", ["N1", "N2"])], 1, ["S"], [ev("BlockSplit", ["S"], ["N1", "N2"], "N2")]
    )
    assert r["truth_events"] == [("BlockSplit", frozenset({"S"}), frozenset({"S", "N1"}), "S")]
    assert M3.event_f1([r]) == M.f1(0, 1, 1)


def test_event_f1_pooled_precision_recall():
    recs = [
        M3.step_record(
            "split", [("S", ["S", "N"])], 0, ["S"], [ev("BlockSplit", ["S"], ["S", "N"], "S")]
        ),
        M3.step_record("merge", [("P", ["P"]), ("Q", ["P"])], 0, ["P", "Q"], []),
        M3.step_record(
            "move", [("X", ["X"])], None, ["X"], [ev("BlockSplit", ["X"], ["X", "Y"], "X")]
        ),
    ]
    d = M3.event_f1(recs)
    assert (d["tp"], d["n_pred"], d["n_truth"]) == (1, 2, 2)
    assert d["f1"] == pytest.approx(0.5)


# ------------------------------------------------------------- retention / sets


def test_retention_counts_only_dominant_and_hard_ops_per_set():
    recs = [
        *base_records(),
        M3.step_record("split_edit", [("S", ["N", "S"])], 1, ["S"], []),
        M3.step_record("merge_edit", [("P", ["Q"]), ("Q", ["Q"])], 0, ["P", "Q"], []),
        M3.step_record("split_three", [("T", ["T", "U", "V"])], 0, ["T"], []),
        M3.step_record("rephrase_extreme", [("R", ["Z"])], None, ["R"], []),
        M3.step_record("move_edit", [("M", ["M"])], None, ["M"], []),
    ]
    hard = M3.aggregate(recs, "hard")
    # rephrase_light ✓, split_edit ✓ (dominant), merge_edit ✗ (kept Q, dominant P),
    # rephrase_extreme ✗, move_edit ✓ ; split_three is not an H1-H1 op
    assert hard["n_retention"] == 5
    assert hard["H1-H1"] == pytest.approx(3 / 5)
    easy = M3.aggregate(recs, "easy")
    assert easy["n_retention"] == 1 and easy["H1-S1"] == 1.0
    # split_three: two non-dominant parts in the S2 denominator, plus split_edit's one, plus insert
    assert hard["n_false_match"] == 4
    assert hard["H1-H4"] == 1.0 and easy["H1-S4"] == 1.0


def test_dup_delete_retention():
    r = M3.step_record(
        "dup_delete_adjacent", [("A", ["A"]), ("C", ["X"]), ("B", [])], None, ["A", "B", "C"], []
    )
    assert r["retained"] == [True, False]


def test_objective_and_distinct_objectives():
    m = {"H1-H1": 0.9, "H1-H2": 0.1, "H1-H3": 0.8, "H1-H4": 1.0}
    assert M3.objective(m) == pytest.approx(3.6)
    assert M3.distinct_objectives([3.6, 3.6000001, 3.7, 3.6000004]) == 2
    assert M3.distinct_objectives([1.0] * 15) == 1
    with pytest.raises(M.MetricError):
        M3.distinct_objectives([])


def test_errors_are_raised_not_zeroed():
    with pytest.raises(M.MetricError):
        M3.step_record("split", [("S", ["A", "B"])], None, ["S"], [])
    with pytest.raises(M.MetricError):
        M3.step_record("move", [("S", ["S"])], 0, ["S"], [])
    with pytest.raises(M.MetricError):
        M3.step_record("split", [("S", ["A", "B"])], 2, ["S"], [])
    with pytest.raises(M.MetricError):
        M3.aggregate([M3.step_record("move", [("S", ["S"])], None, ["S"], [])], "easy")
    with pytest.raises(M.MetricError):
        M3.aggregate([], "medium")


def test_delete_contributes_nothing_as_in_v2():
    r = M3.step_record("delete", [("D", [])], None, ["D"], [ev("BlockSplit", ["D"], ["E", "F"])])
    v2 = M.h1s_step_record("delete", [("D", [])], ["D"], [ev("BlockSplit", ["D"], ["E", "F"])])
    assert r["retained"] == r["new_linked"] == r["truth_events"] == []
    assert v2["retained"] == v2["new_linked"] == v2["truth_events"] == []
    assert len(r["pred_events"]) == len(v2["pred_events"]) == 1


def test_h1r_functions_are_the_v2_functions():
    assert M3.h1r_pair_record is M.h1r_pair_record
    assert M3.h1r_aggregate is M.h1r_aggregate
