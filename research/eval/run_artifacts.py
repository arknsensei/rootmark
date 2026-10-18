"""Hashes of a RUN's byte artefacts (P2-0/Z0; P1-E.1 verification,
recommendation 3).

`verify.sh pair` compares the byte artefacts of two RUNs and records each
one's BLAKE3 (first 32 hex) and size under `bytes` of `pair_compare.json`.
Some of them (the MCP e2e transcripts and vault of `p1d/mcp_e2e/run1/`)
were never committed with the official RUN, so a later reader could not
check them. This writes, next to `pair_compare.json`:

- `artifacts.sha256`: `<sha256>  <path>` (the `shasum -a 256 -c` format) for
  every byte artefact present in the RUN directory, in path order;
- `artifacts.json`: every byte artefact of `pair_compare.json` with its
  recorded `blake3` and `bytes`, whether it is committed (present), and its
  `sha256` when present; a directory the pair compared file by file has its
  recorded `files` count and the `sha256` of each file when present.

A present artefact whose BLAKE3 or size differs from the pair record is an
error (the committed file is not the measured one): nothing is written.

    python -m eval.run_artifacts --run-dir <RUN dir>
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

import blake3

from .compare_runs import resolve


def artefacts(run_dir: Path) -> list[dict]:
    pair = run_dir / "pair_compare.json"
    if not pair.is_file():
        raise SystemExit(f"run_artifacts: {pair} missing (a RUN of `verify.sh pair`)")
    recorded = json.loads(pair.read_text())["bytes"]
    if not recorded:
        raise SystemExit(f"run_artifacts: {pair} lists no byte artefact")
    rows = []
    for name in sorted(recorded):
        rec = recorded[name]
        if not rec.get("equal"):
            raise SystemExit(f"run_artifacts: {name}: the pair did not agree on it")
        p = resolve(run_dir, name)
        if "blake3" not in rec:
            # a directory compared file by file (`files` = its file count)
            files = sorted(f for f in p.rglob("*") if f.is_file()) if p.is_dir() else []
            row = {"path": name, "files": rec["files"], "committed": p.is_dir()}
            if p.is_dir():
                if len(files) != rec["files"]:
                    raise SystemExit(
                        f"run_artifacts: {name}: {len(files)} files committed, {rec['files']} recorded"
                    )
                row["sha256"] = {
                    str(f.relative_to(run_dir)): hashlib.sha256(f.read_bytes()).hexdigest()
                    for f in files
                }
            rows.append(row)
            continue
        # older pair records carry no size
        row = {
            "path": name,
            "blake3": rec["blake3"],
            "bytes": rec.get("bytes"),
            "committed": p.is_file(),
        }
        if p.is_file():
            data = p.read_bytes()
            b3 = blake3.blake3(data).hexdigest()[:32]
            if b3 != rec["blake3"] or rec.get("bytes", len(data)) != len(data):
                raise SystemExit(
                    f"run_artifacts: {name}: committed file differs from the pair record "
                    f"(blake3 {b3}, {len(data)} B; recorded {rec['blake3']}, {rec['bytes']} B)"
                )
            row["file"] = str(p.relative_to(run_dir))
            row["sha256"] = hashlib.sha256(data).hexdigest()
        rows.append(row)
    return rows


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--run-dir", required=True, type=Path)
    a = ap.parse_args(argv)
    rows = artefacts(a.run_dir)
    hashed: dict[str, str] = {}
    for r in rows:
        if not r["committed"]:
            continue
        if isinstance(r["sha256"], dict):
            hashed.update(r["sha256"])
        else:
            hashed[r["file"]] = r["sha256"]
    lines = [f"{h}  {f}\n" for f, h in sorted(hashed.items())]
    (a.run_dir / "artifacts.sha256").write_text("".join(lines))
    doc = {
        "source": "pair_compare.json bytes (blake3 = first 32 hex of BLAKE3)",
        "artifacts": rows,
        "committed": sum(r["committed"] for r in rows),
        "not_committed": sum(not r["committed"] for r in rows),
    }
    (a.run_dir / "artifacts.json").write_text(json.dumps(doc, indent=2, ensure_ascii=False) + "\n")
    print(
        f"run_artifacts: {a.run_dir.name}: {doc['committed']} committed (sha256), "
        f"{doc['not_committed']} not committed (blake3 from the pair record)",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
