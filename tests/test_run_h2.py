"""run_h2 guards: ALPHA must equal the tuning selection; split isolation;
per-cell query counts; selection rule."""

import json
from typing import ClassVar

import pytest

from eval import run_h2 as R
from mdkernel import graph_search as GS


def test_select_alpha_rule():
    c = [{"alpha": 0.3, "ndcg_all": 0.5}, {"alpha": 0.8, "ndcg_all": 0.7}]
    assert R.select_alpha(c) == 0.8
    tie = [
        {"alpha": 0.3, "ndcg_all": 0.7},
        {"alpha": 0.5, "ndcg_all": 0.7},
        {"alpha": 0.7, "ndcg_all": 0.7},
    ]
    assert R.select_alpha(tie) == 0.5  # closest to 0.6, then smaller


def test_test_phase_refuses_mismatched_alpha(monkeypatch):
    monkeypatch.setattr(
        R.runinfo,
        "read_json",
        lambda run, name: {"selected_alpha": GS.ALPHA_DEFAULT + 0.1, "tune_ids": []},
    )
    with pytest.raises(RuntimeError, match="ALPHA_DEFAULT"):
        R.phase_test("20261013T000000Z-abcdef0")


@pytest.mark.skipif(not R.SPLITS.exists(), reason="needs data/splits/d4.json")
def test_load_split_returns_only_that_split():
    s = json.loads(R.SPLITS.read_text())
    tune = R.load_split("tune")
    assert {q["id"] for q in tune} == set(s["tune"])
    assert not {q["id"] for q in tune} & set(s["test"])


def test_cell_query_count_mismatch_is_an_error(monkeypatch):
    class FakeIdx:
        chunks: ClassVar[list] = [{"lineage_ids": ["a"]}] * 10

        @staticmethod
        def chunk(k):
            return {"lineage_ids": ["a"]}

    class FakeRetriever:
        def flat(self, q, k):
            return [(f"k{i}", 1.0) for i in range(k)]

    qs = [{"id": "s1", "type": "single", "question": "q", "truth": [[]]}]
    rows = R.run_queries(FakeRetriever(), "flat", FakeIdx, qs, [[{"a"}]])
    assert len(rows) == 1 and rows[0]["ndcg@10"] > 0
    with pytest.raises(RuntimeError, match="results for"):

        class Short(FakeRetriever):
            def flat(self, q, k):
                return [("k0", 1.0)]

        R.run_queries(Short(), "flat", FakeIdx, qs, [[{"a"}]])
