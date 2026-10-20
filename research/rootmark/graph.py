"""Chunk graph (SPEC §8). Nodes are index keys (`doc_path::chunk_id`).

Edge kinds and weights; parallel contributions are summed, capped at 1.0:
- wikilink 1.0: a chunk containing `[[D]]` → every chunk of document D;
  `[[D#h]]` → only D's chunks under heading h (whole D if h is not found).
- hierarchy 0.5 between consecutive chunks of one document; 0.3 between chunks
  of one document with the same parent heading (`heading_path[:-1]`, non-empty).
- knn 0.3·cos from each chunk to its 5 nearest chunks (ties at the 5th
  similarity broken by key, ascending).
"""

from __future__ import annotations

import itertools
import pickle
from pathlib import Path, PurePosixPath

import networkx as nx
import numpy as np

from rootmark.index import Index

W_WIKILINK = 1.0
W_CONSECUTIVE = 0.5
W_SAME_PARENT = 0.3
W_KNN = 0.3
KNN = 5
CAP = 1.0


def doc_key(doc_path: str) -> str:
    """Wikilink target name of a document: file name without `.md`."""
    name = PurePosixPath(doc_path).name
    return name[:-3] if name.lower().endswith(".md") else name


def _norm(s: str) -> str:
    return " ".join(s.lower().split())


def _add(g: nx.Graph, a: str, b: str, w: float, kind: str) -> None:
    if a == b:
        return
    if g.has_edge(a, b):
        d = g[a][b]
        d["weight"] = min(CAP, d["weight"] + w)
        d["kinds"][kind] = d["kinds"].get(kind, 0) + 1
    else:
        g.add_edge(a, b, weight=min(CAP, w), kinds={kind: 1})


def build_graph(index: Index) -> nx.Graph:
    chunks, keys = index.chunks, index.keys
    g = nx.Graph()
    g.add_nodes_from(keys)
    by_doc = index.doc_rows
    by_name: dict[str, list[str]] = {}
    for d in sorted(by_doc):
        by_name.setdefault(doc_key(d), []).append(d)
    by_name_ci: dict[str, list[str]] = {}
    for name, docs in by_name.items():
        by_name_ci.setdefault(name.lower(), []).extend(docs)
    sections = [{_norm(h) for h in c["heading_path"]} for c in chunks]

    for i, c in enumerate(chunks):
        links = dict.fromkeys((w["target"], w["heading"]) for w in c["wikilinks"])
        for target, heading in links:
            for d in by_name.get(target) or by_name_ci.get(target.lower()) or []:
                members = by_doc[d]
                if heading is not None:
                    under = [j for j in members if _norm(heading) in sections[j]]
                    members = under or members
                for j in members:
                    _add(g, keys[i], keys[j], W_WIKILINK, "wikilink")

    for _d, members in sorted(by_doc.items()):
        for a, b in itertools.pairwise(members):
            _add(g, keys[a], keys[b], W_CONSECUTIVE, "hierarchy")
        groups: dict[tuple[str, ...], list[int]] = {}
        for j in members:
            hp = chunks[j]["heading_path"]
            if len(hp) >= 2:
                groups.setdefault(tuple(hp[:-1]), []).append(j)
        for grp in groups.values():
            for x in range(len(grp)):
                for y in range(x + 1, len(grp)):
                    _add(g, keys[grp[x]], keys[grp[y]], W_SAME_PARENT, "hierarchy")

    v = index.vectors
    n = len(keys)
    if n > 1:
        k = min(KNN, n - 1)
        for start in range(0, n, 1024):
            sims = v[start : start + 1024] @ v.T
            for r in range(sims.shape[0]):
                i = start + r
                sims[r, i] = -np.inf
                # the k nearest, ties at the k-th similarity broken by key:
                # `argpartition` alone picks among equal similarities in a
                # platform-dependent order (P2-A/A4: a different graph on
                # Linux); without a tie at the boundary the set is the same
                kth = np.partition(-sims[r], k - 1)[k - 1]
                cand = np.nonzero(-sims[r] <= kth)[0]
                top = sorted(cand, key=lambda j: (-sims[r, j], keys[j]))[:k]
                for j in top:
                    _add(g, keys[i], keys[j], W_KNN * float(sims[r, j]), "knn")
    return g


def edge_stats(g: nx.Graph) -> dict:
    kinds: dict[str, int] = {}
    combos: dict[str, int] = {}
    weights = []
    for _a, _b, d in g.edges(data=True):
        for k in d["kinds"]:
            kinds[k] = kinds.get(k, 0) + 1
        combo = "+".join(sorted(d["kinds"]))
        combos[combo] = combos.get(combo, 0) + 1
        weights.append(d["weight"])
    w = np.asarray(weights) if weights else np.zeros(1)
    degrees = np.asarray([deg for _n, deg in g.degree()]) if g.number_of_nodes() else np.zeros(1)
    return {
        "nodes": g.number_of_nodes(),
        "edges": g.number_of_edges(),
        "edges_by_kind": dict(sorted(kinds.items())),
        "edges_by_combination": dict(sorted(combos.items())),
        "weight": {
            "min": float(w.min()),
            "mean": float(w.mean()),
            "max": float(w.max()),
            "at_cap": int((w >= CAP).sum()),
        },
        "degree": {"mean": float(degrees.mean()), "max": int(degrees.max())},
    }


def save(g: nx.Graph, path: Path) -> None:
    with open(path, "wb") as f:
        pickle.dump(g, f, protocol=pickle.HIGHEST_PROTOCOL)


def load(path: Path) -> nx.Graph:
    with open(path, "rb") as f:
        return pickle.load(f)


def graph_path(db_dir: Path, strategy: str) -> Path:
    return Path(db_dir) / f"graph_{strategy}.gpickle"
