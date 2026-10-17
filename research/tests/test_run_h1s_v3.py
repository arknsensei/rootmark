"""run_h1s_v3: test phase refuses a constant mismatch; selection rule; dominant lookup."""

import json

import pytest
import rootmark_core as mk

from eval import run_h1s_v3 as R
from eval import runinfo

RUN = "20261013T000000Z-abcdef0"


@pytest.fixture
def runs(tmp_path, monkeypatch):
    monkeypatch.setattr(runinfo, "RUNS", tmp_path)
    return tmp_path


def _tuning(runs, theta, theta_split):
    d = runs / RUN
    d.mkdir()
    (d / "h1s_v3_tuning.json").write_text(
        json.dumps(
            {"selected": {"theta": theta, "theta_split": theta_split}, "tune_ids": [], "H1-G": 3}
        )
    )


def test_test_phase_errors_on_constant_mismatch(runs):
    th, ts = mk.matcher_defaults("v3")
    other = (0.50, 0.55) if (th, ts) != (0.50, 0.55) else (0.70, 0.75)
    _tuning(runs, *other)
    with pytest.raises(RuntimeError, match="!= tuning selection"):
        R.phase_test(RUN)
    assert not (runs / RUN / "h1s_v3.json").exists()


def test_compiled_check_accepts_equal_constants():
    th, ts = mk.matcher_defaults("v3")
    R.check_compiled({"theta": th, "theta_split": ts})


def test_test_phase_needs_tuning_file(runs):
    (runs / RUN).mkdir()
    with pytest.raises(FileNotFoundError):
        R.phase_test(RUN)


def test_select_max_objective_ties_closest_to_default():
    g = [
        {"theta": 0.50, "theta_split": 0.55, "objective": 3.9},
        {"theta": 0.65, "theta_split": 0.65, "objective": 3.9},
        {"theta": 0.70, "theta_split": 0.75, "objective": 3.8},
    ]
    assert R.select(g)["theta"] == 0.65
    g.append({"theta": 0.60, "theta_split": 0.65, "objective": 3.9})
    assert (R.select(g)["theta"], R.select(g)["theta_split"]) == (0.60, 0.65)


def test_grid_is_the_spec_grid():
    assert R.THETAS == [0.50, 0.55, 0.60, 0.65, 0.70]
    assert R.THETA_SPLITS == [0.55, 0.65, 0.75]


def test_dominant_index():
    a, b = {"version_id": "a" * 32, "k": 0}, {"version_id": "b" * 32, "k": 0}
    assert (
        R.dominant_index({"op": "split_edit", "step": 1, "mapping": [[a, [a, b]]], "dominant": b})
        == 1
    )
    assert (
        R.dominant_index(
            {"op": "merge_edit", "step": 1, "mapping": [[a, [b]], [b, [b]]], "dominant": a}
        )
        == 0
    )
    assert R.dominant_index({"op": "move_edit", "step": 1, "mapping": [[a, [a]]]}) is None
    with pytest.raises(RuntimeError):
        R.dominant_index(
            {
                "op": "split",
                "step": 1,
                "mapping": [[a, [a, b]]],
                "dominant": {"version_id": "c" * 32, "k": 0},
            }
        )
