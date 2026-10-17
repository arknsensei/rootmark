"""P1-E.1 measurement helpers: the historical FAIL labels (K-E1.4) never
hide a new failure; the release checklist ids are read from RELEASE.md."""

import json

import pytest

from eval import compare_p1e1, historical, run_p1e1_checks


def _run(tmp, name, files):
    d = tmp / name
    d.mkdir()
    for f, doc in files.items():
        (d / f).write_text(json.dumps(doc))
    return d


def _listing(ref, entries):
    return {"reference_run": str(ref), "entries": entries}


ENTRY = {
    "file": "p1x_checks.json",
    "criterion": "X-1",
    "decisions": ["K-1"],
    "why": "w",
    "changes": [],
}


def test_listed_fail_equal_to_reference_is_historical(tmp_path):
    rec = {"X-1": {"pass": False, "v": 1}, "X-2": {"pass": True}}
    ref = _run(tmp_path, "ref", {"p1x_checks.json": rec})
    run = _run(tmp_path, "run", {"p1x_checks.json": rec})
    r = historical.classify(run, _listing(ref, [ENTRY]))
    assert r["counted_fails"] == 0 and r["historical"] == 1
    assert "tarihsel, karar K-1 ile kapandı" in historical.label(r["fails"][0])


def test_listed_fail_with_another_record_counts(tmp_path):
    ref = _run(tmp_path, "ref", {"p1x_checks.json": {"X-1": {"pass": False, "v": 1}}})
    run = _run(tmp_path, "run", {"p1x_checks.json": {"X-1": {"pass": False, "v": 2}}})
    r = historical.classify(run, _listing(ref, [ENTRY]))
    assert r["counted"] == ["p1x_checks.json:X-1"]
    assert r["fails"][0]["differing"] == ["v"]


def test_unlisted_fail_counts(tmp_path):
    ref = _run(tmp_path, "ref", {"p1x_checks.json": {"X-1": {"pass": True}}})
    run = _run(tmp_path, "run", {"p1x_checks.json": {"X-1": {"pass": False}}})
    r = historical.classify(run, _listing(ref, []))
    assert r["counted_fails"] == 1


def test_decision_exclusion_admits_only_its_subtree(tmp_path, monkeypatch):
    ref = _run(tmp_path, "ref", {"p1x_checks.json": {"X-1": {"pass": True, "a": {"b": 1}, "c": 1}}})
    run = _run(
        tmp_path, "run", {"p1x_checks.json": {"X-1": {"pass": False, "a": {"b": 2}, "c": 1}}}
    )
    monkeypatch.setattr(
        compare_p1e1,
        "EXCLUDED",
        [
            ("p1x_checks.json", ("X-1", "a"), "K-E1.2", "w"),
            ("p1x_checks.json", ("X-1", "pass"), "K-E1.2", "w"),
        ],
    )
    r = historical.classify(run, _listing(ref, []))
    assert r["counted_fails"] == 0 and r["fails"][0]["decisions"] == ["K-E1.2"]
    run2 = _run(
        tmp_path, "run2", {"p1x_checks.json": {"X-1": {"pass": False, "a": {"b": 2}, "c": 2}}}
    )
    assert historical.classify(run2, _listing(ref, []))["counted_fails"] == 1


def test_listed_criterion_missing_counts(tmp_path):
    ref = _run(tmp_path, "ref", {"p1x_checks.json": {"X-1": {"pass": False}}})
    run = _run(tmp_path, "run", {"p1x_checks.json": {"X-2": {"pass": True}}})
    r = historical.classify(run, _listing(ref, [ENTRY]))
    assert [f["label"] for f in r["fails"]] == ["missing"]


def test_unreadable_checks_file_is_an_error(tmp_path):
    ref = _run(tmp_path, "ref", {})
    run = tmp_path / "run"
    run.mkdir()
    (run / "p1x_checks.json").write_text("{ not json")
    with pytest.raises(SystemExit):
        historical.classify(run, _listing(ref, []))


def test_release_ids_cover_r1_to_r15():
    ids = run_p1e1_checks.release_ids()
    assert ids[0] == "R1" and "R4a" in ids and "R15" in ids and len(ids) == 16


def test_unlisted_fail_failing_in_reference_too_counts(tmp_path):
    rec = {"X-1": {"pass": False, "v": 1}}
    ref = _run(tmp_path, "ref", {"p1x_checks.json": rec})
    run = _run(tmp_path, "run", {"p1x_checks.json": rec})
    assert historical.classify(run, _listing(ref, []))["counted_fails"] == 1


def test_exclusion_at_criterion_or_with_wildcard_admits_nothing(tmp_path, monkeypatch):
    ref = _run(tmp_path, "ref", {"p1x_checks.json": {"X-1": {"pass": True, "a": 1}}})
    run = _run(tmp_path, "run", {"p1x_checks.json": {"X-1": {"pass": False, "a": 2}}})
    for pre in [("X-1",), ("X-1", "*"), ("*", "a")]:
        monkeypatch.setattr(compare_p1e1, "EXCLUDED", [("p1x_checks.json", pre, "K", "w")])
        assert historical.classify(run, _listing(ref, []))["counted_fails"] == 1, pre


def test_missing_checks_file_and_non_boolean_pass_count(tmp_path):
    ref = _run(tmp_path, "ref", {"p1x_checks.json": {"X-1": {"pass": False}}})
    run = _run(tmp_path, "run", {"p1y_checks.json": {"Y-1": {"pass": "yes"}}})
    r = historical.classify(run, _listing(ref, [ENTRY]))
    assert sorted(f["label"] for f in r["fails"]) == ["fail", "missing"]
    assert r["counted_fails"] == 2


def test_empty_container_is_not_absent(tmp_path):
    ref = _run(tmp_path, "ref", {"p1x_checks.json": {"X-1": {"pass": False}}})
    run = _run(tmp_path, "run", {"p1x_checks.json": {"X-1": {"pass": False, "d": {}}}})
    assert historical.classify(run, _listing(ref, [ENTRY]))["counted_fails"] == 1


def test_emptied_container_only_with_declared_content(tmp_path, monkeypatch):
    ref = _run(tmp_path, "ref", {"p1x_checks.json": {"X-1": {"pass": True, "d": {}}}})
    run = _run(tmp_path, "run", {"p1x_checks.json": {"X-1": {"pass": False, "d": {"k": 1}}}})
    excl = [
        ("p1x_checks.json", ("X-1", "pass"), "K", "w"),
        ("p1x_checks.json", ("X-1", "d", "k"), "K", "w"),
    ]
    monkeypatch.setattr(compare_p1e1, "EXCLUDED", excl)
    assert historical.classify(run, _listing(ref, []))["counted_fails"] == 0
    run2 = _run(
        tmp_path, "run2", {"p1x_checks.json": {"X-1": {"pass": False, "d": {"k": 1, "j": 2}}}}
    )
    assert historical.classify(run2, _listing(ref, []))["counted_fails"] == 1
