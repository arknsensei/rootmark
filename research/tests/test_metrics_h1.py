"""Hand-computed H1 metric examples (EVAL §H1)."""

import pytest

from eval import metrics as M


def ev(kind, frm, to):
    return {"kind": kind, "meta": {"from_lineages": frm}, "lineage_ids": to}


def test_retention_s1_and_s4():
    recs = [
        M.h1s_step_record("rephrase_light", [("L1", ["L1"])], ["L1"], []),
        M.h1s_step_record("move", [("L2", ["L9"])], ["L2"], []),
        # dup_delete: two kept copies preserved, deleted copy has no new block
        M.h1s_step_record(
            "dup_delete", [("A", ["A"]), ("B", ["B"]), ("C", [])], ["A", "B", "C"], []
        ),
        M.h1s_step_record(
            "dup_delete", [("D", ["D"]), ("E", ["X"]), ("F", [])], ["D", "E", "F"], []
        ),
        M.h1s_step_record("insert", [(None, ["N1"])], ["L1"], []),
        M.h1s_step_record(
            "split", [("S", ["S1", "S2"])], ["S"], [ev("BlockSplit", ["S"], ["S1", "S2"])]
        ),
    ]
    m = M.h1s_aggregate(recs)
    assert m["H1-S1"] == 0.5  # 1 of 2
    assert m["H1-S4"] == 0.75  # 3 of 4 kept copies
    assert m["per_op"]["dup_delete"]["retention"] == 0.75


def test_false_match_s2():
    recs = [
        M.h1s_step_record(
            "insert", [(None, ["OLD"])], ["OLD", "X"], []
        ),  # linked to existing lineage
        M.h1s_step_record("insert", [(None, ["NEW"])], ["X"], []),
        M.h1s_step_record("split", [("P", ["P", "Q"])], ["P"], []),  # one part kept the old lineage
        M.h1s_step_record("merge", [("A", ["M"]), ("B", ["M"])], ["A", "B"], []),
    ]
    m = M.h1s_aggregate(
        recs
        + [M.h1s_step_record("move", [("X", ["X"])], ["X"], [])]
        + [M.h1s_step_record("dup_delete", [("D", ["D"]), ("E", ["E"]), ("F", [])], ["D"], [])]
    )
    # truth-new blocks: N(OLD)=linked, N(NEW), P=linked, Q, M → 2 / 5
    assert m["H1-S2"] == pytest.approx(2 / 5)


def test_split_merge_f1_requires_exact_lineage_sets():
    recs = [
        M.h1s_step_record(
            "split", [("S", ["S1", "S2"])], ["S"], [ev("BlockSplit", ["S"], ["S1", "S2"])]
        ),
        M.h1s_step_record(
            "split", [("T", ["T1", "T2"])], ["T"], [ev("BlockSplit", ["T"], ["T1"])]
        ),  # wrong set
        M.h1s_step_record(
            "merge",
            [("A", ["M"]), ("B", ["M"])],
            ["A", "B"],
            [ev("BlockMerged", ["A", "B"], ["M"])],
        ),
        # spurious split predicted during a rephrase
        M.h1s_step_record(
            "rephrase_heavy", [("R", ["R2", "R3"])], ["R"], [ev("BlockSplit", ["R"], ["R2", "R3"])]
        ),
        M.h1s_step_record("insert", [(None, ["N"])], [], []),
        M.h1s_step_record("dup_delete", [("D", ["D"]), ("E", ["E"]), ("F", [])], ["D"], []),
    ]
    m = M.h1s_aggregate(recs)
    d = m["H1-S3_detail"]
    assert (d["tp"], d["n_pred"], d["n_truth"]) == (2, 4, 3)
    p, r = 2 / 4, 2 / 3
    assert m["H1-S3"] == pytest.approx(2 * p * r / (p + r))


def test_empty_denominators_are_errors():
    with pytest.raises(M.MetricError):
        M.h1s_aggregate([M.h1s_step_record("insert", [(None, ["N"])], [], [])])
    with pytest.raises(M.MetricError):
        M.f1(0, 0, 0)
    with pytest.raises(M.MetricError):
        M.h1s_step_record("merge", [("A", ["M"]), ("B", ["Z"])], [], [])


def test_objective():
    m = {"H1-S1": 0.9, "H1-S2": 0.1, "H1-S3": 0.5, "H1-S4": 1.0}
    assert M.h1s_objective(m) == pytest.approx(3.3)
