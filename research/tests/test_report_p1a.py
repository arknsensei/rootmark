"""report_p1a: one RUN directory only (missing input → error), thresholds
from docs/rounds/P1-A/GOAL.md, decision rule applied mechanically."""

import copy
import json

import pytest

from eval import report_p1a as R

BASE = {
    "p1a_checks.json": {
        "A-1": {"passed": 17, "total": 17, "level_claimed": 1, "fixtures_manifest": "ok"},
        "A-5": {"events_hash": dict(R.A5)},
        "A-6": {
            "violations": 0,
            "check_deps_ok": True,
            "deliberate_reverse_dependency_test_ok": True,
        },
    },
    "holdout.json": {"H1-H1": 0.95, "H1-H2": 0.01, "H1-H3": 0.8, "H1-H4": 0.95},
    "split_grid.json": {"A-7": 3},
    "regress_vs_v3final.json": {"n_fields": 10, "max_relative_difference": 0.0, "H4-2_equal": True},
    "pair_compare.json": {
        "max_relative_difference": 0.0,
        "n_fields": 10,
        "files": ["x"],
        "A-4_equal": True,
    },
}


def _status(d):
    return {r["code"]: r["pass"] for r in R.criteria(d)}


def test_all_pass_confirms_freeze_and_starts_p1b():
    rows = R.criteria(BASE)
    assert all(r["pass"] in (True, None) for r in rows)
    assert any("P1-B başlar" in x for x in R.decision(rows))


@pytest.mark.parametrize(
    "key,value,code",
    [
        ("H1-H1", 0.8999, "A-2"),
        ("H1-H2", 0.0301, "A-2"),
        ("H1-H3", 0.6999, "A-2"),
        ("H1-H4", 0.8999, "A-2"),
    ],
)
def test_a2_thresholds_are_the_goal_thresholds(key, value, code):
    d = copy.deepcopy(BASE)
    assert _status(d)[code] is True
    d["holdout.json"][key] = value
    assert _status(d)[code] is False
    dec = R.decision(R.criteria(d))
    assert any("karar talebi" in x and "P1-B beklemez" in x for x in dec)
    assert not any("P1-B başlar" in x for x in dec)


def test_a2_bounds_are_inclusive():
    d = copy.deepcopy(BASE)
    d["holdout.json"].update({"H1-H1": 0.90, "H1-H2": 0.03, "H1-H3": 0.70, "H1-H4": 0.90})
    assert _status(d)["A-2"] is True


@pytest.mark.parametrize(
    "mutate",
    [
        lambda d: d["regress_vs_v3final.json"].update(
            max_relative_difference=1e-9, **{"H4-2_equal": False}
        ),
        lambda d: d["pair_compare.json"].update(
            max_relative_difference=1e-9, **{"A-4_equal": False}
        ),
        lambda d: d["p1a_checks.json"]["A-5"]["events_hash"].update(v2="0" * 32),
    ],
)
def test_migration_failures_block_p1b(mutate):
    d = copy.deepcopy(BASE)
    mutate(d)
    dec = R.decision(R.criteria(d))
    assert any("taşıma hatası; P1-B başlamaz" in x for x in dec)


def test_a1_or_a6_failure_blocks_freeze():
    for mutate in (
        lambda d: d["p1a_checks.json"]["A-1"].update(passed=16),
        lambda d: d["p1a_checks.json"]["A-6"].update(violations=1),
    ):
        d = copy.deepcopy(BASE)
        mutate(d)
        dec = R.decision(R.criteria(d))
        assert any("P1-B başlamaz" in x for x in dec)
        assert not any("P1-B başlar" in x for x in dec)


def test_a4_without_pair_is_pending():
    d = copy.deepcopy(BASE)
    d["pair_compare.json"] = None
    assert _status(d)["A-4"] is None
    assert any("doğrulamada" in x for x in R.decision(R.criteria(d)))


def test_missing_input_file_is_an_error(tmp_path):
    for name in R.REQUIRED:
        (tmp_path / name).write_text(json.dumps({}))
    R.load(tmp_path)  # complete set loads
    (tmp_path / "holdout.json").unlink()
    with pytest.raises(FileNotFoundError, match="holdout.json"):
        R.load(tmp_path)
