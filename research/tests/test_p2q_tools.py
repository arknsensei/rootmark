"""P2-Q (TASKS Q0, Q2 (e)): the pre-registered scenario, the query evaluator,
the content leak check and the observation script, on their own small
fixtures (no personal corpus is read here)."""

import json
import random
import re
import struct
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


def test_personal_paths_are_keyed_only():
    """Personal paths appear only as keyed blake2b (k2); D-NOTES in the clear."""
    q = SCEN["headless"]["per_corpus"]["qorbit"]
    for d in q["direct"]:
        assert ("doc_path" in d) != ("doc_ref" in d)
        if "doc_ref" in d:
            assert re.fullmatch(r"k2:[0-9a-f]{32}", d["doc_ref"])
    for e in SCEN["obsidian"]["edits"] + SCEN["obsidian"]["renames"]:
        if "doc_path" in e:
            assert e["doc_path"].startswith("pilot/")
        else:
            assert re.fullmatch(r"k2:[0-9a-f]{32}", e["doc_ref"])
    d = SCEN["headless"]["per_corpus"]["dnotes"]
    known = set(list_md(REPO / "research/data/d_notes/notes"))
    assert all(x["doc_ref"] in known for x in d["direct"] if x["op"] == "append")


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
    h = M.headless(M.SEED, paths, "dnotes", lambda p: p)
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


def test_keyed_path_hash_needs_the_key():
    from p2q.common import k2

    a, b = k2("README.md", b"\x01" * 32), k2("README.md", b"\x02" * 32)
    assert a != b and a.startswith("k2:") and b2("README.md") not in a


def test_leakcheck_piece_is_seeded():
    data = "x" * 10 + " ".join(["kelime"] * 20) + "\n"
    a = L.piece_of(data.encode(), random.Random("s"))
    b = L.piece_of(data.encode(), random.Random("s"))
    assert a == b and a is not None


# ---- the observation script (Q-11; TASKS Q2 (e))
def _ev(seq, kind, doc, lids, meta):
    return {
        "seq": seq,
        "doc_path": doc,
        "kind": kind,
        "lineage_ids": lids,
        "old_version_ids": [],
        "new_version_ids": [],
        "snapshot_id": None,
        "meta": meta,
    }


def _obs_vault(tmp: Path) -> Path:
    v = tmp / "vault"
    (v / ".rootmark").mkdir(parents=True)
    (v / "gizli-not.md").write_text("# Gizli başlık\n\nözel metin\n", encoding="utf-8")
    human = {"provenance": {"author_kind": "Human", "source": ""}}
    agent = {"provenance": {"author_kind": "Agent", "source": "toplayici/1.0"}}
    unatt = {
        "layer": "unverified",
        "provenance": {"author_kind": "Human", "source": "unattested_fs"},
    }
    evs = [
        _ev(0, "BlockCreated", "gizli-not.md", ["h1"], human),
        _ev(1, "BlockCreated", "toplayici/a.md", ["a1"], agent),
        _ev(
            2,
            "Promoted",
            "toplayici/a.md",
            ["a1"],
            {"actor": {"kind": "policy", "id": "toplayici"}, "from": "proposal", "to": "canonical"},
        ),
        _ev(3, "BlockCreated", "yorum/b.md", ["b1"], agent),
        _ev(4, "BlockCreated", "yorum/b.md", ["b2"], agent),
        _ev(
            5,
            "Promoted",
            "yorum/b.md",
            ["b1"],
            {"actor": {"kind": "human", "id": "pilot"}, "from": "proposal", "to": "canonical"},
        ),
        _ev(6, "CheckpointCreated", "", [], {"checkpoint_id": "c", "event_seq": 6, "name": None}),
        _ev(7, "BlockCreated", "d.md", ["d1"], unatt),
        _ev(
            8,
            "Demoted",
            "toplayici/a.md",
            ["a1"],
            {"actor": {"kind": "human", "id": "pilot"}, "from": "canonical", "to": "proposal"},
        ),
        _ev(
            9,
            "Demoted",
            "yorum/b.md",
            ["b2"],
            {"actor": {"kind": "policy", "id": "stale"}, "prune": True, "to": "proposal"},
        ),
        _ev(10, "CheckpointCreated", "", [], {"checkpoint_id": "d", "event_seq": 10, "name": "x"}),
        _ev(11, "CheckpointNamed", "", [], {"checkpoint_id": "d", "name": "x"}),
    ]
    (v / ".rootmark/events.jsonl").write_text(
        "".join(json.dumps(e) + "\n" for e in evs) + '{"seq": 12, "partial', encoding="utf-8"
    )
    tr = [
        {
            "trace_id": "t1",
            "consumer": "brif",
            "query_hash": "q",
            "at_seq": 5,
            "returned": [{"doc_path": "gizli-not.md", "lineage_id": "h1", "version_id": "v"}],
        },
        {
            "trace_id": "t2",
            "consumer": "brif",
            "query_hash": None,
            "at_seq": 9,
            "returned": [
                {"doc_path": "gizli-not.md", "lineage_id": "h1", "version_id": "v"},
                {"doc_path": "yorum/b.md", "lineage_id": "b1", "version_id": "w"},
            ],
        },
    ]
    (v / ".rootmark/traces.jsonl").write_text("".join(json.dumps(t) + "\n" for t in tr))
    return v


