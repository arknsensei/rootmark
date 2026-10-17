"""P1-E/E7 tools: compare_p1e (E-8 pair, E-9 regress) catches every
difference outside the declared exclusions; P1-D.1's informational D1-5
fields stay uncompared in the P1-E comparisons too; check_compare_fields
sees the P1-E comparator."""

import json

import pytest

from eval import check_compare_fields as K
from eval import compare_p1e as E


def _write(d, name, obj):
    p = d / name
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(obj) if not isinstance(obj, (bytes, str)) else obj)


def test_every_p1e_field_is_compared(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    _write(a, "p1e/client.json", {"bundle_bytes": 40701, "pass": True})
    _write(b, "p1e/client.json", {"bundle_bytes": 40701, "pass": True})
    assert E.compare_files(a, b, ["p1e/client.json"], lambda _n, _p: None)["differing_fields"] == {}
    _write(b, "p1e/client.json", {"bundle_bytes": 40702, "pass": True})
    r = E.compare_files(a, b, ["p1e/client.json"], lambda _n, _p: None)
    assert list(r["differing_fields"]) == ["p1e/client.json:bundle_bytes"]


def test_absent_field_is_a_difference(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    _write(a, "p1e/scene/scene.json", {"pass": True, "x": 1})
    _write(b, "p1e/scene/scene.json", {"pass": True})
    r = E.compare_files(a, b, ["p1e/scene/scene.json"], lambda _n, _p: None)
    assert r["differing_fields"] == {"p1e/scene/scene.json:x": {"a": 1, "b": "absent"}}


def test_p1d1_informational_fields_stay_uncompared(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    _write(
        a, "p1d1/d1_5.json", {"responses_a_vs_b_equal": True, "write_responses_a_vs_b_equal": True}
    )
    _write(
        b, "p1d1/d1_5.json", {"responses_a_vs_b_equal": False, "write_responses_a_vs_b_equal": True}
    )
    assert E.compare_files(a, b, ["p1d1/d1_5.json"], lambda _n, _p: None)["differing_fields"] == {}
    _write(
        b,
        "p1d1/d1_5.json",
        {"responses_a_vs_b_equal": False, "write_responses_a_vs_b_equal": False},
    )
    r = E.compare_files(a, b, ["p1d1/d1_5.json"], lambda _n, _p: None)
    assert list(r["differing_fields"]) == ["p1d1/d1_5.json:write_responses_a_vs_b_equal"]


def test_exclusion_lists_what_it_hides(tmp_path, monkeypatch):
    a, b = tmp_path / "a", tmp_path / "b"
    _write(a, "p1d1/d1_5.json", {"propose_denied": 8, "pass": True})
    _write(b, "p1d1/d1_5.json", {"propose_denied": 0, "pass": True})
    monkeypatch.setattr(E, "EXCLUDED", [("p1d1/d1_5.json", ("propose_denied",), "K-E0.4", "why")])
    r = E.compare_files(a, b, ["p1d1/d1_5.json"], E._excl_match)
    assert r["differing_fields"] == {}
    assert r["excluded_differences"] == {
        "p1d1/d1_5.json:propose_denied": {"decision": "K-E0.4", "leaves": 1}
    }


def test_p1e_bytes_catch_one_byte(tmp_path, monkeypatch):
    a, b = tmp_path / "a", tmp_path / "b"
    for d in (a, b):
        _write(d, "p1e/scene/events.jsonl", "same\n")
    monkeypatch.setattr(E, "P1E_BYTES", [E.P1E / "scene" / "events.jsonl"])
    assert all(v["equal"] for v in E.p1e_bytes(a, b).values())
    _write(b, "p1e/scene/events.jsonl", "samf\n")
    assert not any(v["equal"] for v in E.p1e_bytes(a, b).values())
    (b / "p1e/scene/events.jsonl").unlink()
    with pytest.raises(FileNotFoundError):
        E.p1e_bytes(a, b)


def test_declaration_covers_compare_p1e():
    r = K.audit()
    assert "compare_p1e" in r["modules"]
    assert r["unmatched"] == 0


def _rows(failed=(), e11=None):
    from eval import report_p1e as R

    r = [{"id": i, "pass": i not in failed} for i in R.GOAL_IDS] + [{"id": "E-11", "pass": e11}]
    ad = [{"id": i, "pass": True} for i in R.ADDENDUM_IDS]
    return r, ad


def test_decision_rule_branches():
    from eval import report_p1e as R

    assert "Faz 1 tamamlandı" in R.decide(*_rows())[0]
    assert "yayım ertelenir" in R.decide(*_rows(["E-7"]))[0]
    assert "istemci yönetişimi" in R.decide(*_rows(["E-4", "E-7"]))[0]
    assert "UI eksik" in R.decide(*_rows(["E-2"]))[0]
    assert "karar talebi" in R.decide(*_rows(["E-2", "E-7"]))[0]
    assert "karar talebi" in R.decide(*_rows(["E-9"]))[0]
    r, ad = _rows()
    ad[0]["pass"] = False  # E-12
    assert "E1 başlamazdı" in R.decide(r, ad)[0]
