"""D1-easy (K4) and D1-hard generators (T22): byte equality with D1, op
rates, edited split/merge instances, disjoint document sets, content-only truth.

Data tests run on the built `data/d1k4`, `data/d1h` (verified against their
manifests by `make d1k4 d1h`); unit tests run on a small fixture document.
"""

import filecmp
import json
import random
from collections import Counter
from pathlib import Path

import mdkernel_core as mk
import pytest

from eval import truth as T
from eval.datasets import synthetic_edits as se
from eval.datasets import synthetic_edits_v2 as se2
from eval.datasets.common import DATA

ROOT = Path(__file__).resolve().parents[1]
FIX = ROOT / "tests" / "fixtures" / "mini_vault"
D1, D1K4, D1H = DATA / "d1", DATA / "d1k4", DATA / "d1h"
needs = pytest.mark.skipif(
    not (D1K4 / "docs.json").exists() or not (D1H / "docs.json").exists(),
    reason="data/d1k4 or data/d1h not built (make d1k4 d1h)",
)


def _norm(text: str) -> str:
    (d,), _ = mk.parse_document(text + "\n", "x.md")
    return d["normalized_text"]


def _blocks(doc_dir: Path, name: str):
    return se.parse_blocks((doc_dir / name).read_text(encoding="utf-8"))[0]


def _at(blocks, ref):
    return [b for b in blocks if b.version_id == ref["version_id"]][ref["k"]]