def test_observe_counts_and_layers(tmp_path):
    from p2q import observe as O

    v = _obs_vault(tmp_path)
    log = tmp_path / "attest.log"
    log.write_text(
        "\n".join(
            json.dumps(x)
            for x in [
                {
                    "event": "attest",
                    "kind": "plain",
                    "result": "accepted",
                    "doc_path": "gizli-not.md",
                },
                {
                    "event": "attest",
                    "kind": "rename",
                    "result": "refused",
                    "code": "denied",
                    "reason": "gizli-not.md: bytes differ",
                    "doc_path": "gizli-not.md",
                },
                {"event": "observe", "ingest": "attested", "doc_path": "gizli-not.md"},
                {
                    "event": "observe",
                    "ingest": "unverified",
                    "doc_path": "d.md",
                    "accepted_before": [{"why": "expired"}],
                },
            ]
        )
        + "\n"
    )
    s = O.observe(v, log, None, 0)
    assert s["head_seq"] == 12  # the partial last line is left out
    assert s["events"]["by_kind"]["BlockCreated"] == 5
    assert s["writes"]["events_by_writer"] == {"agent": 3, "human": 1, "unattested_fs": 1}
    assert s["writes"]["agent_events_by_client"] == {"toplayici": 3}
    assert s["promotions"]["lineages_by_actor"] == {"human": 1, "policy": 1}
    assert s["promotions"]["lineages_by_rule"] == {"toplayici": 1}
    assert s["demotions"]["lineages_by_actor"] == {"human": 1, "pruned": 1}
    assert s["layers"]["lineages"] == {
        "canonical": 2,
        "inactive": 1,
        "proposal": 1,
        "unverified": 1,
    }
    assert s["layers"]["unverified_at_each_checkpoint"] == [0, 1]
    assert s["checkpoints"] == {"created": 2, "named": 1}
    assert s["traces"]["searches"] == 1 and s["traces"]["reads"] == 1
    assert s["traces"]["most_relied_on"][0] == {"lineage_b2": b2("h1"), "count": 2}
    assert s["attest"]["refusal_classes"] == {"rename:denied": 1}
    assert s["attest"]["observations"]["unverified_because:expired"] == 1
    p = O.observe(v, None, None, 7)
    assert p["events"]["total"] == 5 and p["traces"]["total"] == 1


def test_observe_is_read_only_and_prints_no_content(tmp_path, capsys):
    from p2q import observe as O
    from p2q.common import tree_b2

    v = _obs_vault(tmp_path)
    before = tree_b2(v)
    assert O.main(["--vault", str(v)]) == 0
    out = capsys.readouterr().out
    assert tree_b2(v) == before
    for secret in ("gizli", "özel", "toplayici/a.md", "yorum/b.md", "d.md"):
        assert secret not in out
    try:
        O.main(["--vault", str(v), "--out", str(v / "x.json")])
    except SystemExit as e:
        assert "outside the vault" in str(e)
    else:
        raise AssertionError("--out inside the vault was accepted")


# ---- the comparison (Q-6, Q-8)
def _run(tmp: Path, name: str, score: float, chunk: str = "c1", eh: str = "e") -> Path:
    d = tmp / name
    d.mkdir()
    (d / "hashes.json").write_text(
        json.dumps(
            {
                "events_hash": eh,
                "index_files": {"vectors/vectors.f32": "v", "manifest.json": "m"},
                "vectors": "v",
                "manifest": "m",
            }
        )
    )
    hit = {"doc": "a.md", "chunk_id": chunk, "layer": "canonical", "score": score}
    (d / "queries.json").write_text(
        json.dumps({"son": {"q01": {"default": [hit], "include_proposal": [hit]}}})
    )
    return d


def test_compare_names_last_ulp_score_differences_and_ranking_changes(tmp_path):
    from p2q import compare as C

    assert C.ulp_distance(1.0, 1.0) == 0
    nxt = struct.unpack(
        "<d", struct.pack("<q", struct.unpack("<q", struct.pack("<d", 1.0))[0] + 1)
    )[0]
    assert C.ulp_distance(1.0, nxt) == 1
    a = _run(tmp_path, "mac", 1.0)
    b = _run(tmp_path, "docker", nxt)
    assert C.main([str(a), str(b)]) == 0
    r = C.compare(C.load(a), C.load(b))
    assert r["max_ulp"] == 1 and len(r["scores_differing"]) == 2 and r["rankings_equal"]
    c = _run(tmp_path, "ci", 1.0, chunk="c2")
    assert C.main([str(a), str(c)]) == 1
    e = _run(tmp_path, "other", 1.0, eh="x")
    assert C.compare(C.load(a), C.load(e))["byte_equal"] is False


# ---- the Turkish pre-registration (K-P2Q.3)
def test_turkish_scenario_and_folding_rule():
    from p2q import make_scenario_tr as T
    from p2q.tr_queries import body, fold_tokens

    reg = json.loads((REPO / "research/p2q/scenario_tr.json").read_text(encoding="utf-8"))
    assert reg == json.loads(json.dumps(T.build()))
    assert len(reg["queries"]) >= 30
    assert {q["group"] for q in reg["queries"]} >= {
        "fold",
        "ascii_i",
        "accentless",
        "mixed",
        "semantic",
    }
    assert fold_tokens("İSTANBUL") == ["istanbul"]
    assert fold_tokens("ISTANBUL") == ["ıstanbul"]
    assert fold_tokens("IŞIK ışık Işık") == ["ışık"] * 3
    assert fold_tokens("Şirket-değişikliği 2026") == ["şirket", "değişikliği", "2026"]
    assert fold_tokens("sirket") != fold_tokens("şirket")
    assert body("---\ndate: x\n---\nmetin") == "\nmetin"
