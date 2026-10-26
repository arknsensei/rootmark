"""P2-R tools (not measurers): the chain's alignment regression check
(R0 c5)."""

import copy
import json
from pathlib import Path

from eval.check_p2b_align_regress import regressions

ROOT = Path(__file__).resolve().parents[2]
R0 = json.loads((ROOT / "reports/rounds/P2-R/r0/align.json").read_text())
B2 = json.loads((ROOT / "reports/rounds/P2-B/b2/align.json").read_text())


def test_the_reference_does_not_regress_against_itself():
    assert regressions(R0, R0) == []


def test_p2b_values_regress_against_the_p2r_reference():
    bad = regressions(B2, R0)
    assert any(b.startswith("notes: error") for b in bad), bad
    assert any(b.startswith("edge.bom: error") for b in bad), bad


def test_one_class_higher_is_a_regression():
    got = copy.deepcopy(R0)
    got["edge"]["table"]["error"] = 0.001
    assert regressions(got, R0) == ["edge.table: error 0.001 > reference 0.0"]


def test_another_measured_set_is_a_regression():
    got = copy.deepcopy(R0)
    got["notes_digest"] = "0" * 32
    got["notes"]["blocks"] += 1
    bad = regressions(got, R0)
    assert len(bad) == 2 and bad[0].startswith("notes_digest"), bad
