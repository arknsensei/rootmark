"""eval.compare_p1b1 (P1-B.1 GOAL B1-6): only the C2-07 entry, the manifest
hash and traces.jsonl may differ from the P1-B official RUN."""

from __future__ import annotations

import json
import shutil
from pathlib import Path

import pytest

from eval import compare_p1b1

P1B_RUN = Path(__file__).resolve().parents[2] / "reports/rounds/P1-B/runs/20261014T135400Z-7735a07"


def _copy(tmp_path: Path) -> Path:
    """A RUN equal to the P1-B official one, laid out as a current RUN: its
    vault under `.rootmark/` (K-GH.1; the official RUN keeps `.mdkernel/`)."""
    dst = tmp_path / "run"
    shutil.copytree(P1B_RUN, dst)
    (dst / "governance/vault/.mdkernel").rename(dst / "governance/vault/.rootmark")
    return dst


def _edit(path: Path, fn) -> None:
    obj = json.loads(path.read_text(encoding="utf-8"))
    fn(obj)
    path.write_text(json.dumps(obj), encoding="utf-8")


def _run(run: Path, capsys) -> dict:
    try:
        compare_p1b1.main([str(P1B_RUN), str(run)])
        code = 0
    except SystemExit as e:
        code = e.code
    out = capsys.readouterr().out
    return {"code": code, "res": json.loads(out) if out else None}


def test_equal_run(tmp_path, capsys):
    r = _run(_copy(tmp_path), capsys)
    assert r["code"] == 0 and r["res"]["B1-6_equal"]


def test_excluded_fields_may_differ(tmp_path, capsys):
    run = _copy(tmp_path)

    def c207(o):
        t = next(t for t in o["B-1"]["tests"] if t["id"] == "C2-07")
        t["actual_hash"] = t["expected_hash"] = "0" * 32
        o["fixtures_manifest_hash"] = "1" * 32
        o["governance"]["traces_sha256"] = "2" * 64

    _edit(run / "p1b_checks.json", c207)
    _edit(run / "governance/governance.json", lambda o: o.update(traces_sha256="2" * 64))
    (run / "governance/vault/.rootmark/traces.jsonl").write_text("changed\n")
    r = _run(run, capsys)
    assert r["code"] == 0, r
    assert len(r["res"]["differing_excluded"]) == 5


@pytest.mark.parametrize("what", ["other_test", "events", "quality"])
def test_other_differences_fail(tmp_path, capsys, what):
    run = _copy(tmp_path)
    if what == "other_test":

        def other(o):
            t = next(t for t in o["B-1"]["tests"] if t["id"] == "C2-06")
            t["actual_hash"] = "0" * 32

        _edit(run / "p1b_checks.json", other)
    elif what == "events":
        p = run / "governance/vault/.rootmark/events.jsonl"
        p.write_bytes(p.read_bytes() + b"\n")
    else:
        _edit(run / "holdout_split.json", lambda o: o.update(extra_marker=1))
    r = _run(run, capsys)
    assert r["code"] != 0
