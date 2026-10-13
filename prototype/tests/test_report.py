"""report.py and compare_runs.py: only the named run, missing file → error,
numbers come from JSON, decision rule applied; H4-2 field comparison."""

import json
import shutil

import pytest

from eval import compare_runs as C
from eval import report as R
from eval import runinfo

STRATS = ["flat", "hybrid", "graph_flat", "graph_hybrid", "hier"]


def fake_run(d, **over):
    cells = {
        f"{s}/{c}/{t}": {
            "n": 3,
            "ndcg@10": 0.5,
            "recall@5": 0.5,
            "recall@10": 0.6,
            "mrr": 0.4,
            "full_cover@10": 0.3,
        }
        for s in STRATS
        for c in ("structural", "fixed")
        for t in ("single", "multi")
    }
    boot = {
        f"{a}-{b}/{t}/{m}": {"delta": 0.01, "ci95": [-0.01, 0.03], "n": 3, "samples": 10000}
        for a, b in (("graph_hybrid", "hybrid"), ("graph_flat", "flat"))
        for t in ("single", "multi")
        for m in ("ndcg@10", "full_cover@10")
    }
    ops = [
        "rephrase_light",
        "rephrase_heavy",
        "move",
        "dup_delete",
        "insert",
        "split",
        "merge",
        "delete",
    ]
    per_op = {op: {"steps": 2} for op in ops}
    for op in ("rephrase_light", "rephrase_heavy", "move", "dup_delete"):
        per_op[op]["retention"] = 1.0
    for op in ("insert", "split", "merge"):
        per_op[op]["false_match"] = 0.0
    for op in ("split", "merge"):
        per_op[op]["event_recall"] = 1.0
    phase = {
        "git_sha": "a" * 40,
        "worktree_clean": True,
        "data_manifests": runinfo.data_manifests(),
        "matcher_defaults": [0.6, 0.7],
        "versions": {"rust": "rustc 1.92.0", "python": "3.12.9", "platform": "darwin"},
        "params": {},
        "outputs": [],
    }
    files = {
        "manifest.json": {"run_id": "x", "seed": 20261013, "phases": {"h1s_tune": phase}},
        "h1s_tuning.json": {
            "grid": [
                {
                    "theta": t,
                    "theta_split": s,
                    "H1-S1": 1.0,
                    "H1-S2": 0.0,
                    "H1-S3": 1.0,
                    "H1-S4": 1.0,
                    "objective": 4.0,
                }
                for t in (0.5, 0.6)
                for s in (0.6, 0.7)
            ],
            "selected": {"theta": 0.6, "theta_split": 0.7},
            "grid_flat": True,
            "sensitivity_probe": {
                "theta": 0.95,
                "theta_split": 0.99,
                "H1-S1": 0.5,
                "H1-S2": 0.0,
                "H1-S3": 1.0,
                "H1-S4": 1.0,
            },
        },
        "h1s.json": {
            "H1-S1": 0.99,
            "H1-S2": 0.01,
            "H1-S3": 0.9,
            "H1-S4": 1.0,
            "n_s1": 5,
            "n_s2": 5,
            "n_s4": 2,
            "n_docs": 1,
            "n_steps": 10,
            "theta": 0.6,
            "theta_split": 0.7,
            "H1-S3_detail": {
                "precision": 1.0,
                "recall": 0.8,
                "f1": 0.9,
                "tp": 4,
                "n_pred": 4,
                "n_truth": 5,
            },
            "per_op": per_op,
        },
        "h1r.json": {
            "H1-R1": 0.9999,
            "H1-R2": 0.98,
            "touched_retention": 0.7,
            "blocks": 10,
            "untouched": 8,
            "touched": 2,
            "untouched_lost": 0,
            "pairs": 300,
            "repo": "r",
            "rev": "b" * 40,
            "commits": 300,
            "loss_categories": {"dup_content": 0, "hunk_adjacent": 0, "type_change": 0, "other": 0},
            "other_examples": [],
            "other_count": 0,
        },
        "h2_tuning.json": {
            "curve": [
                {"alpha": a, "ndcg_all": 0.5, "ndcg_single": 0.5, "ndcg_multi": 0.5}
                for a in (0.3, 0.6)
            ],
            "selected_alpha": 0.6,
            "n_questions": 50,
            "rule": "r",
        },
        "h2.json": {
            "H2-1": 0.05,
            "H2-2": 0.0,
            "H2-3": 2.0,
            "H2-4": 0.05,
            "H2-5": 0.06,
            "alpha": 0.6,
            "n_questions": {"single": 3, "multi": 3},
            "cells": cells,
            "bootstrap": boot,
            "latency_ms": {
                f"{s}/{c}": {"p50": 1.0, "p95": 2.0, "n": 3}
                for s in STRATS
                for c in ("structural", "fixed")
            },
        },
        "h3.json": {
            "H3-1": {"p50_ms": 10.0, "p95_ms": 20.0, "n": 200},
            "H3-2": {"lost": 0, "gained": 0},
            "summary": {
                "p50": 10.0,
                "p95": 20.0,
                "n": 200,
                "lost": 0,
                "gained": 0,
                "log_equal": True,
            },
            "rechunk": {"log_equal": True, "lineages": 5},
            "latency": {
                "hit_at_1_rate": 0.8,
                "hit_at_10_rate": 0.9,
                "n_edits": 200,
                "latencies_ms": [1.0, 2.0],
                "rows": [{"doc_path": "a.md", "ms": 1.0, "hit_at_1": True}],
            },
        },
        "h4.json": {
            "H4-1": {"manifest_hash": "h", "rebuilt_hash": "h", "equal": True},
            "docs_equal": True,
        },
    }
    for name, patch in over.items():
        files[name].update(patch)
    d.mkdir(parents=True, exist_ok=True)
    for name, obj in files.items():
        (d / name).write_text(json.dumps(obj))
    return d


