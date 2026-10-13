"""scripts/verify.sh: strict mode, no absolute paths, valid syntax."""

import re
import subprocess
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "verify.sh"


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
    r = subprocess.run(["bash", str(SCRIPT), "bogus"], capture_output=True, text=True)
    assert r.returncode == 2 and "usage" in r.stderr
