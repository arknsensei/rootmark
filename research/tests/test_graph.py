"""Chunk graph (SPEC §8): keys, wikilink targets incl. [[D#h]], hierarchy, knn, cap."""

import networkx as nx
import numpy as np
import pytest

from rootmark import graph as G
from rootmark.index import Index


def chunk(doc, cid, hp, links=(), text="t"):
    return {
        "key": f"{doc}::{cid}",
        "doc_path": doc,
        "chunk_id": cid,
        "lineage_ids": [f"{doc}{cid}"],
        "heading_path": list(hp),
        "text": text,
        "wikilinks": [{"target": t, "heading": h} for t, h in links],
    }


def make_index(chunks, vectors=None):
    n = len(chunks)
    if vectors is None:
        vectors = np.eye(n, 8, dtype=np.float32)
    return Index("structural", chunks, vectors.astype(np.float32), table=None, bm25=None)


@pytest.fixture
def small():
    chunks = [
        chunk("a.md", "1", ["A"], links=[("b", None)]),
        chunk("a.md", "2", ["A", "X"], links=[("c", "Usage")]),
        chunk("b.md", "1", ["B"]),
        chunk("b.md", "2", ["B", "P"]),
        chunk("b.md", "3", ["B", "Q"]),
        chunk("c.md", "1", ["C"]),
        chunk("c.md", "2", ["C", "Usage"]),
        chunk("c.md", "3", ["C", "Usage", "Deep"]),
        chunk("dir/d.md", "1", ["D"], links=[("missing", None), ("C", "nope")]),
    ]
    return make_index(chunks)


def test_nodes_are_doc_chunk_keys(small):
    g = G.build_graph(small)
    assert set(g.nodes) == set(small.keys)
    assert all("::" in n for n in g.nodes)


def test_wikilink_whole_document(small):
    g = G.build_graph(small)
    for b in ("b.md::1", "b.md::2", "b.md::3"):
        assert g.has_edge("a.md::1", b) and "wikilink" in g["a.md::1"][b]["kinds"]


def test_wikilink_heading_only_under_heading(small):
    g = G.build_graph(small)
    linked = {n for n in g["a.md::2"] if "wikilink" in g["a.md::2"][n]["kinds"]}
    assert linked == {"c.md::2", "c.md::3"}  # chunks under "Usage" (any depth), not c.md::1


def test_unresolvable_heading_falls_back_to_document_and_case_insensitive_target(small):
    g = G.build_graph(small)
    linked = {n for n in g["dir/d.md::1"] if "wikilink" in g["dir/d.md::1"][n]["kinds"]}
    assert linked == {"c.md::1", "c.md::2", "c.md::3"}


def test_hierarchy_edges(small):
    g = G.build_graph(small)
    # consecutive chunks of b.md, and b.md::2 / b.md::3 share parent heading "B"
    assert g["b.md::1"]["b.md::2"]["kinds"] == {"hierarchy": 1}
    assert g["b.md::1"]["b.md::2"]["weight"] == pytest.approx(G.W_CONSECUTIVE)
    assert g["b.md::2"]["b.md::3"]["weight"] == pytest.approx(G.W_CONSECUTIVE + G.W_SAME_PARENT)
    assert "hierarchy" not in g["b.md::1"].get("b.md::3", {"kinds": {}})["kinds"]


def test_weights_sum_and_cap_at_one():
    # a.md::1 links to b (1.0) and is consecutive with... separate docs: use knn + wikilink
    chunks = [chunk("a.md", "1", ["A"], links=[("b", None)]), chunk("b.md", "1", ["B"])]
    v = np.array([[1, 0, 0, 0], [1, 0, 0, 0]], dtype=np.float32)
    g = G.build_graph(make_index(chunks, v))
    d = g["a.md::1"]["b.md::1"]
    assert d["kinds"] == {"wikilink": 1, "knn": 2}  # knn both directions + wikilink
    assert d["weight"] == G.CAP
    stats = G.edge_stats(g)
    assert stats["weight"]["max"] <= 1.0 and stats["weight"]["at_cap"] == 1


def test_knn_weight_is_scaled_cosine():
    chunks = [chunk("a.md", str(i), ["A" + str(i)]) for i in range(3)]
    v = np.array([[1, 0], [0.6, 0.8], [0, 1]], dtype=np.float32)
    g = G.build_graph(make_index(chunks, v))
    # a.md::0 ↔ a.md::1: consecutive 0.5 + knn 0.3·0.6 twice (each side's top list)
    assert g["a.md::0"]["a.md::1"]["weight"] == pytest.approx(min(1.0, 0.5 + 2 * 0.3 * 0.6))
    assert "knn" in g["a.md::0"]["a.md::2"]["kinds"] and g["a.md::0"]["a.md::2"][
        "weight"
    ] == pytest.approx(0.0)


def test_duplicate_chunk_in_two_documents_gives_two_nodes():
    chunks = [chunk("a.md", "same", ["H"]), chunk("b.md", "same", ["H"])]
    g = G.build_graph(make_index(chunks))
    assert {"a.md::same", "b.md::same"} <= set(g.nodes)


def test_save_load_roundtrip(small, tmp_path):
    g = G.build_graph(small)
    p = G.graph_path(tmp_path, "structural")
    G.save(g, p)
    h = G.load(p)
    assert nx.utils.graphs_equal(g, h) and p.name == "graph_structural.gpickle"
