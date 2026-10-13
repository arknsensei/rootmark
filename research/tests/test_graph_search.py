"""graph_flat / graph_hybrid / hier (SPEC §9): determinism, ego subgraph cap,
PPR restart mass on seeds only, hier restricted to the top-20 documents."""

import shutil
from pathlib import Path

import mdkernel_core as mk
import networkx as nx
import numpy as np
import pytest

from mdkernel import embed as E
from mdkernel import graph as G
from mdkernel import graph_search as GS
from mdkernel import index as I

MINI = Path(__file__).resolve().parents[2] / "spec" / "fixtures" / "mini_vault"


@pytest.fixture(scope="module")
def gr(tmp_path_factory):
    root = tmp_path_factory.mktemp("gs")
    shutil.copytree(MINI, root / "docs")
    v = mk.Vault(str(root))
    v.ingest_dir(str(root / "docs"))
    idx = I.build(v, "structural", root / "lance")
    return GS.GraphRetriever(idx, G.build_graph(idx))


@pytest.mark.parametrize("name", ["graph_flat", "graph_hybrid", "hier"])
def test_deterministic_and_valid(gr, name):
    q = "How long do beans rest before espresso and which origins are in the blend?"
    a = getattr(gr, name)(q, 10)
    b = getattr(gr, name)(q, 10)
    assert a == b and len(a) == 10 and len({k for k, _ in a}) == 10
    assert all(k in gr.index.pos for k, _ in a)
    assert [s for _, s in a] == sorted((s for _, s in a), reverse=True)


def star(n_hub: int, n_leaf: int) -> nx.Graph:
    g = nx.Graph()
    for i in range(n_hub):
        g.add_edge("seed", f"h{i:04d}", weight=(i + 1) / n_hub)
        for j in range(n_leaf):
            g.add_edge(f"h{i:04d}", f"l{i:04d}-{j:02d}", weight=(i + 1) / n_hub)
    return g


def test_ego_subgraph_respects_node_cap_and_keeps_seeds():
    g = star(100, 30)  # 1 + 100 + 3000 nodes within 2 hops
    nodes = GS.ego_subgraph(g, ["seed"])
    assert len(nodes) == GS.MAX_NODES and nodes[0] == "seed"
    assert {f"h{i:04d}" for i in range(100)} <= set(nodes)  # hop 1 admitted first
    # strongest hubs' leaves win the remaining room
    assert "l0099-00" in nodes and "l0000-00" not in nodes
    assert GS.ego_subgraph(g, ["seed"]) == nodes
    small = GS.ego_subgraph(star(3, 2), ["seed"])
    assert len(small) == 1 + 3 + 6


def test_ego_subgraph_union_of_seeds_two_hops():
    g = nx.path_graph(10)
    g = nx.relabel_nodes(g, {i: f"n{i}" for i in range(10)})
    nx.set_edge_attributes(g, 1.0, "weight")
    assert set(GS.ego_subgraph(g, ["n0", "n9"])) == {"n0", "n1", "n2", "n7", "n8", "n9"}


def test_ppr_restart_mass_only_on_seeds():
    g = nx.Graph()
    g.add_edge("s", "a", weight=1.0)
    g.add_edge("a", "b", weight=1.0)
    g.add_node("iso")
    nodes = ["s", "a", "b", "iso"]
    r = GS.personalized_pagerank(g, nodes, ["s"])
    assert r[3] == 0.0  # unreachable, no restart mass
    # the seed and its neighbour outrank the far node; everything reachable is > 0
    assert min(r[0], r[1]) > r[2] > 0
    assert r.sum() == pytest.approx(1.0, abs=1e-6)
    # restart mass is uniform over seeds
    r2 = GS.personalized_pagerank(g, nodes, ["s", "b"])
    assert r2[0] == pytest.approx(r2[2]) and r2[3] == 0.0


def test_ppr_dangling_seed_keeps_all_mass():
    g = nx.Graph()
    g.add_node("s")
    g.add_node("x")
    r = GS.personalized_pagerank(g, ["s", "x"], ["s"])
    assert r[0] == pytest.approx(1.0) and r[1] == 0.0


def test_hier_uses_only_top20_documents(gr):
    idx = gr.index
    q = "descale the espresso machine"
    qv = E.embed_query(q)
    top_docs = sorted(idx.doc_vectors, key=lambda d: (-float(idx.doc_vectors[d] @ qv), d))[
        : GS.HIER_DOCS
    ]
    res = gr.hier(q, 10)
    assert {idx.chunk(k)["doc_path"] for k, _ in res} <= set(top_docs)
    # with a 3-document limit the restriction is visible on mini_vault
    old = GS.HIER_DOCS
    try:
        GS.HIER_DOCS = 3
        top3 = sorted(idx.doc_vectors, key=lambda d: (-float(idx.doc_vectors[d] @ qv), d))[:3]
        assert {idx.chunk(k)["doc_path"] for k, _ in gr.hier(q, 10)} <= set(top3)
    finally:
        GS.HIER_DOCS = old


def test_alpha_one_reduces_graph_flat_to_cosine_order(gr):
    q = "class D extinguisher"
    one = GS.GraphRetriever(gr.index, gr.graph, alpha=1.0)
    res = one.graph_flat(q, 5)
    qv = E.embed_query(q)
    cos = {k: float(gr.index.vectors[gr.index.pos[k]] @ qv) for k, _ in res}
    assert [s for _, s in res] == pytest.approx([cos[k] for k, _ in res], abs=1e-6)


def test_graph_hybrid_seeds_get_full_rrf_weight(gr):
    zero = GS.GraphRetriever(gr.index, gr.graph, alpha=1.0)
    res = zero.graph_hybrid("class D extinguisher next to the roaster", 5)
    assert res[0][1] == pytest.approx(1.0)  # best seed: rrf / max rrf
    assert np.all(np.array([s for _, s in res]) <= 1.0 + 1e-9)
