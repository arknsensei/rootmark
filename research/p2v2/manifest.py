"""P2-V.2/Y0 (TASKS Y0; GOAL P2-V.2 Kapsam 1): the blake2b of every file of a
downloaded tier-2 artifact set (`gh run download <run> -n tier2-A -n tier2-B
-n tier2-compare -D reports/rounds/P2-V.2/ci/<run>/`), written to
`reports/rounds/P2-V.2/ci/<run>/manifest.json`.

Paths are relative to the run's directory (no host value). With `--check`
the manifest is recomputed and compared: a missing, extra or changed file
is an error (exit 3), never a silent skip.

    python -m p2v2.manifest <dir> [--run <id>] [--check]

(from `research/`, `PYTHONPATH=.`).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

NAME = "manifest.json"


def digest(p: Path) -> str:
    h = hashlib.blake2b()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def build(d: Path, run: str | None) -> dict:
    files = {}
    for p in sorted(d.rglob("*")):
        if p.is_file() and p.relative_to(d).as_posix() != NAME:
            files[p.relative_to(d).as_posix()] = {"blake2b": digest(p), "bytes": p.stat().st_size}
    if not files:
        raise SystemExit(f"error: no file under {d}")
    return {
        "run": run or d.name,
        "algorithm": "blake2b (hashlib.blake2b, 64-byte digest)",
        "n_files": len(files),
        "n_bytes": sum(v["bytes"] for v in files.values()),
        "files": files,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("dir", type=Path)
    ap.add_argument("--run")
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    m = build(a.dir, a.run)
    out = a.dir / NAME
    if a.check:
        old = json.loads(out.read_text(encoding="utf-8"))
        diff = sorted(
            k
            for k in set(old["files"]) | set(m["files"])
            if old["files"].get(k) != m["files"].get(k)
        )
        print(f"{a.dir}: {m['n_files']} files, {len(diff)} differ from {NAME}")
        for k in diff[:20]:
            print(f"  {k}")
        sys.exit(3 if diff else 0)
    out.write_text(json.dumps(m, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{out}: {m['n_files']} files, {m['n_bytes']} bytes")


if __name__ == "__main__":
    main()