@pytest.fixture
def runs(tmp_path, monkeypatch):
    monkeypatch.setattr(runinfo, "RUNS", tmp_path)
    return tmp_path


RUN = "20261013T000000Z-abcdef0"


def test_missing_file_is_an_error(runs):
    d = fake_run(runs / RUN)
    (d / "h2.json").unlink()
    with pytest.raises(FileNotFoundError, match="h2.json"):
        R.load_run(RUN)


def test_unknown_run_is_an_error(runs):
    with pytest.raises(FileNotFoundError):
        R.load_run("20261013T000001Z-abcdef0")
    with pytest.raises(ValueError):
        R.load_run("latest")


def test_build_uses_only_json_numbers_and_applies_decision(runs, monkeypatch, tmp_path):
    fake_run(runs / RUN)
    c = tmp_path / "commentary.md"
    c.write_text("Yorum metni, sayı yok.")
    monkeypatch.setattr(R, "COMMENTARY", c)
    text = R.build(RUN)
    assert "| H1-S2 | Sentetik yanlış eşleme | ≤ 0.02 | 0.010000 | GEÇTİ |" in text
    assert "H1 ∧ H2 ∧ H3 (∧ H4) geçti" in text
    assert "H4-2 doğrulamada ölçülür" in text
    assert "h1s.json:H1-S1" in text


def test_decision_rule_branches():
    base = {c: "GEÇTİ" for c, *_ in R.THRESHOLDS}
    base["H4-2"] = "doğrulamada"
    assert any("dondurulur" in x for x in R.decision(base))
    st = dict(base, **{"H4-1": "KALDI"})
    assert any("başka hiçbir sonuç geçerli sayılmaz; önce determinizm" in x for x in R.decision(st))
    st = dict(base, **{"H1-S2": "KALDI"})
    assert any("H1 geçmedi" in x for x in R.decision(st))
    st = dict(base, **{"H2-1": "KALDI"})
    assert any("araştırma kolu" in x for x in R.decision(st))
    st = dict(base, **{"H2-3": "KALDI"})
    assert any("Kısmi H2" in x for x in R.decision(st))
    st = dict(base, **{"H3-1": "KALDI"})
    assert any("sıcak katman" in x for x in R.decision(st))


