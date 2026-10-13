"""scripts/verify.sh v3: `compare` mode exits non-zero when H4-2 ≠ 0
(docs/v3/04_VERIFY_DELTA.md V1); the chain runs every v3 phase."""

import json
import subprocess
from pathlib import Path

from eval import compare_p1a as P
from eval import compare_runs as C

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT.parent / "scripts" / "verify.sh"


def _runs(tmp_path):
    a, b = tmp_path / "A", tmp_path / "B"
    for d in (a, b):
        d.mkdir(parents=True)
        # P1-A: `verify.sh compare` runs compare_p1a (v3 set + holdout + split_grid)
        for i, name in enumerate(P.RESULT_FILES_P1A):
            (d / name).write_text(json.dumps({"value": 0.5 + i, "nested": {"n": [1, 2]}}))
        (d / "h3_v3.json").write_text(json.dumps({"H3-1": {"p95_ms": 10.0}, "x": 1}))
    return a, b


def _compare(a, b, out=None):
    # compare takes the monorepo root and runs compare_runs in <repo>/research
    args = ["bash", str(SCRIPT), "compare", str(ROOT.parent), str(a), str(b)]
    if out:
        args.append(str(out))
    return subprocess.run(args, capture_output=True, text=True, check=False)


def test_compare_mode_zero_when_equal(tmp_path):
    a, b = _runs(tmp_path)
    # wall-clock latency differs: excluded, H4-2 still 0
    (b / "h3_v3.json").write_text(json.dumps({"H3-1": {"p95_ms": 99.0}, "x": 1}))
    r = _compare(a, b, tmp_path / "h4_2.json")
    assert r.returncode == 0, r.stderr
    assert json.loads((tmp_path / "h4_2.json").read_text())["H4-2_equal"] is True


def test_compare_mode_nonzero_when_a_quality_number_differs(tmp_path):
    a, b = _runs(tmp_path)
    d = json.loads((a / "h1s_v3.json").read_text())
    d["value"] += 1e-9
    (a / "h1s_v3.json").write_text(json.dumps(d))
    r = _compare(a, b, tmp_path / "h4_2.json")
    assert r.returncode != 0
    assert "H4-2" in r.stderr
    out = json.loads((tmp_path / "h4_2.json").read_text())
    assert out["H4-2_equal"] is False and out["max_relative_difference"] > 0


def test_compare_mode_nonzero_on_missing_file(tmp_path):
    a, b = _runs(tmp_path)
    (b / "h4_v2repro.json").unlink()
    assert _compare(a, b).returncode != 0


def test_compare_mode_covers_holdout_and_split_grid(tmp_path):
    for name in ("holdout.json", "split_grid.json"):
        a, b = _runs(tmp_path / name)
        d = json.loads((a / name).read_text())
        d["value"] += 1e-9
        (a / name).write_text(json.dumps(d))
        r = _compare(a, b, tmp_path / f"{name}.out.json")
        assert r.returncode != 0, name
        out = json.loads((tmp_path / f"{name}.out.json").read_text())
        assert out["A-4_equal"] is False and out["per_file"][name] > 0
        a2, b2 = _runs(tmp_path / f"{name}-missing")
        (b2 / name).unlink()
        assert _compare(a2, b2).returncode == 1, name


def test_compare_runs_cli_exit_code(tmp_path):
    a, b = _runs(tmp_path)
    ok = subprocess.run(
        [
            "uv",
            "run",
            "--frozen",
            "python",
            "-m",
            "eval.compare_runs",
            str(a),
            str(b),
            "--set",
            "v3",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert ok.returncode == 0, ok.stderr
    (b / "h1r_v3.json").write_text(json.dumps({"value": 0.0, "nested": {"n": [1, 2]}}))
    bad = subprocess.run(
        [
            "uv",
            "run",
            "--frozen",
            "python",
            "-m",
            "eval.compare_runs",
            str(a),
            str(b),
            "--set",
            "v3",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    assert bad.returncode == C.NOT_EQUAL_EXIT


def test_chain_runs_v2_and_v3_phases():
    # P1-A chain (docs/rounds/P1-A/TASKS.md A4/A6): datasets, both freeze
    # checkers, conformance, and `make regress` (v2 + v3 phases compared with
    # the v3-final reference run); H2 is never run
    text = SCRIPT.read_text()
    chain = text.split("chain() {")[1].split("pair() {")[0]
    for step in (
        "make datasets",
        "make freeze-check\n",
        "make freeze-check-v3",
        "make conformance",
        'make regress RUN="$run"',
        'make eval-holdout RUN="$run"',
        'make eval-split-grid RUN="$run"',
        'make p1a-checks RUN="$run"',
    ):
        assert step in chain, step
    assert "eval-h2" not in chain
    makefile = (ROOT.parent / "Makefile").read_text()
    regress = makefile.split("\nregress:")[1].split("\n\n")[0]
    for step in (
        "eval-h1s-tune eval-h1s eval-h1r",
        "eval-h1s-v3 RUN=$(RUN) PHASE=tune",
        "eval-h1s-v3 RUN=$(RUN) PHASE=test",
        "eval-h1r-v3 eval-h3 eval-h4 eval-h4-v2repro",
        "eval.compare_runs reports/runs/$(V3_REF_RUN)",
    ):
        assert step in regress, step
