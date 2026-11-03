"""P2-Q (TASKS Q0, Q2 (e)): the pre-registered scenario, the query evaluator,
the content leak check and the observation script, on their own small
fixtures (no personal corpus is read here)."""

import json
import random
from pathlib import Path

from p2q import evaluate as E
from p2q import leakcheck as L
from p2q.common import b2, corpus_summary, list_md

REPO = Path(__file__).resolve().parents[2]
SCEN = json.loads((REPO / "research/p2q/scenario.json").read_text(encoding="utf-8"))


# ---- the scenario (GOAL Kapsam 1, Q-1)
def test_scenario_sizes_are_the_registered_minimums():
    for name in ("qorbit", "dnotes"):
        h = SCEN["headless"]["per_corpus"][name]
        assert len(h["propose"]) >= 100
        assert sum(w["rule"] for w in h["propose"]) == 50
        assert len(h["direct"]) >= 20
        assert len(h["promote_docs"]) >= 10 and len(h["demote_docs"]) >= 2
        assert len(h["queries"]) >= 40
        langs = {q["lang"] for q in h["queries"]}
        assert {"tr", "en", "mix"} <= langs
    o = SCEN["obsidian"]
    assert len(o["edits"]) >= 20 and len(o["renames"]) >= 2
    assert len(o["layer_ops"]) >= 5 and len(o["queries"]) >= 40
    assert {e["op"] for e in o["edits"]} >= {"create", "append", "replace", "delete"}
    assert len(SCEN["headless"]["checkpoints"]) >= 3
    assert len(SCEN["headless"]["trace_queries"]) >= 5


def test_rule_writes_are_under_the_prefix_and_comments_are_not():
    prefix = SCEN["agent"]["rule_prefix"]
    for h in SCEN["headless"]["per_corpus"].values():
        for w in h["propose"]:
            assert w["doc_path"].startswith(prefix) == w["rule"]
    for r in SCEN["recipes"].values():
        assert 'actor = "agent:toplayici"' in r and 'path_prefix = "toplayici/"' in r
        assert 'unattested = "unverified"' in r and 'human_id = "pilot"' in r


def test_personal_paths_are_hashed_only():
    """Personal corpora appear as blake2b; only D-NOTES lists paths."""
    q = SCEN["headless"]["per_corpus"]["qorbit"]
    assert "append_paths" not in q
    assert all("doc_path" not in d for d in q["direct"] if d["op"] == "append")
    for e in SCEN["obsidian"]["edits"] + SCEN["obsidian"]["renames"]:
        if "doc_path" in e:
            assert e["doc_path"].startswith("pilot/")
    d = SCEN["headless"]["per_corpus"]["dnotes"]
    assert [b2(p) for p in d["append_paths"]] == [
        x["doc_path_b2"] for x in d["direct"] if x["op"] == "append"
    ]


def test_dnotes_snapshot_is_the_repositorys():
    assert (
        corpus_summary(REPO / "research/data/d_notes/notes")
        == SCEN["corpora"]["dnotes"]["snapshot"]
    )


def test_agent_content_is_deterministic_and_synthetic():
    """The templates' text is the generator's: regenerating the D-NOTES
    scene from the seed gives the same writes (no clock, no randomness
    outside the seed)."""
    from p2q import make_scenario as M

    paths = list_md(REPO / "research/data/d_notes/notes")
    h = M.headless(M.SEED, paths, "dnotes")
    reg = SCEN["headless"]["per_corpus"]["dnotes"]
    assert [w["content"] for w in h["propose"]] == [w["content"] for w in reg["propose"]]
    assert [w["content"] for w in h["direct"]] == [w["content"] for w in reg["direct"]]


# ---- the evaluator (Q-6; VERIFY VQ3 (a), (c))
def _hit(doc, layer):
    return {"doc": doc, "layer": layer, "score": 1.0, "chunk_id": "c", "lineage_ids": ["l"]}


