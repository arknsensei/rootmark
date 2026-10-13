"""run_h1r_v3: same pair count as the v2 run; the matcher is v3."""

import json
import subprocess
from pathlib import Path

from eval import run_h1r_v3

ROOT = Path(__file__).resolve().parents[1]


def test_pair_count_is_the_v2_count():
    out = subprocess.run(
        ["git", "show", "v2-final:reports/runs/20261013T154851Z-f133d0b/h1r.json"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    assert json.loads(out)["pairs"] == run_h1r_v3.V2_PAIRS == 836


def test_matcher_is_v3():
    assert run_h1r_v3.MATCHER == "v3"
