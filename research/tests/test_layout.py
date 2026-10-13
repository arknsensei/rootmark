"""research/ layout: Python here reaches the kernel only through the
`mdkernel_core` extension module (docs/PACKAGES.md; scripts/check_deps.sh)."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def test_research_root_has_pyproject():
    assert (ROOT / "pyproject.toml").is_file()
