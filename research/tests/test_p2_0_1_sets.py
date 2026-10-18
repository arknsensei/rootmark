"""P2-0.1/Y3: the P2-0.1 generators are deterministic (the same seed gives
the same sets), have the pre-registered sizes and kinds (ASSUMPTIONS
Y0-4..Y0-10), and D-A2's paths write exactly the intended text."""

from __future__ import annotations

import collections
import json
import shutil
from pathlib import Path

import blake3

from eval import agent_fs_paths
from eval.datasets import p2_0_1_sets as s
from eval.datasets import p2_0_sets as s0


def test_same_seed_same_sets():
    for f in [s.d_b_test2, s.d_a_prime, s.d_b2, s.d_a2, s.d_c_prime]:
        assert json.dumps(f(), sort_keys=True) == json.dumps(f(), sort_keys=True)


# the sets as committed with Y3 (before any measurement): a generator change
# changes one of these digests
DIGESTS = {
    "d_b_test2": "825bbb7c314b4166aa4792116d8761be",
    "d_a_prime": "0084fedd55c4c6b944b377d64b5f9607",
    "d_b2": "78492efebb049ca6daa9713e6c3a5e47",
    "d_a2": "ff79ff0f9306e86e32c3d0d413edc6da",
    "d_c_prime": "5e887ea904c7d9cb77933f34922b59f3",
}


def test_sets_equal_their_committed_digests():
    for name, want in DIGESTS.items():
        got = blake3.blake3(json.dumps(getattr(s, name)(), sort_keys=True).encode()).hexdigest()
        assert got[:32] == want, name


def test_d_b_test2_seed_and_sizes():
    assert s.d_b_seed() == int(blake3.blake3(b"P2-0.1/D-B").hexdigest()[:8], 16) == 2237993741
    t = s.d_b_test2()
    assert len(t["sessions"]) == 200
    # P2-0's generator with the new seed (P2-0's own test set, seed
    # 20261018, is never generated here)
    assert t["sessions"] == s0.d_b(2237993741)[1]["sessions"]
    assert t["seed"] != s0.SEED
    assert set(collections.Counter(x["kind"] for x in t["sessions"])) == set(s0.D_B_KINDS)


def test_pre_registered_sizes():
    a = s.d_a_prime()
    assert collections.Counter(x["kind"] for x in a["scenarios"]) == {
        "in_agent_block": 25,
        "in_human_block": 15,
        "unobserved": 10,
    }
    assert [f["variant"] for f in a["fakes"]] == s.FAKE_VARIANTS and len(a["fakes"]) == 10
    assert all(
        st["delay_ms"] in (0, 10, 25, 50)
        for x in a["scenarios"]
        if x["kind"] == "unobserved"
        for st in x["steps"]
        if st["op"] == "rename"
    )
    b = s.d_b2()
    assert collections.Counter(x["kind"] for x in b["sessions"]) == {
        "atomic": 50,
        "inplace": 30,
        "append": 20,
        "mv": 30,
        "rapid": 20,
    }
    assert all(3 <= len(x["saves"]) <= 4 for x in b["sessions"] if x["kind"] == "rapid")
    c = s.d_a2()
    paths = collections.Counter(x["kind"].split("/")[0] for x in c["scenarios"])
    assert paths == {p: 20 for p in agent_fs_paths.PATHS}
    kinds = collections.Counter(x["kind"] for x in c["scenarios"])
    assert all(n == 5 for n in kinds.values()) and len(kinds) == 24
    d = s.d_c_prime()
    assert collections.Counter(t["agent_delay_ms"] for t in d["ticks"]) == {
        0: 10,
        20: 10,
        50: 10,
        100: 10,
    }


def test_d_a2_paths_write_the_intended_text(tmp_path: Path):
    v = tmp_path / "vault"
    shutil.copytree(s.MINI, v, ignore=shutil.ignore_patterns(".rootmark"))
    for sc in s.d_a2()["scenarios"]:
        for st in sc["steps"]:
            agent_fs_paths.apply_step(v, st, tmp_path / "work")
            assert (v / st["path"]).read_text(encoding="utf-8").rstrip("\n") == st["text"].rstrip(
                "\n"
            )


def test_d_a2_refuses_paths_outside_the_documents(tmp_path: Path):
    for bad in ["../x.md", ".rootmark/x.md", "/abs.md"]:
        try:
            agent_fs_paths.apply_step(
                tmp_path, {"op": "python", "path": bad, "text": "x", "kind": "new_file"}, tmp_path
            )
        except ValueError:
            continue
        raise AssertionError(f"{bad} accepted")


def test_run_real_needs_the_harness_not_a_directory(tmp_path: Path, monkeypatch):
    """Y3 review 1: the real-Obsidian runner makes its work directory and
    stops only on what is really missing (the harness bundle)."""
    import pytest

    from eval import run_p2_0_1 as r

    fake_obsidian = tmp_path / "Obsidian"
    fake_obsidian.write_text("")
    monkeypatch.setattr(r, "REPO", tmp_path)
    with pytest.raises(FileNotFoundError, match="make p2-0-1-harness"):
        r.run_real(
            "d_c_prime",
            "rootmark",
            str(fake_obsidian),
            tmp_path,
            0.5,
            tmp_path / "out",
            tmp_path / "work" / "d_c_prime",
        )
    assert (tmp_path / "work" / "d_c_prime" / "d_c_prime.json").is_file()
