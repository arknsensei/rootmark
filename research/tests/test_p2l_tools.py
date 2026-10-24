"""P2-L measurement tools (TASKS L4): D-TR's building blocks, the doc-level
metrics, the criteria arithmetic, and the 12-cell path on a tiny synthetic
set (never on D-TR: its numbers are the round's measurement)."""

from __future__ import annotations

import json
import random

import pytest

from eval import run_p2l as R
from eval.datasets import d_tr as D


def test_the_seed_is_the_pre_registered_one():
    import blake3

    assert D.SEED == int(blake3.blake3(b"P2-L/D-TR").hexdigest()[:8], 16) == 3775637042


def test_eligible_lead_and_sentences():
    para = "Bir iki üç dört beş altı yedi sekiz dokuz. " * 20
    text = "\n\n".join([para] * 5)
    assert D.eligible("İstanbul", text)
    assert not D.eligible("Film (anlam ayrımı)", text)
    assert not D.eligible("Kategori:Şehirler", text)
    assert not D.eligible("Kısa", "a\n\nb\n\nc\n\nd")
    lead = D.lead(text)
    assert lead.count("\n\n") + 1 <= D.LEAD_MAX and len(lead) >= D.LEAD_CHARS
    assert all(len(s) >= D.MIN_SENT_WORDS for s in D.sentences(text))


def test_a_window_is_seeded_and_within_one_sentence():
    text = "Kısa cümle. Türk kahvesi cezvede kısık ateşte yavaşça pişirilir ve köpüğü korunur."
    a = D.window(random.Random(7), text, "x")
    b = D.window(random.Random(7), text, "x")
    assert a == b
    n = len(a.split())
    assert D.WINDOW[0] <= n <= D.WINDOW[1]
    assert a in text
    with pytest.raises(RuntimeError):
        D.window(random.Random(1), "Çok kısa.", "x")


def test_doc_metrics():
    hits = [{"doc_path": p} for p in ["a", "a", "b", "c"]]
    m = R.doc_metrics(hits, "b")
    assert m["rank"] == 2 and m["recall@10"] == 1.0 and m["mrr"] == 0.5
    assert abs(m["ndcg@10"] - 1 / 1.584962500721156) < 1e-12
    assert R.doc_metrics(hits, "z") == {"rank": None, "ndcg@10": 0.0, "recall@10": 0.0, "mrr": 0.0}
    far = [{"doc_path": f"d{i}"} for i in range(12)]
    m = R.doc_metrics(far, "d11")
    assert m["rank"] == 12 and m["ndcg@10"] == 0.0 and m["recall@10"] == 0.0 and m["mrr"] == 1 / 12


def test_criteria_arithmetic():
    cells = {}
    for mode in R.MODES:
        for tok in R.TOKENIZERS:
            for emb in R.EMBEDDERS:
                cells[f"{mode}/{tok}/{emb}"] = {"ndcg@10": 0.5}
    cells["lexical/uni_fold_tr/minilm-l6-v2"]["ndcg@10"] = 0.61
    cells["flat/mdk_lower_alnum/multilingual-minilm-l12-v2"]["ndcg@10"] = 0.54
    cells["hybrid/uni_fold_tr/multilingual-minilm-l12-v2"]["ndcg@10"] = 0.59
    ref_d = {"hybrid/structural/single": 0.0005, "flat/structural/multi": -0.0009}
    en = {
        "mdk_lower_alnum": {"vs_reference": {"compared": 618, "differing": [], "pass": True}},
        "uni_fold_tr": {"vs_reference": {"differing": ["x"]}, "delta_ndcg@10": ref_d},
    }
    c = R.criteria({"cells": cells}, en)
    assert c["L-2"]["pass"] and abs(c["L-2"]["value"] - 0.11) < 1e-12
    assert not c["L-5"]["pass"]
    assert not c["L-6"]["pass"] and abs(c["L-6"]["value"] - 0.09) < 1e-12
    assert c["L-3"]["pass"]
    assert c["L-4"]["default_tokenizer"] == "uni_fold_tr"
    en["uni_fold_tr"]["delta_ndcg@10"] = {"hybrid/structural/single": 0.0011}
    assert R.criteria(None, en)["L-4"]["default_tokenizer"] == "mdk_lower_alnum"


def test_twelve_cells_on_a_tiny_set(tmp_path, monkeypatch):
    """The cell path end to end (both embedders, both variants, three modes)
    on three synthetic documents: duplicate cells equal, the Turkish
    capitals found by `uni_fold_tr` only."""
    docs = tmp_path / "docs"
    (docs / "tr").mkdir(parents=True)
    (docs / "tr" / "1.md").write_text("# İSTANBUL\n\nBoğaz kıyısında ILIK bir akşam.\n")
    (docs / "tr" / "2.md").write_text("# Kahve\n\nTürk kahvesi değerlendirme notları.\n")
    (docs / "tr" / "3.md").write_text("# Borsa\n\nHisse senetleri bugün yükseldi.\n")
    queries = [
        {"id": "q1", "type": "title", "lang": "tr", "query": "istanbul", "truth": "tr/1.md"},
        {"id": "q2", "type": "sentence", "lang": "tr", "query": "ılık akşam", "truth": "tr/1.md"},
        {
            "id": "q3",
            "type": "sentence",
            "lang": "tr",
            "query": "kahvesi değerlendirme",
            "truth": "tr/2.md",
        },
    ]
    monkeypatch.setattr(D, "load_set", lambda: (docs, queries))
    man = tmp_path / "MANIFEST.json"
    man.write_text(json.dumps({"set": "tiny"}))
    monkeypatch.setattr(D, "MANIFEST", man)
    out = R.d_tr_cells(tmp_path / "work")
    assert len(out["cells"]) == 12
    assert all(d["equal"] for d in out["duplicates_equal"])
    q1 = {k: next(r for r in v if r["id"] == "q1")["rank"] for k, v in out["per_query"].items()}
    assert q1["lexical/uni_fold_tr/minilm-l6-v2"] == 1
    assert q1["lexical/mdk_lower_alnum/minilm-l6-v2"] is None
    for k, b in out["builds"].items():
        tok, emb = k.split("/")
        assert b["manifest"]["lexical"]["tokenizer"] == tok
        assert b["manifest"]["embedder_id"].startswith(emb + "/")
