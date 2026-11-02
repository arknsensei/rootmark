"""P2-V.2 (GOAL V2-5; FREEZE_EXCEPTIONS F1–F3): byte comparison of tool
outputs against their committed references.

    python -m p2v2.before_after --pair <out file> <reference file> [--pair …] [--echo <field> …] [--out <json>]

Each pair is compared byte for byte. When the bytes differ, the two JSON
documents are compared again with the `--echo` fields (top-level keys that
only echo an input path given on the command line, e.g. `tier2 pair`'s
`frozen_result_read`, `compare_p2r pair`'s `run_a`/`run_b`) replaced by the
reference's value: equal then → `equal_but_echo` with the differing
fields named; otherwise `different`. Exit 0 when no pair is `different`,
3 otherwise; a missing file is an error (exit 1).
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def compare(out: Path, ref: Path, echo: list[str]) -> dict:
    for p in (out, ref):
        if not p.is_file():
            raise FileNotFoundError(f"{p} missing")
    a, b = out.read_bytes(), ref.read_bytes()
    row = {"out": str(out), "reference": str(ref), "bytes": [len(a), len(b)]}
    if a == b:
        return {**row, "result": "byte_equal"}
    try:
        x, y = json.loads(a), json.loads(b)
    except json.JSONDecodeError:
        return {**row, "result": "different"}
    fields = [k for k in echo if isinstance(x, dict) and x.get(k) != y.get(k)]
    for k in fields:
        x[k] = y.get(k)
    if x == y:
        # the rest re-serialised exactly as the reference
        return {**row, "result": "equal_but_echo", "echo_fields": fields}
    return {**row, "result": "different"}


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--pair", nargs=2, type=Path, action="append", required=True)
    ap.add_argument("--echo", action="append", default=[])
    ap.add_argument("--out", type=Path)
    a = ap.parse_args(argv)
    try:
        rows = [compare(o, r, a.echo) for o, r in a.pair]
    except FileNotFoundError as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    res = {
        "echo_fields_allowed": a.echo,
        "pairs": rows,
        "byte_equal": sum(r["result"] == "byte_equal" for r in rows),
        "equal_but_echo": sum(r["result"] == "equal_but_echo" for r in rows),
        "different": sum(r["result"] == "different" for r in rows),
    }
    text = json.dumps(res, indent=1, ensure_ascii=False) + "\n"
    if a.out:
        a.out.write_text(text, encoding="utf-8")
    print(text, end="")
    sys.exit(3 if res["different"] else 0)


if __name__ == "__main__":
    main()
