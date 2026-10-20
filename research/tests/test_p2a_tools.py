"""P2-A/A0 e: the bounded real-Obsidian retry."""

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