def test_commentary_with_numbers_is_rejected(tmp_path, monkeypatch):
    c = tmp_path / "commentary.md"
    c.write_text("Bu yorumda 3 sayı var.")
    monkeypatch.setattr(R, "COMMENTARY", c)
    with pytest.raises(ValueError):
        R.commentary()


def test_compare_runs_equal_and_different(runs):
    a = fake_run(runs / "A")
    b = fake_run(runs / "B")
    r = C.compare(a, b)
    assert r["max_relative_difference"] == 0.0 and r["H4-2_equal"] and r["n_fields"] > 200
    # latency differences are ignored, quality differences are not
    fake_run(b, **{"h2.json": {"H2-3": 99.0}})
    assert C.compare(a, b)["max_relative_difference"] == 0.0
    fake_run(b, **{"h1s.json": {"H1-S2": 0.02}})
    r = C.compare(a, b)
    assert (
        r["max_relative_difference"] == pytest.approx(0.5)
        and "h1s.json:H1-S2" in r["differing_fields"]
    )
    shutil.rmtree(b)
    with pytest.raises(FileNotFoundError):
        C.compare(a, b)


def test_figures_are_written(runs, tmp_path):
    fake_run(runs / RUN)
    R.figures(R.load_run(RUN), tmp_path / "figs")
    assert sorted(p.name for p in (tmp_path / "figs").iterdir()) == [
        "h1s_grid.png",
        "h2_alpha.png",
        "h2_ndcg.png",
    ]


def test_compare_runs_covers_every_non_latency_leaf(runs):
    a = fake_run(runs / "A")
    b = fake_run(runs / "B")
    # latency-only differences are ignored
    fake_run(
        b,
        **{
            "h3.json": {
                "H3-1": {"p50_ms": 99.0, "p95_ms": 99.0, "n": 200},
                "summary": {
                    "p50": 9.0,
                    "p95": 9.0,
                    "n": 200,
                    "lost": 0,
                    "gained": 0,
                    "log_equal": True,
                },
                "latency": {
                    "hit_at_1_rate": 0.8,
                    "hit_at_10_rate": 0.9,
                    "n_edits": 200,
                    "latencies_ms": [5.0, 6.0],
                    "rows": [{"doc_path": "a.md", "ms": 7.0, "hit_at_1": True}],
                },
            }
        },
    )
    assert C.compare(a, b)["max_relative_difference"] == 0.0
    # a hit flag inside the latency rows is quality, not latency
    fake_run(
        b,
        **{
            "h3.json": {
                "latency": {
                    "hit_at_1_rate": 0.8,
                    "hit_at_10_rate": 0.9,
                    "n_edits": 200,
                    "latencies_ms": [1.0, 2.0],
                    "rows": [{"doc_path": "a.md", "ms": 1.0, "hit_at_1": False}],
                }
            }
        },
    )
    assert C.compare(a, b)["max_relative_difference"] == float("inf")
    # list-valued fields (the tuning grid) are compared element by element
    b2 = fake_run(runs / "B2")
    grid = json.loads((b2 / "h1s_tuning.json").read_text())["grid"]
    grid[0]["objective"] = 3.0
    fake_run(b2, **{"h1s_tuning.json": {"grid": grid}})
    r = C.compare(a, b2)
    assert any("grid" in k and "objective" in k for k in r["differing_fields"])
    # a field present in one run only is an error
    fake_run(b2, **{"h2.json": {"extra": 1}})
    with pytest.raises(Exception, match="field sets differ"):
        C.compare(a, b2)


def test_report_refuses_inconsistent_data_manifests(runs, monkeypatch, tmp_path):
    d = fake_run(runs / RUN)
    m = json.loads((d / "manifest.json").read_text())
    m["phases"]["h1s_tune"]["data_manifests"] = {"MANIFEST.d1.json": "x"}
    (d / "manifest.json").write_text(json.dumps(m))
    c = tmp_path / "commentary.md"
    c.write_text("Yorum.")
    monkeypatch.setattr(R, "COMMENTARY", c)
    with pytest.raises(RuntimeError, match="data manifests"):
        R.build(RUN)
