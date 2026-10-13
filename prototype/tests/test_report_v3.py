"""report_v3: thresholds coded from docs/v3/00_GOAL.md, mechanical decision
rule, only the named run (missing file → error), numbers traceable."""

import json

import pytest

from eval import report_v3 as R
from eval import runinfo

RUN = "20261013T000000Z-abcdef0"


def all_pass() -> dict:
    st = {c: "GEÇTİ" for c, *_ in R.THRESHOLDS}
    st["H4-2"] = "doğrulamada"
    return st


def text(st):
    return "\n".join(R.decision(st))


def test_thresholds_match_goal_table():
    t = {c: (cmp, bound) for c, _, _, _, cmp, bound in R.THRESHOLDS}
    assert t["H1-S1"] == (">=", 0.95) and t["H1-S2"] == ("<=", 0.02)
    assert t["H1-S3"] == (">=", 0.80) and t["H1-S4"] == (">=", 0.95)
    assert t["H1-H1"] == (">=", 0.90) and t["H1-H2"] == ("<=", 0.03)
    assert t["H1-H3"] == (">=", 0.70) and t["H1-H4"] == (">=", 0.90)
    assert t["H1-R1"] == (">=", 0.999) and t["H1-R2"] == (">=", 0.85)
    assert t["H1-G"] == (">=", 3)
    assert t["H3-1"] == ("<", 1000.0) and t["H3-2"] == ("==", 0)
    assert t["H4-1"] == ("is", True) and t["H4-V2"] == ("is", True)
    assert t["H4-2"] == ("verify", None)


def test_all_pass_freezes_spec():
    assert "§1–7 (K4 ve v3 eşleyiciyle) dondurulur; çekirdek fazı 1 başlar" in text(all_pass())


def test_h4v2_failure_invalidates_everything():
    st = all_pass()
    st["H4-V2"] = "KALDI"
    t = text(st)
    assert "v2 davranışı korunamamış" in t and "dondurulur" not in t


def test_h4_1_failure_is_determinism():
    st = all_pass()
    st["H4-1"] = "KALDI"
    assert "determinizm bozuldu" in text(st)


def test_h1g_failure_makes_hard_informational():
    st = all_pass()
    st["H1-G"] = "KALDI"
    st["H1-H3"] = "KALDI"  # ignored: hard set is information only
    t = text(st)
    assert "zor set kanıtı eksik" in t
    assert "dondurulur" in t


def test_only_false_match_fails_means_one_more_matcher_round():
    st = all_pass()
    st["H1-H2"] = "KALDI"
    assert "eşleyici bir tur daha" in text(st)
    st["H1-S2"] = "KALDI"
    assert "eşleyici bir tur daha" in text(st)


def test_false_match_branch_requires_the_others_including_h3():
    st = all_pass()
    st["H1-S2"] = "KALDI"
    st["H3-1"] = "KALDI"
    t = text(st)
    assert "eşleyici bir tur daha" not in t and "dondurulur" not in t


def test_other_h1_failure_revisits_data_model():
    st = all_pass()
    st["H1-R1"] = "KALDI"
    t = text(st)
    assert "veri modeli yeniden ele alınır" in t and "dondurulur" not in t
    st = all_pass()
    st["H1-S2"] = st["H1-S3"] = "KALDI"
    assert "veri modeli yeniden ele alınır" in text(st)


def test_values_trace_to_json_paths():
    r = {
        "h1s_v3": {**{c: 0.5 for c in R.EASY + R.HARD}, "H1-G": 4},
        "h1s_v3_tuning": {"H1-G": 4},
        "h1r_v3": {"H1-R1": 1.0, "H1-R2": 0.9},
        "h3_v3": {"H3-1": {"p95_ms": 12.0}, "H3-2": {"lost": 0, "gained": 0}},
        "h4_v3": {"H4-1": {"equal": True}},
        "h4_v2repro": {"H4-V2": {"equal": True}},
    }
    v = R.values(r)
    assert v["H1-H2"] == (0.5, "h1s_v3.json:H1-H2")
    assert v["H1-G"] == (4, "h1s_v3_tuning.json:H1-G")
    assert v["H4-V2"] == (True, "h4_v2repro.json:H4-V2.equal")
    st = R.statuses(v)
    assert st["H1-S1"] == "KALDI" and st["H1-G"] == "GEÇTİ" and st["H4-2"] == "doğrulamada"
    r["h1s_v3"]["H1-G"] = 3
    with pytest.raises(RuntimeError):
        R.values(r)


def test_missing_file_is_an_error(tmp_path, monkeypatch):
    monkeypatch.setattr(runinfo, "RUNS", tmp_path)
    (tmp_path / RUN).mkdir()
    for n in R.REQUIRED[:-1]:
        (tmp_path / RUN / n).write_text(json.dumps({}))
    with pytest.raises(FileNotFoundError, match="h4_v2repro.json"):
        R.load_run(RUN)
