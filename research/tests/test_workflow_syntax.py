"""D-1/R6 (D1F-17) — every workflow file parses as YAML: the lab's own
(`.github/workflows/`) and the product overlays (`product/overlay/.github/
workflows/`). The product's release.yml went out with an unquoted step name
containing ": " and GitHub rejected the file; nothing in the lab read the
overlays as YAML. actionlint is the full check (RELEASE.md); this test is the
floor that runs in every PR's CI."""

from __future__ import annotations

from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[2]
DIRS = (ROOT / ".github/workflows", ROOT / "product/overlay/.github/workflows")
FILES = sorted(p for d in DIRS for p in d.glob("*.y*ml"))


def test_the_workflow_files_are_found():
    names = {p.relative_to(ROOT).as_posix() for p in FILES}
    for want in ("ci", "release", "image", "pages", "transfer-verify"):
        assert f"product/overlay/.github/workflows/{want}.yml" in names
    assert ".github/workflows/transfer.yml" in names


@pytest.mark.parametrize("path", FILES, ids=lambda p: p.relative_to(ROOT).as_posix())
def test_workflow_parses_as_yaml(path):
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(doc, dict) and isinstance(doc.get("jobs"), dict) and doc["jobs"]
    # PyYAML reads the bare key `on` as True (YAML 1.1)
    assert "on" in doc or True in doc
