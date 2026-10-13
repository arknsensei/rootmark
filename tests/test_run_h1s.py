"""run_h1s guards: tune/test separation, constant check, selection rule."""

import pytest

from eval import run_h1s as R


def g(th, ts, obj):
    return {"theta": th, "theta_split": ts, "objective": obj}


def test_select_max_objective_then_closest_to_default():
    assert R.select([g(0.5, 0.6, 1.0), g(0.7, 0.8, 2.0)]) == g(0.7, 0.8, 2.0)
    tie = [g(0.5, 0.6, 2.0), g(0.65, 0.7, 2.0), g(0.6, 0.8, 2.0)]
    assert R.select(tie) == g(0.65, 0.7, 2.0)


def test_test_phase_refuses_mismatched_constants(monkeypatch):
    monkeypatch.setattr(R, "verify_d1", dict)
    monkeypatch.setattr(
        R.runinfo,
        "read_json",
        lambda run, name: {"selected": {"theta": 0.99, "theta_split": 0.99}, "tune_ids": []},
    )
    with pytest.raises(RuntimeError, match="compiled THETA"):
        R.phase_test("20261013T000000Z-abcdef0")


@pytest.mark.skipif(not (R.D1 / "splits.json").exists(), reason="D1 not built (make d1)")
def test_splits_disjoint_on_disk():
    s = R.splits()
    assert s["tune"] and s["test"] and not set(s["tune"]) & set(s["test"])
