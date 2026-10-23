"""P2-0.3: the thin wrapper's seed (TASKS R0) — D-B test-4 gets the new seed,
nothing else does, and D-B test-3's sessions are never generated."""

from pathlib import Path

import blake3
import pytest

from eval import report_p2_0_3 as rep
from eval import run_p2_0_3 as r3
from eval.datasets import p2_0_2_sets as sets2


def test_the_seed_is_the_pre_registered_one() -> None:
    assert r3.SEED == int(blake3.blake3(b"P2-0.3/D-B").hexdigest()[:8], 16) == 2671050228
    assert r3.SEED not in r3.OLD_SEEDS
    assert sets2.d_b_seed() in r3.OLD_SEEDS


def test_the_new_seed_reaches_d_b_only_inside_the_block() -> None:
    old_vault = sets2.vault3()
    old_a_prime = sets2.d_a_prime()
    with r3._seed(r3.SEED), r3._guard():
        s = sets2.d_b3()
        cov = sets2.coverage()
        new_vault = sets2.vault3()
    assert s["seed"] == cov["seed"] == r3.SEED and len(s["sessions"]) == 200
    assert cov["ok"], cov["zero"]
    assert new_vault["texts"] != old_vault["texts"]
    # after the block: P2-0.2's sets as they were (D-A′ on the test-3 vault)
    assert sets2.d_b_seed() in r3.OLD_SEEDS
    assert sets2.vault3() == old_vault
    assert sets2.d_a_prime() == old_a_prime


def test_d_b_test_3_sessions_are_refused_under_the_guard() -> None:
    with r3._guard(), pytest.raises(RuntimeError, match="only with the seed"):
        sets2.d_b3()
    sets2.d_b3()  # the guard is gone after the block


def test_run_d_b4_without_obsidian_is_skipped(tmp_path: Path) -> None:
    r = r3.run_d_b4("rootmark", str(tmp_path / "none"), tmp_path, 0.5, tmp_path)
    assert r["skipped"] is True and r["pass"] is None


def _chains(seed_ok: bool = True, logs_ok: bool = True) -> dict:
    c = {"logs_vs_p2a": {"pass": logs_ok}, "seed": {"pass": seed_ok}}
    return {k: c for k in ("A", "B", "load A", "load B")}


def _rows(failed: set[str], waiting: set[str] = frozenset()) -> list[dict]:
    return [{"id": i, "pass": None if i in waiting else i not in failed} for i in rep.IDS]


@pytest.mark.parametrize(
    ("failed", "branch"),
    [
        (set(), "Hepsi geçti"),
        ({"H-P2-0.3b-yük"}, "Yalnız 0.3b-yük kaldı"),
        ({"H-P2-0.3b"}, "0.3b kaldı"),
        ({"H-P2-0.3b", "H-P2-0.3b-yük"}, "0.3b kaldı"),
        ({"H-P2-0.3b", "H-P2-0.3c"}, "Tanımlanmamış kalış"),
        ({"H-P2-0.3c"}, "Tanımlanmamış kalış"),
        ({"H-P2-0.3e", "H-P2-0.3b"}, "Tasarım kusuru"),
        ({"H-P2-0.3a′"}, "Tasarım kusuru"),
        ({"H-P2-0.3i"}, "Altyapı/regresyon kusuru"),
        ({"H-P2-0.3b", "H-P2-0.3j"}, "Altyapı/regresyon kusuru"),
    ],
)
def test_the_decision_rule_branches(failed: set[str], branch: str) -> None:
    rule, got = rep.decide(_rows(failed), _chains())
    assert branch in rule and set(got) == failed


def test_the_rule_waits_and_an_invalid_round_comes_first() -> None:
    assert "Karar bekler" in rep.decide(_rows(set(), {"H-P2-0.3b"}), _chains())[0]
    # the logs against P2-A are reported, not a branch (ASSUMPTIONS R0-6)
    assert "Hepsi geçti" in rep.decide(_rows(set()), _chains(logs_ok=False))[0]
    assert "Tur geçersiz" in rep.decide(_rows(set()), _chains(seed_ok=False))[0]
