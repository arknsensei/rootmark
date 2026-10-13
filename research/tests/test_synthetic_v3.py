"""P1-A sets (eval/datasets/synthetic_edits_v3.py) and their runners:
split_hard ops edit every piece, short pieces ≤ 8 words, heavy merges change
≥ 15 % of the words; generation is deterministic; the runners fail fast on a
manifest mismatch or overlapping sets."""

import json
import random
from pathlib import Path

import pytest

from eval import run_holdout, run_split_grid
from eval.datasets import synthetic_edits as se
from eval.datasets import synthetic_edits_v2 as s2
from eval.datasets import synthetic_edits_v3 as s3

MINI = Path(__file__).resolve().parents[2] / "spec" / "fixtures" / "mini_vault"
LONG = (
    "Roasting turns green coffee into brown beans every single morning. "
    "The roaster controls heat, airflow and drum speed for every batch of green coffee today. "
    "Short note here."
)
OTHER = "Cupping follows within one day of roasting and every lot is scored by two tasters."
VOCAB = sorted({w.strip(".,") for w in (LONG + " " + OTHER).split() if len(w.strip(".,")) >= 4})


def _doc(*paras: str) -> se.Doc:
    blocks = [se.make_block("Heading", "# Page")] + [se.make_block("Paragraph", p) for p in paras]
    return se.Doc(blocks, None)


@pytest.mark.parametrize("short", [False, True])
def test_split_edits_both_pieces_and_none_is_a_substring(short):
    hits = 0
    for seed in range(40):
        doc = _doc(LONG, OTHER)
        res = s3.op_split_both_edited(doc, random.Random(seed), VOCAB, short=short)
        if res is None:
            continue
        hits += 1
        new, [(i, js)] = res
        old_norm = s3.norm_text(doc.blocks[i])
        pieces = [new[j] for j in js]
        assert len(pieces) == 2
        assert all(s3.norm_text(p) not in old_norm for p in pieces)
        if short:
            assert min(len(se.words_of(p.text)) for p in pieces) <= s3.SHORT_MAX_WORDS
    assert hits > 0


def test_short_piece_needs_a_short_sentence():
    no_short = (
        "Roasting turns green coffee into brown beans every single morning here. "
        "The roaster controls heat, airflow and drum speed for every batch today."
    )
    for seed in range(20):
        assert s3.op_split_both_edited(_doc(no_short), random.Random(seed), VOCAB, True) is None


def test_heavy_merge_changes_at_least_15_percent():
    hits = 0
    for seed in range(40):
        doc = _doc(LONG, OTHER)
        res = s3.op_merge_heavy(doc, random.Random(seed), VOCAB)
        if res is None:
            continue
        hits += 1
        new, pairs = res
        assert pairs == [(1, [1]), (2, [1])]
        merged = doc.blocks[1].text.rstrip() + " " + doc.blocks[2].text.lstrip()
        before, after = merged.split(" "), new[1].text.split(" ")
        changed = sum(a != b for a, b in zip(before, after, strict=True))
        assert changed / len(before) >= s3.MERGE_MIN_EDIT
    assert hits > 0


def test_generation_is_deterministic_and_k4(tmp_path):
    text = (MINI / "roasting.md").read_text(encoding="utf-8")
    texts = {p.name: p.read_text(encoding="utf-8") for p in sorted(MINI.glob("*.md"))}
    pool = se.insert_pool(texts, {"roasting.md"})
    for name, table, steps in (
        ("ho", s2.OPS_HARD, 6),
        ("sh", s3.OPS_SPLIT_HARD, 6),
    ):
        a, b = tmp_path / f"{name}a", tmp_path / f"{name}b"
        seed = s3.HOLDOUT_SEED if name == "ho" else s3.SPLIT_HARD_SEED
        ta = s3.generate("roasting.md", text, pool, a, seed, table, steps)
        s3.generate("roasting.md", text, pool, b, seed, table, steps)
        assert {p.name: p.read_bytes() for p in a.iterdir()} == {
            p.name: p.read_bytes() for p in b.iterdir()
        }
        for s in ta["steps"]:
            if s["op"] in s2.SPLIT_OPS + s2.MERGE_OPS:
                assert "dominant" in s
            if name == "sh" and s["scored"] and s["op"] != "dup_delete_adjacent":
                assert s["variant"] in ("both_edited", "short_piece", "heavy")


def test_holdout_eligibility():
    assert s3.holdout_eligible("# T\n\n" + LONG + "\n\n- item\n")
    assert not s3.holdout_eligible("# T\n\n- only a list\n")


def test_disjointness_fails_on_overlap(monkeypatch):
    monkeypatch.setattr(s2, "easy_sources", lambda: ["a.md", "b.md"])
    monkeypatch.setattr(s3, "hard_sources", lambda: ["c.md"])
    monkeypatch.setattr(s3, "holdout_sources", lambda: ["b.md"])
    with pytest.raises(RuntimeError, match="not disjoint"):
        run_holdout.disjointness()


def _fake_set(tmp_path, monkeypatch, attr_dir, attr_manifest):
    tmp_path = tmp_path / attr_dir
    root = tmp_path / "set"
    (root / "docs" / "000").mkdir(parents=True)
    (root / "docs" / "000" / "truth.json").write_text("{}")
    manifest = tmp_path / "MANIFEST.json"
    manifest.write_text(json.dumps({"files": {"docs/000/truth.json": "0" * 32}}))
    monkeypatch.setattr(s3, attr_dir, root)
    monkeypatch.setattr(s3, attr_manifest, manifest)


def test_runners_fail_fast_on_manifest_mismatch(tmp_path, monkeypatch):
    _fake_set(tmp_path, monkeypatch, "HOLDOUT_DIR", "MANIFEST_HOLDOUT")
    with pytest.raises(RuntimeError, match="does not match MANIFEST.d1ho.json"):
        run_holdout.verify_holdout()
    _fake_set(tmp_path, monkeypatch, "SPLIT_HARD_DIR", "MANIFEST_SPLIT_HARD")
    with pytest.raises(RuntimeError, match="does not match MANIFEST.d1sh.json"):
        run_split_grid.verify_split_hard()


def test_split_grid_never_writes_a_constant():
    src = Path(run_split_grid.__file__).read_text(encoding="utf-8")
    assert "lineage.rs" not in src and "apply_constants" not in src
    assert run_split_grid.THETA_SPLITS == [0.45, 0.55, 0.65, 0.75, 0.85]
