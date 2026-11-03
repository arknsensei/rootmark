"""P2-Q query evaluator (GOAL Q-6; TASKS Q0 "sorgu değerlendirici").

A driver records every registered query's hits per phase as

    {"<qid>": {"default": [hit, ...], "include_proposal": [hit, ...]}}

where a hit is {"doc", "layer", "score", "chunk_id", "lineage_ids"} and
`doc` is the vault path for synthetic and open documents and "b2:<blake2b>"
for a personal corpus's own document (common.b2 of the path). This module
compares them with the scenario's expectations; it returns failures (an
empty list is a pass) and never reads a corpus.
"""

from __future__ import annotations

LEAK_LAYERS = {"proposal", "unverified"}


def docs(hits: list[dict]) -> list[str]:
    seen, out = set(), []
    for h in hits:
        if h["doc"] not in seen:
            seen.add(h["doc"])
            out.append(h["doc"])
    return out


def default_leaks(hits: list[dict]) -> list[dict]:
    """Hits of a default search (no include_proposal) that are not canonical."""
    return [h for h in hits if h.get("layer") != "canonical"]


def check_query(q: dict, phase: str, got: dict) -> list[str]:
    """Failures of one headless query at one phase."""
    out = []
    exp = q["expect"][phase]
    d, ip = got.get("default"), got.get("include_proposal")
    if d is None or ip is None:
        return [f"{q['id']}@{phase}: no result recorded"]
    for h in default_leaks(d):
        out.append(f"{q['id']}@{phase}: default hit {h['doc']} is {h.get('layer')}")
    dd, ipd = docs(d), docs(ip)
    if q["match"] == "exact":
        if sorted(dd) != sorted(exp["default"]):
            out.append(f"{q['id']}@{phase}: default docs {sorted(dd)} != {sorted(exp['default'])}")
        if sorted(ipd) != sorted(exp["include_proposal"]):
            out.append(
                f"{q['id']}@{phase}: include_proposal docs {sorted(ipd)} != "
                f"{sorted(exp['include_proposal'])}"
            )
    else:  # contains: the targets are (not) among the hits
        for t in q["targets"]:
            want_default = t in exp["default"]
            if want_default != (t in dd):
                out.append(
                    f"{q['id']}@{phase}: target {t} in default hits: {t in dd}, "
                    f"expected {want_default}"
                )
            if t in exp["include_proposal"] and t not in ipd:
                out.append(f"{q['id']}@{phase}: target {t} missing with include_proposal")
    if exp.get("layer"):
        for t in q["targets"]:
            layers = sorted({h.get("layer") for h in ip if h["doc"] == t})
            if layers and layers != [exp["layer"]]:
                out.append(f"{q['id']}@{phase}: target {t} layers {layers} != {exp['layer']}")
    return out


def check_phase(queries: list[dict], phase: str, results: dict) -> list[str]:
    out = []
    for q in queries:
        out += check_query(q, phase, results.get(q["id"], {}))
    return out


def check_obsidian_query(q: dict, hits: list[dict]) -> list[str]:
    out = [f"{q['id']}: default hit {h['doc']} is {h.get('layer')}" for h in default_leaks(hits)]
    dd = docs(hits)
    if q.get("match") == "contains":
        for t in q["expect_default"]:
            if t not in dd:
                out.append(f"{q['id']}: target {t} not among the hits")
    elif sorted(dd) != sorted(q["expect_default"]):
        out.append(f"{q['id']}: docs {sorted(dd)} != {sorted(q['expect_default'])}")
    return out


def direct_write_canonical(blames: dict[str, list[dict]], seqs: set[int]) -> int:
    """Blocks created by the agent's direct writes (their `seq_created` among
    the watcher's ingest events of those writes) that are canonical (Q-6: 0)."""
    n = 0
    for blocks in blames.values():
        for b in blocks:
            if b.get("seq_created") in seqs and b.get("layer") == "canonical":
                n += 1
    return n
