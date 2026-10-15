"""P1-C.1/C1.4 tools: compare_p1c1 exclusions (K-C1.6), C1-9 comparison,
gen_quality_ref refuses without C1-1 evidence (K-C1.2)."""

import json

import pytest

from eval import compare_p1c1 as P
from eval import gen_quality_ref as G
from eval import metrics as M
from eval import run_p1c1_checks as R


def _write(d, name, obj):
    p = d / name
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(obj))


def test_allowed_subtree_differs_but_nothing_else(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    _write(
        a,
        "p1c_checks.json",
        {
            "C-1": {"spec_version": "1.0.1", "pass": True},
            "C-11": {"pass": True, "differing_outside_exclusions": []},
        },
    )
    _write(
        b,
        "p1c_checks.json",
        {
            "C-1": {"spec_version": "1.0.2", "pass": True},
            "C-11": {"pass": False, "differing_outside_exclusions": ["x"]},
        },
    )
    res = P.compare(a, b, ["p1c_checks.json"], allow=True)
    assert res["differing_fields"] == {}
    assert set(res["differing_excluded"]) == {
        "p1c_checks.json:C-1›spec_version",
        "p1c_checks.json:C-11›pass",
        "p1c_checks.json:C-11›differing_outside_exclusions",
    }
    # without the exclusions the field sets differ → error
    with pytest.raises(M.MetricError):
        P.compare(a, b, ["p1c_checks.json"], allow=False)


def test_difference_outside_the_list_is_reported(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    _write(a, "p1c_checks.json", {"C-1": {"spec_version": "1.0.1", "pass": True}})
    _write(b, "p1c_checks.json", {"C-1": {"spec_version": "1.0.1", "pass": False}})
    res = P.compare(a, b, ["p1c_checks.json"], allow=True)
    assert list(res["differing_fields"]) == ["p1c_checks.json:C-1›pass"]


def _c3(v):
    row = {"id": "s1", "type": "single", "trace_id": "t", **dict.fromkeys(G.METRICS, v)}
    return {
        "cells": {"hybrid/structural/single": {"n": 1, **dict.fromkeys(G.METRICS, v)}},
        "per_query": {"hybrid/structural": [row]},
    }


def test_quality_vs_ref_exact():
    ref = {"source_run": "r", **G.reference(_c3(0.5))}
    assert R.quality_vs_ref(_c3(0.5), ref)["pass"]
    bad = R.quality_vs_ref(_c3(0.5000001), ref)
    assert not bad["pass"] and bad["differing"]


def test_gen_quality_ref_refuses_failed_evidence(tmp_path):
    p = tmp_path / "c1_variant.json"
    p.write_text(json.dumps({"criterion": "C1-1", "pass": False, "rust_run": G.SOURCE_RUN.name}))
    with pytest.raises(RuntimeError, match="did not pass"):
        G.main(["--c1-variant", str(p)])
