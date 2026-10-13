"""v3 runners (T25, T26): fixed reference values, D3 manifest v2 fields
unchanged since v2-final, matcher wiring."""

import json
import subprocess
from pathlib import Path

import mdkernel_core as mk

from eval import d3_v3, run_h1r_v3, run_h4_v2repro

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = "data/MANIFEST.d3.json"


def git_show(rev_path: str) -> str:
    return subprocess.run(
        ["git", "show", rev_path], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout


def test_d3_manifest_v2_fields_unchanged_since_v2_final():
    old = json.loads(git_show(f"v2-final:{MANIFEST}"))
    new = json.loads((ROOT / MANIFEST).read_text())
    assert d3_v3.v2_fields(new) == old
    assert old["events_hash"] == run_h4_v2repro.D3_EXPECTED
    assert set(new) - set(old) <= {d3_v3.V3_FIELD}


def test_d3_manifest_git_diff_only_adds_the_v3_field(tmp_path):
    # P1-A moved data/ under research/: diff the v2-final blob against the moved file
    old = tmp_path / "MANIFEST.d3.v2-final.json"
    old.write_text(git_show(f"v2-final:{MANIFEST}"))
    diff = subprocess.run(
        ["git", "diff", "--no-index", "-U0", "--", str(old), str(ROOT / MANIFEST)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    changed = [
        ln for ln in diff.splitlines() if ln[:1] in "+-" and not ln.startswith(("+++", "---"))
    ]
    # sorted keys: the only change is one added line carrying the v3 hash
    assert len(changed) == 1 and changed[0].startswith("+"), changed
    assert f'"{d3_v3.V3_FIELD}"' in changed[0]


def test_v2_reference_values():
    ref = json.loads(git_show(f"{run_h4_v2repro.REF_TAG}:{run_h4_v2repro.REF_PATH}"))
    assert ref["split"] == "test" and (ref["theta"], ref["theta_split"]) == (0.6, 0.7)
    h1r = json.loads(git_show(f"v2-final:reports/runs/{run_h4_v2repro.REF_RUN}/h1r.json"))
    assert h1r["pairs"] == run_h1r_v3.V2_PAIRS


def test_matcher_env_restores(monkeypatch):
    monkeypatch.delenv("MDK_MATCHER", raising=False)
    with d3_v3.matcher_env("v2"):
        assert mk.matcher_defaults() == mk.matcher_defaults("v2")
    assert mk.matcher_defaults() == mk.matcher_defaults("v3")


def test_ingest_docs_uses_requested_matcher(tmp_path):
    docs = ROOT.parent / "spec" / "fixtures" / "mini_vault"
    v2 = d3_v3.ingest_docs(tmp_path / "a", "v2", docs)
    v3 = d3_v3.ingest_docs(tmp_path / "b", "v3", docs)
    assert (v2.matcher_lock(), v3.matcher_lock()) == ("v2", "v3")
    assert v2.events_hash() == "8914f9b418ae360d49c85c951050f432"
    assert v3.events_hash() != v2.events_hash()
