"""Run directory and run manifest (CLAUDE.md §Ölçüm dürüstlüğü 7).

`reports/runs/<run_id>/manifest.json` records, per phase, the git sha, whether
the working tree was clean, the blake3 of every data manifest, toolchain
versions and the chosen parameters. Nothing here searches for "latest" runs.
"""

from __future__ import annotations

import json
import platform
import re
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path

import mdkernel_core

from eval.datasets.common import DATA, ROOT, SEED, dump_json, file_blake3

RUNS = ROOT / "reports" / "runs"
RUN_RE = re.compile(r"^\d{8}T\d{6}Z-[0-9a-f]{7}$")


def _git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout.strip()


def new_run_id() -> str:
    ts = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    return f"{ts}-{_git('rev-parse', 'HEAD')[:7]}"


def run_dir(run_id: str, create: bool = True) -> Path:
    if not RUN_RE.match(run_id):
        raise ValueError(f"malformed RUN {run_id!r} (expected <UTC yyyymmddThhmmssZ>-<sha7>)")
    d = RUNS / run_id
    if create:
        d.mkdir(parents=True, exist_ok=True)
    elif not d.is_dir():
        raise FileNotFoundError(f"run directory {d} does not exist")
    return d


def _versions() -> dict:
    rustc = subprocess.run(
        ["rustc", "--version"], cwd=ROOT, capture_output=True, text=True, check=False
    ).stdout.strip()
    return {"rust": rustc, "python": platform.python_version(), "platform": sys.platform}


def worktree_clean() -> bool:
    """True only if nothing is modified, staged or untracked (ignored paths —
    data/, reports/runs/ — do not count)."""
    return _git("status", "--porcelain", "--untracked-files=all") == ""


def data_manifests() -> dict:
    return {p.name: file_blake3(p) for p in sorted(DATA.glob("MANIFEST.*.json"))}


def record_phase(run_id: str, phase: str, params: dict, outputs: list[str]) -> None:
    """Adds/replaces `phase` in the run manifest."""
    d = run_dir(run_id)
    path = d / "manifest.json"
    m = (
        json.loads(path.read_text())
        if path.exists()
        else {"run_id": run_id, "seed": SEED, "phases": {}}
    )
    m["phases"][phase] = {
        "git_sha": _git("rev-parse", "HEAD"),
        "worktree_clean": worktree_clean(),
        "data_manifests": data_manifests(),
        "versions": _versions(),
        "matcher_defaults": list(mdkernel_core.matcher_defaults()),
        "params": params,
        "outputs": outputs,
    }
    path.write_text(dump_json(m))


def write_json(run_id: str, name: str, obj) -> Path:
    p = run_dir(run_id) / name
    p.write_text(dump_json(obj))
    return p


def read_json(run_id: str, name: str):
    p = run_dir(run_id, create=False) / name
    if not p.exists():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text())


if __name__ == "__main__":
    print(new_run_id())
