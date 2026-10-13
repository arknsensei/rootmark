"""Rust core behaviour re-checked through the Python binding (5 cases) plus
Rust/Python agreement on events hash and token counts."""

import json
import random
import subprocess
from pathlib import Path

import mdkernel_core as mk
import pytest
import tiktoken

from eval.datasets.common import SEED

ROOT = Path(__file__).resolve().parents[1]
MINI = ROOT.parent / "spec" / "fixtures" / "mini_vault"
MDK = ROOT.parent / "target" / "release" / "mdk"

P1 = "The roaster controls heat, airflow and time for every batch of green coffee."
P2 = "Cupping is the standard tasting protocol that we use to evaluate each lot."
P3 = "Grinders are calibrated every morning before the first espresso is pulled."


def doc(*ps):
    return "\n\n".join(ps) + "\n"


def kinds(r):
    return [e["kind"] for e in r["events"] if e["kind"] != "SnapshotCommitted"]


def test_same_text_same_version_id_and_parse():
    a, fm = mk.parse_document("---\nt: 1\n---\n\nHello   world\nagain\n", "a.md")
    b, _ = mk.parse_document("Hello world again", "b.md")
    assert fm == "t: 1\n"
    assert (
        a[0]["version_id"] == b[0]["version_id"] == mk.version_id("Paragraph", "Hello world again")
    )
    assert mk.normalize("café  x") == "café x"
    assert mk.extract_wikilinks("[[a/b.md#H|l]]") == [{"target": "b", "heading": "H"}]


def test_identical_reingest_zero_events_and_replay(tmp_path):
    v = mk.Vault(str(tmp_path))
    v.ingest("d.md", doc(P1, P2))
    r = v.ingest("d.md", doc(P1, P2))
    assert r["events"] == []
    assert v.replay("d.md") == v.snapshot("d.md")


def test_move_and_update(tmp_path):
    v = mk.Vault(str(tmp_path))
    v.ingest("d.md", doc(P1, P2, P3))
    before = [l for l, _ in v.snapshot("d.md")["blocks"]]
    # P3 jumps ahead of P1 (exact match outside the LIS), P2 is edited
    r = v.ingest("d.md", doc(P3, P1, P2.replace("standard", "usual")))
    assert sorted(kinds(r)) == ["BlockMoved", "BlockUpdated"]
    assert sorted(l for l, _ in v.snapshot("d.md")["blocks"]) == sorted(before)


def test_lineage_formula_and_resolve_truth_err(tmp_path):
    v = mk.Vault(str(tmp_path))
    r = v.ingest("d.md", doc(P1, "dup", "dup"))
    blocks = r["snapshot"]["blocks"]
    assert blocks[2][0] == mk.lineage_formula("d.md", blocks[2][1], 1)
    assert v.resolve_truth("d.md", blocks[2][1], 1) == blocks[2][0]
    with pytest.raises(RuntimeError):
        v.resolve_truth("d.md", blocks[2][1], 2)
    with pytest.raises(RuntimeError):
        v.resolve_truth("nope.md", blocks[2][1], 0)


def test_split_detected_and_params(tmp_path):
    long = (
        "Roasting turns green coffee into the brown beans we grind every day. "
        "The roaster controls heat, airflow and time, and every batch is logged carefully."
    )
    a, b = long.split(" The roaster")
    v = mk.Vault(str(tmp_path), theta=0.55, theta_split=0.6)
    assert v.params() == (0.55, 0.6)
    v.ingest("d.md", doc(P1, long))
    r = v.ingest("d.md", doc(P1, a, "The roaster" + b))
    assert kinds(r) == ["BlockSplit"]
    assert mk.matcher_defaults() == mk.Vault(str(tmp_path / "x")).params()


def test_python_and_cli_events_hash_equal(tmp_path):
    v = mk.Vault(str(tmp_path / "py"))
    v.ingest_dir(str(MINI))
    out = subprocess.run(
        [str(MDK), "ingest-dir", str(tmp_path / "cli"), str(MINI)],
        capture_output=True,
        text=True,
        check=True,
    )
    assert json.loads(out.stdout)["events_hash"] == v.events_hash()


def test_tokenizer_equality_20_random_chunks(tmp_path):
    enc = tiktoken.get_encoding("cl100k_base")
    v = mk.Vault(str(tmp_path))
    v.ingest_dir(str(MINI))
    chunks = [c for d in v.doc_paths() for s in ("structural", "fixed") for c in v.chunk(d, s)]
    rng = random.Random(SEED)
    sample = rng.sample(chunks, 20)
    for c in sample:
        assert len(enc.encode_ordinary(c["text"])) == c["token_count"] == mk.count_tokens(c["text"])
    # also odd text: unicode, code, whitespace runs
    for t in ["héllo wörld 🚀", "```py\n  x  =  1\n```", "a\n\n\n\tb", "<|endoftext|> literal"]:
        assert len(enc.encode_ordinary(t)) == mk.count_tokens(t)
