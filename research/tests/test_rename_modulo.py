"""Comparisons modulo the project rename (K-GH.1, GH-0/F3): `renamed` maps the
pre-rename names, keeps the data identifiers, and is the identity on text
without them; `resolve` finds a pre-rename RUN's file under its new name."""

from __future__ import annotations

import json
from pathlib import Path

from eval import compare_runs as C


def test_renamed_maps_the_old_names():
    assert C.renamed("MDK_FIXTURES") == "ROOTMARK_FIXTURES"
    assert C.renamed("mdk 0.1.0") == "rootmark 0.1.0"
    assert C.renamed("mdkernel-obsidian/0.1.0") == "rootmark-obsidian/0.1.0"
    assert C.renamed("vault/.mdkernel/events.jsonl") == "vault/.rootmark/events.jsonl"
    assert C.renamed("target/release/mdk") == "target/release/rootmark"


def test_renamed_keeps_identifiers_and_other_text():
    for s in ["mdk_version", "mdk_lower_alnum", "rootmark", "gSIdMdKsJo", "abcmdk", "", "3fdac071"]:
        assert C.renamed(s) == s


def test_leaves_compare_old_and_new_names_equal():
    old = {"MDK_X": {"server": "mdk", "v": 1}, "mdk_version": "0.1.0"}
    new = {"ROOTMARK_X": {"server": "rootmark", "v": 1}, "mdk_version": "0.1.0"}
    assert C.leaves(old) == C.leaves(new)
    # a real difference stays a difference
    assert C.leaves(old) != C.leaves({**new, "mdk_version": "0.2.0"})


def test_resolve_finds_a_pre_rename_path(tmp_path: Path):
    run = tmp_path / "run"
    (run / "p1e/release").mkdir(parents=True)
    (run / "p1e/release/conformance_mdk.json").write_text(json.dumps({"passed": 34}))
    (run / "vault/.mdkernel").mkdir(parents=True)
    (run / "vault/.mdkernel/events.jsonl").write_text("x\n")
    assert C._load(run, "p1e/release/conformance_rootmark.json") == {"passed": 34}
    assert C.resolve(run, "vault/.rootmark/events.jsonl").read_text() == "x\n"
    missing = C.resolve(run, "p1e/release/absent.json")
    assert missing == run / "p1e/release/absent.json" and not missing.exists()


def test_keys_equal_after_the_rename_are_an_error():
    import pytest

    from eval import historical
    from eval import metrics as M

    with pytest.raises(M.MetricError):
        C.leaves({"mdk_x": 1, "rootmark_x": 2})
    with pytest.raises(ValueError):
        list(historical.leaves({"MDK_A": 1, "ROOTMARK_A": 1}))
    assert C.renamed("mdkernel-index/3") == "mdkernel-index/3"