def _walk_refs(obj):
    if isinstance(obj, dict) and "version_id" in obj:
        yield obj
    elif isinstance(obj, dict):
        for v in obj.values():
            yield from _walk_refs(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from _walk_refs(v)


# ---------------------------------------------------------------- unit


def test_hard_generator_on_fixture_is_deterministic(tmp_path):
    text = (FIX / "roasting.md").read_text(encoding="utf-8")
    pool = se.insert_pool({p.name: p.read_text() for p in FIX.glob("*.md")}, {"roasting.md"})
    a = se2.generate_hard_doc("roasting.md", text, pool, tmp_path / "a", n_steps=6)
    b = se2.generate_hard_doc("roasting.md", text, pool, tmp_path / "b", n_steps=6)
    assert a == b
    for f in sorted((tmp_path / "a").iterdir()):
        assert f.read_bytes() == (tmp_path / "b" / f.name).read_bytes()


def test_dominant_is_longest_first_on_tie():
    a = se.make_block("Paragraph", "Short one here.")
    b = se.make_block("Paragraph", "A much longer paragraph text here.")
    c = se.make_block("Paragraph", "Short two here.")
    assert se2.longest([a, b, c], [0, 1, 2]) == 1
    assert se2.longest([a, c], [0, 1]) == 0  # equal length → first


def test_dup_delete_adjacent_mapping():
    note = se.make_block("Paragraph", se.NOTES[0])
    x = se.make_block("Paragraph", "Some other paragraph text.")
    doc = se.Doc([x, note, note, note, x], None)
    new, pairs = se2.op_dup_delete_adjacent(doc, [1, 2, 3])
    assert len(new) == 4
    assert pairs == [(1, [1]), (3, [2]), (2, [])]


# ---------------------------------------------------------------- data


@needs
def test_d1k4_texts_byte_equal_to_d1():
    d1_files = sorted(p.relative_to(D1).as_posix() for p in D1.rglob("v*.md"))
    k4_files = sorted(p.relative_to(D1K4).as_posix() for p in D1K4.rglob("v*.md"))
    assert d1_files == k4_files and d1_files
    _match, mismatch, errors = filecmp.cmpfiles(D1, D1K4, d1_files, shallow=False)
    assert not mismatch and not errors
    for name in ("docs.json", "splits.json"):
        assert (D1 / name).read_bytes() == (D1K4 / name).read_bytes()


@needs
def test_d1k4_truth_is_v2_truth_plus_dominant():
    for doc_dir in sorted((D1 / "docs").iterdir()):
        t2 = json.loads((doc_dir / "truth.json").read_text())
        tk = json.loads((D1K4 / "docs" / doc_dir.name / "truth.json").read_text())
        assert len(t2["steps"]) == len(tk["steps"])
        for s2, sk in zip(t2["steps"], tk["steps"], strict=True):
            extra = set(sk) - set(s2)
            if s2["op"] in ("split", "merge"):
                assert extra == {"dominant"}
            else:
                assert not extra
            assert {k: sk[k] for k in s2} == s2


def _set_docs(d):
    return {x["source"] for x in json.loads((d / "docs.json").read_text())}


@needs
def test_easy_and_hard_documents_disjoint():
    easy, hard = _set_docs(D1K4), _set_docs(D1H)
    assert len(easy) == len(hard) == 200
    assert not easy & hard
    s = json.loads((D1H / "splits.json").read_text())
    assert len(s["tune"]) == 50 and len(s["test"]) == 150
    assert not set(s["tune"]) & set(s["test"])


@needs
def test_hard_op_rates_within_3_points_over_1000_steps():
    ops = []
    for doc_dir in sorted((D1H / "docs").iterdir()):
        t = json.loads((doc_dir / "truth.json").read_text())
        ops += [s["op"] for s in t["steps"] if s["scored"]]
    assert len(ops) == 2000
    rng = random.Random(0)
    for sample in (ops[:1000], ops[1000:], rng.sample(ops, 1000)):
        c = Counter(sample)
        for op, p in se2.OPS_HARD:
            assert abs(c[op] / 1000 - p) <= 0.03, (op, c[op], p)


@needs
def test_easy_op_rates_within_3_points_over_1000_steps():
    ops = []
    for doc_dir in sorted((D1K4 / "docs").iterdir()):
        t = json.loads((doc_dir / "truth.json").read_text())
        ops += [s["op"] for s in t["steps"] if s["scored"]]
    for sample in (ops[:1000], ops[1000:]):
        c = Counter(sample)
        for op, p in se.OPS:
            assert abs(c[op] / 1000 - p) <= 0.03, (op, c[op], p)


@needs
def test_hard_split_merge_are_never_exact_recombinations():
    n = 0
    for doc_dir in sorted((D1H / "docs").iterdir()):
        t = json.loads((doc_dir / "truth.json").read_text())
        prev = t["base_file"]
        for s in t["steps"]:
            if s["op"] in se2.SPLIT_OPS + se2.MERGE_OPS:
                old, new = _blocks(doc_dir, prev), _blocks(doc_dir, s["file"])
                if s["op"] in se2.SPLIT_OPS:
                    ((o, ns),) = s["mapping"]
                    whole = _norm(_at(old, o).text)
                    parts = " ".join(_norm(_at(new, r).text) for r in ns)
                    assert len(ns) == (3 if s["op"] == "split_three" else 2)
                else:
                    whole = " ".join(_norm(_at(old, o).text) for o, _ in s["mapping"])
                    ((_, (r,)),) = s["mapping"][:1]
                    parts = _norm(_at(new, r).text)
                # normalized Levenshtein is 1.0 iff the strings are equal, so sim < 1.0 ⇔ ≠
                assert whole != parts, (doc_dir.name, s["step"])
                n += 1
            prev = s["file"]
    assert n > 0


@needs
def test_truth_is_content_only_and_dominant_is_longest():
    for root in (D1K4, D1H):
        for doc_dir in sorted((root / "docs").iterdir()):
            t = json.loads((doc_dir / "truth.json").read_text())
            assert "lineage" not in json.dumps(t)
            for r in _walk_refs(t["steps"]):
                assert set(r) == {"version_id", "k"}
                T.check_ref(r)
            prev = t["base_file"]
            for s in t["steps"]:
                if "dominant" in s:
                    old, new = _blocks(doc_dir, prev), _blocks(doc_dir, s["file"])
                    if s["op"] in se2.SPLIT_OPS:
                        cands = [_at(new, r) for r in s["mapping"][0][1]]
                        dom = _at(new, s["dominant"])
                    else:
                        cands = [_at(old, o) for o, _ in s["mapping"]]
                        dom = _at(old, s["dominant"])
                    lens = [len(_norm(b.text)) for b in cands]
                    assert len(_norm(dom.text)) == max(lens)
                    assert cands.index(dom) == lens.index(max(lens))
                prev = s["file"]
