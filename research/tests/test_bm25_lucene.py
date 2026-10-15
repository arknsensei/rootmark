"""eval.bm25_lucene (P1-C.1/C1.1): the K-C1.1 formula on a hand-computed corpus."""

import math

from eval import bm25_lucene as L


def test_hand_computed_scores():
    corpus = [["a", "b", "a"], ["b", "c"], ["d"]]
    bm = L.LuceneBM25(corpus)
    assert bm.avgdl == 2.0
    s, hit = bm.scores(["a", "c"])
    idf_a = math.log(1 + (3 - 1 + 0.5) / (1 + 0.5))
    idf_c = idf_a
    want0 = idf_a * 2.0 * 2.2 / (2.0 + 1.2 * (1 - 0.75 + 0.75 * 3 / 2))
    want1 = idf_c * 1.0 * 2.2 / (1.0 + 1.2 * (1 - 0.75 + 0.75 * 2 / 2))
    assert s == [want0, want1, 0.0]
    assert hit == [True, True, False]


def test_idf_is_positive_for_a_term_in_every_document():
    assert L.idf(3, 3) > 0  # Okapi's ln((N - n + 0.5)/(n + 0.5)) would be negative


def test_repeated_query_token_counts_again():
    bm = L.LuceneBM25([["x"], ["y"]])
    one, _ = bm.scores(["x"])
    two, _ = bm.scores(["x", "x"])
    assert two[0] == 2 * one[0]


def test_unknown_terms_and_empty_corpus():
    s, hit = L.LuceneBM25([["x"]]).scores(["zzz"])
    assert s == [0.0] and hit == [False]
    assert L.LuceneBM25([[]]).avgdl == 1.0