def _q(**kw):
    q = {
        "id": "q01",
        "mode": "lexical",
        "match": "exact",
        "targets": ["toplayici/x.md"],
        "expect": {
            "son": {
                "layer": "canonical",
                "default": ["toplayici/x.md"],
                "include_proposal": ["toplayici/x.md"],
            }
        },
    }
    q.update(kw)
    return q


def test_evaluator_passes_the_expected_result():
    got = {
        "default": [_hit("toplayici/x.md", "canonical")],
        "include_proposal": [_hit("toplayici/x.md", "canonical")],
    }
    assert E.check_query(_q(), "son", got) == []


def test_evaluator_names_a_proposal_in_the_default_search():
    got = {
        "default": [_hit("toplayici/x.md", "canonical"), _hit("yorum/y.md", "proposal")],
        "include_proposal": [_hit("toplayici/x.md", "canonical")],
    }
    fails = E.check_query(_q(), "son", got)
    assert any("is proposal" in f for f in fails)


def test_evaluator_names_a_changed_expected_set():
    q = _q()
    q["expect"]["son"]["default"] = ["toplayici/other.md"]
    got = {
        "default": [_hit("toplayici/x.md", "canonical")],
        "include_proposal": [_hit("toplayici/x.md", "canonical")],
    }
    fails = E.check_query(q, "son", got)
    assert len(fails) == 1 and "default docs" in fails[0]


def test_evaluator_contains_and_missing_result():
    q = _q(match="contains", mode="hybrid")
    got = {"default": [_hit("a.md", "canonical")], "include_proposal": [_hit("a.md", "canonical")]}
    assert any("in default hits: False" in f for f in E.check_query(q, "son", got))
    assert E.check_query(q, "son", {}) == ["q01@son: no result recorded"]


def test_direct_write_canonical_counts_only_those_writes():
    blames = {
        "d.md": [
            {"seq_created": 7, "layer": "canonical"},
            {"seq_created": 8, "layer": "unverified"},
        ],
        "h.md": [{"seq_created": 1, "layer": "canonical"}],
    }
    assert E.direct_write_canonical(blames, {7, 8}) == 1
    assert E.direct_write_canonical(blames, {8}) == 0


# ---- the leak check (Q-9; VERIFY VQ3 (b))
def _corpus(tmp: Path) -> Path:
    c = tmp / "corpus"
    c.mkdir()
    (c / "a.md").write_text(
        "# Başlık\n\nBu notun içinde oldukça uzun bir cümle var ve harfler çok.\n", encoding="utf-8"
    )
    (c / "b.md").write_text("short\n", encoding="utf-8")
    (c / "sub").mkdir()
    (c / "sub" / "c.md").write_text(
        "Another fairly long English line that has many letters.\n", encoding="utf-8"
    )
    return c


def test_leakcheck_finds_a_piece_and_reports_no_text(tmp_path):
    c = _corpus(tmp_path)
    ps, n = L.pieces(c)
    assert n == 3 and len(ps) == 2
    assert all(len(p) <= L.PIECE and b"\n" not in p for p in ps)
    rec = tmp_path / "records"
    rec.mkdir()
    (rec / "clean.json").write_text('{"n": 1}\n')
    assert L.run([c], None, "HEAD", None, [rec])["found"] == 0
    (rec / "leak.md").write_bytes(b"prefix " + ps[0] + b" suffix\n")
    rep = L.run([c], None, "HEAD", None, [rec])
    assert rep["found"] == 1 and rep["pass"] is False
    out = json.dumps(rep).encode()
    assert all(p not in out for p in ps)


def test_leakcheck_piece_is_seeded():
    data = "x" * 10 + " ".join(["kelime"] * 20) + "\n"
    a = L.piece_of(data.encode(), random.Random("s"))
    b = L.piece_of(data.encode(), random.Random("s"))
    assert a == b and a is not None
