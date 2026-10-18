"""P2-0 data sets and the window rule (TASKS Z5): sizes, split, determinism,
the agent writer's boundary, `choose_window`."""

import json

import pytest

from eval.agent_fs_writer import apply_step
from eval.datasets import p2_0_sets as sets
from eval.run_p2_0 import choose_window


def test_d_a_has_200_scenarios_of_every_kind():
    a = sets.d_a()["scenarios"]
    assert len(a) == 200
    assert {s["kind"] for s in a} == set(sets.D_A_KINDS)
    assert len({s["id"] for s in a}) == 200
    for s in a:
        assert s["steps"], s["id"]
        for st in s["steps"]:
            assert st["op"] in {"write", "rename"}
            assert ".rootmark" not in st["path"]


def test_d_b_splits_100_tuning_200_test_sessions():
    tune, test = sets.d_b()
    assert len(tune["sessions"]) == 100 and len(test["sessions"]) == 200
    ids_tune = {s["id"] for s in tune["sessions"]}
    ids_test = {s["id"] for s in test["sessions"]}
    assert not ids_tune & ids_test
    assert {s["kind"] for s in tune["sessions"]} == set(sets.D_B_KINDS)
    assert {s["kind"] for s in test["sessions"]} == set(sets.D_B_KINDS)


def test_d_c_has_160_ticks_with_tagged_authors():
    c = sets.d_c()["ticks"]
    assert len(c) == 160
    assert {t["mode"] for t in c} <= {"apart", "same_doc_after", "collide"}
    for t in c:
        assert f"Human {t['id']}" in t["human"]["text"]
        assert f"Agent {t['id']}" in t["agent"]["text"]
        assert sets.author_of(f"Agent {t['id']}") == "agent"


def test_sets_are_deterministic():
    a = json.dumps([sets.d_a(), sets.d_b(), sets.d_c()], sort_keys=True)
    b = json.dumps([sets.d_a(), sets.d_b(), sets.d_c()], sort_keys=True)
    assert a == b


def test_d_e_vaults_are_committed():
    for name in sets.D_E_VAULTS:
        d = sets.OLD_VAULTS / name
        exp = json.loads((d / "expected.json").read_text())
        assert exp["written_by"] == "rootmark 0.1.0"
        assert (d / "store" / "events.jsonl").is_file()
        assert not (d / "store" / "server.token").exists()


def test_agent_writer_stays_out_of_the_store(tmp_path):
    for bad in [".rootmark/events.jsonl", "../x.md", "/tmp/x.md", "a/.rootmark/x.md"]:
        with pytest.raises(ValueError):
            apply_step(tmp_path, {"op": "write", "path": bad, "text": "x"})
    apply_step(tmp_path, {"op": "write", "path": "d/n.md", "text": "# N\n"})
    apply_step(tmp_path, {"op": "rename", "path": "d/n.md", "to": "m.md"})
    assert (tmp_path / "m.md").read_text() == "# N\n"
    with pytest.raises(ValueError):
        apply_step(tmp_path, {"op": "delete", "path": "m.md"})


def _row(w, rate, p95):
    return {"window_s": w, "false_unverified_rate": rate, "latency_p95_ms": p95}


def test_choose_window_smallest_lowest_rate_under_500():
    rows = [_row(0.5, 0.02, 150), _row(1.0, 0.0, 160), _row(2.0, 0.0, 150), _row(3.0, 0.0, 600)]
    assert choose_window(rows) == 1.0
    # the lowest rate only above 500 ms: still the smallest with that rate
    rows = [_row(0.5, 0.02, 150), _row(2.0, 0.0, 700), _row(3.0, 0.0, 650)]
    assert choose_window(rows) == 2.0
    # an unmeasured latency counts as not below 500
    rows = [_row(0.5, 0.0, None), _row(1.0, 0.0, 200)]
    assert choose_window(rows) == 1.0


def test_choose_window_zero_latency_is_measured():
    rows = [_row(0.5, 0.0, 0), _row(1.0, 0.0, 120)]
    assert choose_window(rows) == 0.5
