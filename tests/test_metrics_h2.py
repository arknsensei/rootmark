"""Hand-computed H2/H3/H4 metric examples (EVAL §H2; VERIFY V8)."""

import math

import pytest

from eval import metrics as M

A, B = {"a1", "a2"}, {"b1"}


def rel(*chunks, groups=(A, B)):
    return M.relevance(chunks, groups)


def test_relevance_is_lineage_intersection():
    r = rel(["x"], ["a2", "z"], ["b1"], ["a1", "b1"])
    assert r == [set(), {0}, {1}, {0, 1}]
    with pytest.raises(M.MetricError):
        M.relevance([["x"]], [set()])


def test_ndcg_min_cover_2_three_chunks():
    # ranks: miss, A, B → DCG = 1/log2(3) + 1/log2(4); IDCG (2) = 1 + 1/log2(3)
    r = rel(["x"], ["a1"], ["b1"])
    exp = (1 / math.log2(3) + 1 / math.log2(4)) / (1 + 1 / math.log2(3))
    assert M.ndcg_at_k(r, 2) == pytest.approx(exp)
    assert M.ndcg_at_k(rel(["a1"], ["b1"]), 2) == pytest.approx(1.0)


def test_ndcg_above_one_is_an_error():
    # three relevant chunks on top but min_cover=2 claims only two exist
    with pytest.raises(M.MetricError):
        M.ndcg_at_k(rel(["a1"], ["a2"], ["b1"]), 2)
    with pytest.raises(M.MetricError):
        M.ndcg_at_k(rel(["a1"]), 0)


def test_full_cover_and_recall_multi():
    only_a = [["a1"], ["a2"], ["x"]]
    assert M.full_cover_at_k(rel(*only_a), 2) == 0.0
    assert M.recall_at_k(only_a, [A, B], 10) == 0.5
    both = [["x"], ["a1"], ["b1"]]
    assert M.full_cover_at_k(rel(*both), 2) == 1.0
    # v1 recall: fraction of each group's lineages covered, averaged over groups
    assert M.recall_at_k(both, [A, B], 10) == pytest.approx((0.5 + 1.0) / 2)
    assert M.recall_at_k([["x"], ["a1"]], [A], 2) == 0.5  # v1 test case
    assert M.recall_at_k([["x"], ["a1"]], [A], 1) == 0.0
    q = M.query_metrics([["x"], ["a1", "a2"]], [A], 1)
    assert q["full_cover@10"] == q["recall@10"] == 1.0  # single-hop: full_cover = recall@10
    q = M.query_metrics(both, [A, B], 2)
    assert q["full_cover@10"] == 1.0 and q["recall@10"] == 0.75


def test_mrr():
    assert M.mrr(rel(["x"], ["x"], ["b1"])) == pytest.approx(1 / 3)
    assert M.mrr(rel(["x"])) == 0.0


def test_ideal_relevant_count():
    corpus = [["a1"], ["a2", "z"], ["b1"], ["q"], ["a1", "b1"]]
    assert M.ideal_relevant_count(corpus, [A, B]) == 4
    assert M.ideal_relevant_count(corpus * 5, [A, B]) == 10  # capped at k
    with pytest.raises(M.MetricError):
        M.ideal_relevant_count([["q"]], [A])
    # with the ideal count, a ranking of all relevant chunks scores exactly 1
    ranked = [["a1"], ["a2", "z"], ["b1"], ["a1", "b1"]]
    assert M.ndcg_at_k(rel(*ranked), M.ideal_relevant_count(corpus, [A, B])) == pytest.approx(1.0)


def test_lineage_loss_and_h3_summary():
    assert M.lineage_loss(["a", "b", "c"], ["b", "c", "d"]) == {
        "lost": 1,
        "gained": 1,
        "A": 3,
        "B": 3,
    }
    assert M.lineage_loss(["a"], ["a"])["lost"] == 0
    h = M.h3_summary([10.0, 20.0, 30.0, 40.0], lost=0, gained=0, log_equal=True)
    assert h["p50"] == pytest.approx(25.0) and h["p95"] == pytest.approx(38.5) and h["n"] == 4
    assert h["lost"] == 0 and h["log_equal"] is True


def test_latency_and_bootstrap():
    ms = [100.0] * 10 + list(range(1, 101))
    s = M.latency_summary(ms)
    assert s["n"] == 100 and s["p50"] == pytest.approx(50.5) and s["p95"] == pytest.approx(95.05)
    a, b = [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]
    bt = M.paired_bootstrap(a, b, n=1000)
    assert bt["delta"] == 1.0 and bt["ci95"] == [1.0, 1.0]
    assert M.paired_bootstrap([0.3, 0.1], [0.1, 0.3], n=500) == M.paired_bootstrap(
        [0.3, 0.1], [0.1, 0.3], n=500
    )
    with pytest.raises(M.MetricError):
        M.paired_bootstrap([1.0], [])


def test_max_relative_difference():
    a = {"h2": {"x": 1.0, "y": 0.5}, "n": 3}
    b = {"h2": {"x": 1.0, "y": 0.25}, "n": 3}
    r = M.max_relative_difference(a, b, ["h2.x", "h2.y", "n"])
    assert r["max_relative_difference"] == pytest.approx(0.5)
    assert list(r["differing_fields"]) == ["h2.y"]
    assert M.max_relative_difference(a, a, ["h2.x", "h2.y", "n"])["max_relative_difference"] == 0.0
    with pytest.raises(M.MetricError):
        M.max_relative_difference(a, b, ["h2.missing"])
    with pytest.raises(M.MetricError):
        M.max_relative_difference({"x": float("nan")}, {"x": 1.0}, ["x"])
    with pytest.raises(M.MetricError):
        M.max_relative_difference({"x": float("inf")}, {"x": float("inf")}, ["x"])
    assert M.max_relative_difference({"x": 1}, {"x": True}, ["x"])[
        "max_relative_difference"
    ] == float("inf")
    assert M.max_relative_difference({"x": "a"}, {"x": "b"}, ["x"])[
        "max_relative_difference"
    ] == float("inf")
    assert M.max_relative_difference({"x": 2}, {"x": 2.0}, ["x"])["max_relative_difference"] == 0.0
