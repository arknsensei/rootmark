"""P2-0 measurement tools."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

import blake3
import pytest

from eval import run_artifacts


def _run(tmp_path: Path, files: dict[str, bytes], recorded: dict[str, dict]) -> Path:
    for name, data in files.items():
        p = tmp_path / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(data)
    (tmp_path / "pair_compare.json").write_text(json.dumps({"bytes": recorded}))
    return tmp_path


def _rec(data: bytes) -> dict:
    return {"blake3": blake3.blake3(data).hexdigest()[:32], "bytes": len(data), "equal": True}


def test_committed_and_missing_artifacts(tmp_path):
    run = _run(
        tmp_path,
        {"a/x.jsonl": b"x\n", "d/one": b"1", "d/two": b"2"},
        {"a/x.jsonl": _rec(b"x\n"), "gone/y.jsonl": _rec(b"y"), "d/": {"equal": True, "files": 2}},
    )
    assert run_artifacts.main(["--run-dir", str(run)]) == 0
    lines = (run / "artifacts.sha256").read_text().splitlines()
    assert lines == [
        f"{hashlib.sha256(b'x\n').hexdigest()}  a/x.jsonl",
        f"{hashlib.sha256(b'1').hexdigest()}  d/one",
        f"{hashlib.sha256(b'2').hexdigest()}  d/two",
    ]
    doc = json.loads((run / "artifacts.json").read_text())
    assert (doc["committed"], doc["not_committed"]) == (2, 1)
    gone = next(r for r in doc["artifacts"] if r["path"] == "gone/y.jsonl")
    assert gone["committed"] is False and "sha256" not in gone


def test_committed_file_that_is_not_the_measured_one_fails(tmp_path):
    run = _run(tmp_path, {"a/x.jsonl": b"changed"}, {"a/x.jsonl": _rec(b"x\n")})
    with pytest.raises(SystemExit, match="differs from the pair record"):
        run_artifacts.main(["--run-dir", str(run)])
    assert not (run / "artifacts.sha256").exists()


def test_directory_with_another_file_count_fails(tmp_path):
    run = _run(tmp_path, {"d/one": b"1"}, {"d/": {"equal": True, "files": 2}})
    with pytest.raises(SystemExit, match="1 files committed, 2 recorded"):
        run_artifacts.main(["--run-dir", str(run)])
