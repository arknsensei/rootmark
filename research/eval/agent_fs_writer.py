"""The agent of D-A and D-C (GOAL P2-0: `agent_fs_writer`; TASKS Z5): writes
a vault's files directly, the way an agent with file access does (Claude
Code's Write/Edit tools): no MCP, no HTTP, no attestation. It never touches
`.rootmark/`. One step is `{"op": "write", "path", "text"}` (create or
replace the file, parent directories made) or `{"op": "rename", "path",
"to"}`.

GOAL names it `scripts/agent_fs_writer.py`; Python lives only in
`research/` (root CLAUDE.md), so it is `research/eval/agent_fs_writer.py`
(ASSUMPTIONS Z5-1).

    python -m eval.agent_fs_writer --vault <dir> --steps <json file>
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path


def _target(vault: Path, rel: str) -> Path:
    parts = Path(rel).parts
    if Path(rel).is_absolute() or ".." in parts or any(p == ".rootmark" for p in parts):
        raise ValueError(f"agent step outside the vault's documents: {rel!r}")
    return vault / rel


def apply_step(vault: Path, step: dict) -> None:
    """Applies one file-system step to `vault`."""
    op = step["op"]
    if op == "write":
        p = _target(vault, step["path"])
        p.parent.mkdir(parents=True, exist_ok=True)
        # one write call, as an editor tool does (not an atomic rename)
        with open(p, "wb") as f:
            f.write(step["text"].encode("utf-8"))
            f.flush()
            os.fsync(f.fileno())
    elif op == "rename":
        src, dst = _target(vault, step["path"]), _target(vault, step["to"])
        dst.parent.mkdir(parents=True, exist_ok=True)
        os.rename(src, dst)
    else:
        raise ValueError(f"unknown agent step {op!r}")


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--vault", required=True, type=Path)
    ap.add_argument("--steps", required=True, type=Path)
    a = ap.parse_args(argv)
    for s in json.loads(a.steps.read_text(encoding="utf-8")):
        apply_step(a.vault, s)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
