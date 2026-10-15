"""P1-D/D7 tools: compare_p1d (D-8 pair, D-9 regress) catches every
difference outside the P1-D latency fields; run_p1d_checks names its
checks; report_p1d's decision rule (GOAL P1-D)."""

import json

import pytest

from eval import compare_p1d as P
from eval import metrics as M
from eval import report_p1d as R
from eval import run_p1d_checks as K


def _write(d, name, obj):
    p = d / name
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(obj) if not isinstance(obj, (bytes, str)) else obj)


def test_latency_fields_are_excluded_but_nothing_else(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    _write(a, "p1d/d10.json", {"latencies_ms": [1.0], "p95_ms": 1.0, "edits": 20, "pass": True})
    _write(b, "p1d/d10.json", {"latencies_ms": [9.0], "p95_ms": 9.0, "edits": 20, "pass": True})
    assert P.compare(a, b, ["p1d/d10.json"])["differing_fields"] == {}
    _write(b, "p1d/d10.json", {"latencies_ms": [9.0], "p95_ms": 9.0, "edits": 19, "pass": True})
    assert list(P.compare(a, b, ["p1d/d10.json"])["differing_fields"]) == ["p1d/d10.json:edits"]


def test_field_set_difference_is_an_error(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    _write(a, "p1d/scene/scene.json", {"pass": True, "x": 1})
    _write(b, "p1d/scene/scene.json", {"pass": True})
    with pytest.raises(M.MetricError):
        P.compare(a, b, ["p1d/scene/scene.json"])


def test_p1d_bytes_catch_one_byte(tmp_path, monkeypatch):
    a, b = tmp_path / "a", tmp_path / "b"
    for d in (a, b):
        _write(d, "p1d/mcp_e2e/run1/transcript.jsonl", "same\n")
    monkeypatch.setattr(P, "P1D_BYTES", [P.E2E / "transcript.jsonl"])
    assert all(v["equal"] for v in P.p1d_bytes(a, b).values())
    _write(b, "p1d/mcp_e2e/run1/transcript.jsonl", "samf\n")
    assert not any(v["equal"] for v in P.p1d_bytes(a, b).values())
    (b / "p1d/mcp_e2e/run1/transcript.jsonl").unlink()
    with pytest.raises(FileNotFoundError):
        P.p1d_bytes(a, b)


def test_regress_compares_conformance_tests(tmp_path, monkeypatch):
    a, b = tmp_path / "a", tmp_path / "b"
    conf = {
        "spec_version": "1.0.2",
        "level_claimed": 3,
        "fixtures_manifest_hash": "h",
        "fixtures_root": "/x",
        "tests": [{"id": "C1-01", "level": 1, "result": "pass"}],
    }
    _write(a, "conformance_p1c1.json", conf)
    _write(b, "conformance_p1c1.json", {**conf, "fixtures_root": "/elsewhere"})
    monkeypatch.setattr(P, "P1C1_FILES", [])
    assert P.regress(a, b)["equal"]
    _write(
        b,
        "conformance_p1c1.json",
        {**conf, "tests": [{"id": "C1-01", "level": 1, "result": "fail"}]},
    )
    r = P.regress(a, b)
    assert not r["equal"] and r["conformance"]["differing_tests"] == ["C1-01"]


def test_named_checks_must_exist():
    e2e = {"checks": [{"check": "a", "pass": True}]}
    assert K.named(e2e, ["a"])["pass"]
    with pytest.raises(KeyError):
        K.named(e2e, ["a", "missing"])


def _rows(**fail):
    ids = [f"D-{i}" for i in range(1, 12)]
    return [{"id": i, "pass": not fail.get(i.replace("-", "_"), False)} for i in ids]


def test_decision_rule():
    assert R.decision(_rows()) == ["D-1..D-11 geçti → **P1-E başlar**."]
    assert "yönetişim ihlali" in R.decision(_rows(D_4=True))[0]
    assert "determinizm" in R.decision(_rows(D_8=True))[0]
    assert "0.1.0 bekler" in R.decision(_rows(D_7=True))[0]
    assert "P1-E başlar" in R.decision(_rows(D_1=True))[0]
    both = R.decision(_rows(D_3=True, D_1=True))
    assert "P1-E başlamaz" in both[0] and "0.1.0 bekler" in both[1]
    assert "karar talebi" in R.decision(_rows(D_10=True))[0]
