"""Index, flat/hybrid retrieval, hot layer and watcher on mini_vault."""

import shutil
from pathlib import Path

import mdkernel_core as mk
import numpy as np
import pytest

from mdkernel import embed as E
from mdkernel import index as I
from mdkernel.hot import HotLayer
from mdkernel.retrieval import Retriever, rrf
from mdkernel.watcher import Watcher

MINI = Path(__file__).resolve().parent / "fixtures" / "mini_vault"


@pytest.fixture(scope="module")
def built(tmp_path_factory):
    root = tmp_path_factory.mktemp("idx")
    docs = root / "docs"
    shutil.copytree(MINI, docs)
    v = mk.Vault(str(root))
    v.ingest_dir(str(docs))
    idx = I.build(v, "structural", root / "lance")
    return root, docs, v, idx


def test_keys_and_embeddings(built):
    _, _, v, idx = built
    assert len(idx.keys) == len(set(idx.keys)) == idx.table.count_rows()
    assert all(k == f"{c['doc_path']}::{c['chunk_id']}" for k, c in zip(idx.keys, idx.chunks))
    norms = np.linalg.norm(idx.vectors, axis=1)
    assert np.allclose(norms, 1.0, atol=1e-5)
    assert set(idx.doc_vectors) == set(v.doc_paths())
    # embedding is deterministic for the same input
    a, b = E.embed(["espresso recipe"]), E.embed(["espresso recipe"])
    assert np.array_equal(a, b)


def test_hybrid_returns_k_valid_keys(built):
    _, _, _, idx = built
    r = Retriever(idx)
    out = r.hybrid("x", 10)
    assert len(out) == 10 and len({k for k, _ in out}) == 10
    assert all(k in idx.pos for k, _ in out)
    assert r.hybrid("x", 10) == out  # deterministic


def test_flat_and_hybrid_find_the_obvious_chunk(built):
    _, _, _, idx = built
    r = Retriever(idx)
    top = r.flat("How long is an espresso shot and what pressure is used?", 3)
    assert any(k.startswith("espresso.md::") for k, _ in top)
    top = r.hybrid("class D extinguisher next to the roaster", 3)
    assert top[0][0].startswith("safety.md::")
    scores = [s for _, s in r.flat("roast", 10)]
    assert scores == sorted(scores, reverse=True)


def test_rrf_definition():
    f = rrf([("a", 9.0), ("b", 1.0)], [("b", 5.0), ("c", 1.0)])
    assert f["a"] == pytest.approx(1 / 61)
    assert f["b"] == pytest.approx(1 / 62 + 1 / 61)
    assert f["c"] == pytest.approx(1 / 62)


def test_hot_layer_shadows_cold_and_hides_deleted(built):
    _, _, _, idx = built
    hot = HotLayer(idx)
    r = Retriever(idx, hot)
    target = next(c for c in idx.chunks if c["doc_path"] == "grinders.md")
    shadow = dict(target, text="Completely new text about volcanic eruptions and lava flows.")
    hot.add([shadow])
    res = r.flat("volcanic eruptions and lava", 3)
    assert res[0][0] == target["key"]
    assert res[0][1] == pytest.approx(
        float(hot.matrix[0] @ E.embed_query("volcanic eruptions and lava"))
    )
    assert [k for k, _ in r.flat("volcanic eruptions and lava", 10)].count(target["key"]) == 1
    hb = r.hybrid("volcanic lava", 5)
    assert hb[0][0] == target["key"]
    other = next(c for c in idx.chunks if c["doc_path"] == "staff.md")
    hot.delete([other["key"]])
    assert other["key"] not in [k for k, _ in r.flat("training baristas shifts", 20)]


def test_compact_moves_hot_to_cold(tmp_path):
    docs = tmp_path / "docs"
    shutil.copytree(MINI, docs)
    v = mk.Vault(str(tmp_path))
    v.ingest_dir(str(docs))
    idx = I.build(v, "structural", tmp_path / "lance")
    hot = HotLayer(idx)
    n0 = idx.table.count_rows()
    c = dict(
        idx.chunks[0], key="new.md::x", doc_path="new.md", chunk_id="x", text="brand new chunk"
    )
    hot.add([c])
    gone = idx.chunks[1]["key"]
    hot.delete([gone])
    hot.compact()
    assert len(hot) == 0
    assert idx.table.count_rows() == n0
    assert "new.md::x" in idx.pos and gone not in idx.pos
    assert Retriever(idx).flat("brand new chunk", 1)[0][0] == "new.md::x"


def test_watcher_reindexes_changed_lineages(tmp_path):
    docs = tmp_path / "docs"
    shutil.copytree(MINI, docs)
    v = mk.Vault(str(tmp_path))
    v.ingest_dir(str(docs))
    idx = I.build(v, "structural", tmp_path / "lance")
    hot = HotLayer(idx)
    w = Watcher(v, hot, "structural", docs)
    p = docs / "grinders.md"
    p.write_text(
        p.read_text() + "\nGrinders also need a monthly burr inspection for chipped teeth.\n"
    )
    out = w.process(p)
    assert out["events"] > 0 and out["hot_added"]
    r = Retriever(idx, hot)
    assert r.flat("monthly burr inspection chipped teeth", 1)[0][0] in out["hot_added"]
    # unchanged file → nothing new
    assert w.process(p)["hot_added"] == []


def test_hot_bm25_uses_cold_statistics(built):
    _, _, _, idx = built
    hot = HotLayer(idx)
    c = idx.chunks[5]
    hot.add([dict(c)])  # identical text → identical BM25 score as in the cold model
    q = "roast drum temperature"
    cold_score = float(idx.bm25.get_scores(I.bm25_tokens(q))[5])
    assert hot.bm25(q, 1)[0][1] == pytest.approx(cold_score)


def test_bm25_ranks_only_matching_documents(built):
    _, _, _, idx = built
    r = Retriever(idx)
    assert r._bm25("zzzunknownterm", 50) == []
    hits = r._bm25("extinguisher", 50)
    assert hits and all(s > 0 for _, s in hits)


def test_hot_vector_search_keeps_negative_cosines(built):
    _, _, _, idx = built
    hot = HotLayer(idx)
    hot.add([dict(idx.chunks[0])], vectors=-idx.vectors[:1])
    assert len(hot.search_vec(idx.vectors[0], 5)) == 1
    assert hot.bm25("zzzunknownterm", 5) == []


def test_hot_key_shadows_cold_even_when_not_in_hot_topk(built):
    _, _, _, idx = built
    hot = HotLayer(idx)
    r = Retriever(idx, hot)
    q = "class D extinguisher next to the roaster"
    cold_top = r.flat(q, 1)[0][0]
    # same key, unrelated text: the stale cold entry must not come back
    hot.add([dict(idx.chunk(cold_top), text="unrelated words about tulips")])
    res = dict(r.flat(q, 10))
    assert cold_top not in res or res[cold_top] == pytest.approx(
        float(hot.matrix[0] @ E.embed_query(q))
    )
    assert len(r.flat(q, 10)) == 10
    hot.delete([k for k, _ in Retriever(idx).flat(q, 15)])
    assert len(r.flat(q, 10)) == 10
