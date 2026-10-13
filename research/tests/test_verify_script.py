"""scripts/verify.sh: strict mode, no absolute paths, valid syntax."""

import re
import subprocess
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "scripts" / "verify.sh"


def test_strict_mode_and_syntax():
    text = SCRIPT.read_text()
    assert "set -euo pipefail" in text
    subprocess.run(["bash", "-n", str(SCRIPT)], check=True)


def test_no_absolute_paths():
    for i, line in enumerate(SCRIPT.read_text().splitlines(), 1):
        code = line.split("#", 1)[0]
        assert not re.search(r"(?<![\w$}])/(Users|home|private|opt|var|tmp|usr)\b", code), (
            f"line {i}: {line}"
        )


def test_usage_error_without_valid_mode():
    r = subprocess.run(["bash", str(SCRIPT), "bogus"], capture_output=True, text=True, check=False)
    assert r.returncode == 2 and "usage" in r.stderr


def test_pair_stops_when_a_chain_step_fails(tmp_path):
    src = tmp_path / "src"
    src.mkdir()
    (src / "Makefile").write_text("setup:\n\t@echo failing setup; exit 1\n")

    def git(*a):
        subprocess.run(["git", *a], cwd=src, check=True, capture_output=True)

    git("init", "-q")
    git("-c", "user.email=t@t", "-c", "user.name=t", "add", "Makefile")
    git("-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "x")
    r = subprocess.run(
        ["bash", str(SCRIPT), "pair", str(src), str(tmp_path / "A"), str(tmp_path / "B")],
        capture_output=True,
        text=True,
        check=False,
    )
    assert r.returncode != 0
    assert not (tmp_path / "B").exists(), "the second chain must not start after a failure"
