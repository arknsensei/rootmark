"""P2-A: the bounded real-Obsidian retry (A0 e) and the round's classifier (A2)."""

import json
from pathlib import Path

import pytest

from eval import real_retry


def test_a_hung_save_is_retried_once(tmp_path: Path) -> None:
    seen: list[Path] = []

    def run(w: Path) -> dict:
        seen.append(w)
        if len(seen) == 1:
            raise RuntimeError(
                "node sessions.mjs: exit 1\nCDP evaluate timed out: __p21.saveAndWait"
            )
        return {"pass": True}

    r = real_retry.with_retry(run, tmp_path)
    assert r["pass"] is True
    assert [x.name for x in seen] == ["try1", "try2"]
    assert len(r["retries"]) == 1 and "timed out" in r["retries"][0]["error"]


def test_a_second_hang_fails_and_other_failures_are_not_retried(tmp_path: Path) -> None:
    def hang(w: Path) -> dict:
        raise RuntimeError("x.md was not written by Obsidian within 2 s of the save")

    with pytest.raises(RuntimeError):
        real_retry.with_retry(hang, tmp_path)
    calls: list[Path] = []

    def other(w: Path) -> dict:
        calls.append(w)
        raise RuntimeError("rootmark serve did not start")

    with pytest.raises(RuntimeError):
        real_retry.with_retry(other, tmp_path)
    assert len(calls) == 1


def test_no_retry_is_recorded_as_empty(tmp_path: Path) -> None:
    assert real_retry.with_retry(lambda w: {"pass": False}, tmp_path)["retries"] == []


# ---- the round's classifier (A2): historical, migration_required, fail


def _run(tmp: Path, name: str, docs: dict) -> Path:
    d = tmp / name
    d.mkdir()
    for f, doc in docs.items():
        (d / f).write_text(json.dumps(doc))
    return d


def test_classifier_labels(tmp_path: Path, monkeypatch) -> None:
    from eval import historical_p2a as h

    monkeypatch.setattr(h, "ROOT", tmp_path / "research")
    ref = {
        "criteria": {
            "E1-1": {"pass": True, "search": {"code": "index_stale"}, "n": 8},
            "D-2": {"pass": False, "x": 1},
            "Z-1": {"pass": True, "y": 1},
        }
    }
    _run(tmp_path, "ref", {"p1e1_checks.json": ref})
    now = {
        "criteria": {
            # the new behaviour: listed, only the listed path changed, to the expected value
            "E1-1": {"pass": False, "search": {"code": "migration_required"}, "n": 8},
            "D-2": {"pass": False, "x": 1},
            "Z-1": {"pass": False, "y": 2},
        }
    }
    run = _run(tmp_path, "run", {"p1e1_checks.json": now})
    listing = {
        "reference_run": "ref",
        "entries": [
            {
                "file": "p1e1_checks.json",
                "criterion": "D-2",
                "decisions": ["K-x"],
                "why": "w",
                "changes": [],
            }
        ],
    }
    mig = {
        "reference_run": "ref",
        "entries": [
            {
                "file": "p1e1_checks.json",
                "criterion": "E1-1",
                "decisions": ["K-P2A.2"],
                "why": "older index",
                "paths": [{"path": "search/code", "expect": "migration_required"}],
            }
        ],
    }
    r = h.classify(run, listing, mig, excl=[])
    lab = {x["criterion"]: x["label"] for x in r["fails"]}
    assert lab == {"E1-1": "migration_required", "D-2": "historical", "Z-1": "fail"}
    assert r["counted_fails"] == 1 and r["migration_required"] == 1

    # the listed path holding another value, or a difference elsewhere: fail
    now["criteria"]["E1-1"]["search"]["code"] = "internal"
    run2 = _run(tmp_path, "run2", {"p1e1_checks.json": now})
    assert {x["criterion"]: x["label"] for x in h.classify(run2, listing, mig, excl=[])["fails"]}[
        "E1-1"
    ] == "fail"
    now["criteria"]["E1-1"]["search"]["code"] = "migration_required"
    now["criteria"]["E1-1"]["n"] = 7
    run3 = _run(tmp_path, "run3", {"p1e1_checks.json": now})
    assert {x["criterion"]: x["label"] for x in h.classify(run3, listing, mig, excl=[])["fails"]}[
        "E1-1"
    ] == "fail"
