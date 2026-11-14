"""D-1 — the transfer tool's command line (research/, not runtime code).

From research/:
  python3 -E -s -m transfer.cli build --ref <tag|commit> --out <dir> [--tag <vX.Y.Z>] [--repo <lab>]
  python3 -E -s -m transfer.cli verify --product <checkout> --lab-commit <sha> [--repo <lab>]
      [--expect-tree <id> …]

`build` writes <out>/tree, <out>/tree.git and <out>/transfer_result.json and
exits 1 when the transfer is not clean (classification, overlays, leaks).
`verify` (DESIGN §2.2) rebuilds the tree from the lab commit and checks:
the rebuilt tree id = every --expect-tree (e.g. the product commit's tree and
its Transfer-Tree line); every `lab:` row of the product's TRANSFER.json is
the lab commit's blob (and for = and K rows the product file's bytes are
that blob's), every `overlay:` row is the overlay's bytes; no product file
outside TRANSFER.json.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

from transfer import build as B

REPO = Path(__file__).resolve().parents[2]


def cmd_build(a) -> int:
    try:
        r = B.build(Path(a.repo), a.ref, Path(a.out), a.tag, a.config_ref)
    except (B.TransferError, Exception) as e:  # noqa: BLE001 — every failure is reported, exit 1
        print(f"transfer: {e}", file=sys.stderr)
        return 1
    summary = {
        "tree": r["tree"],
        "lab_commit": r["lab_commit"],
        "lab_tag": r["lab_tag"],
        "product_files": r["product_files"],
        "counts": r["counts"],
        "leaks": r["leaks"]["counts"],
        "errors": r["classification_errors"],
        "ok": r["ok"],
    }
    print(json.dumps(summary, ensure_ascii=False))
    if not r["ok"]:
        for e in r["classification_errors"]:
            print(f"transfer: {e}", file=sys.stderr)
        for k, v in r["leaks"]["files"].items():
            print(f"transfer: leak {k}: {v[:10]}", file=sys.stderr)
    return 0 if r["ok"] else 1


def cmd_verify(a) -> int:
    out = Path(a.out)
    problems = []
    try:
        r = B.build(Path(a.repo), a.lab_commit, out, a.tag)
    except Exception as e:  # noqa: BLE001
        print(f"transfer verify: rebuild failed: {e}", file=sys.stderr)
        return 1
    if not r["ok"]:
        problems.append("the rebuilt transfer is not clean (see transfer_result.json)")
    for t in a.expect_tree or []:
        if t != r["tree"]:
            problems.append(f"tree {r['tree']} != expected {t}")
    prod = Path(a.product)
    man = json.loads((prod / "TRANSFER.json").read_text(encoding="utf-8"))
    if man.get("lab_commit") != r["lab_commit"]:
        problems.append(f"TRANSFER.json lab_commit {man.get('lab_commit')} != {r['lab_commit']}")
    commit = B.Commit(Path(a.repo), a.lab_commit)
    listed = set()
    for row in man["files"]:
        p = row["path"]
        listed.add(p)
        f = prod / p
        if not f.is_file():
            problems.append(f"{p}: in TRANSFER.json, not in the product tree")
            continue
        data = f.read_bytes()
        if hashlib.sha256(data).hexdigest() != row["sha256"]:
            problems.append(f"{p}: sha256 differs from TRANSFER.json")
        kind, rest = row["origin"].split(":", 1)
        src, blob = rest.rsplit("@", 1)
        if src not in commit.entries or commit.entries[src][1] != blob:
            problems.append(f"{p}: origin {row['origin']} is not the lab commit's blob")
            continue
        if (kind == "overlay" or row["transform"] in ("=", "K")) and commit.read(src) != data:
            problems.append(f"{p}: bytes differ from {row['origin']}")
    tracked = (
        subprocess.run(["git", "-C", str(prod), "ls-files", "-z"], capture_output=True, check=True)
        .stdout.decode("utf-8")
        .split("\0")
    )
    extra = sorted({x for x in tracked if x} - listed - {"TRANSFER.json"})
    if extra:
        problems.append(f"{len(extra)} product files outside TRANSFER.json: {extra[:10]}")
    res = {
        "tree": r["tree"],
        "expected": a.expect_tree or [],
        "rows": len(man["files"]),
        "problems": problems,
        "ok": not problems,
    }
    print(json.dumps(res, ensure_ascii=False))
    return 0 if not problems else 1


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(prog="transfer")
    sub = ap.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build")
    b.add_argument("--ref", required=True)
    b.add_argument("--out", required=True)
    b.add_argument("--tag", default=None)
    b.add_argument(
        "--config-ref",
        default=None,
        help="lists and overlays from this commit (standalone check of an older tree only)",
    )
    b.add_argument("--repo", default=str(REPO))
    v = sub.add_parser("verify")
    v.add_argument("--product", required=True)
    v.add_argument("--lab-commit", required=True)
    v.add_argument("--tag", default=None)
    v.add_argument("--out", required=True)
    v.add_argument("--repo", default=str(REPO))
    v.add_argument("--expect-tree", action="append")
    a = ap.parse_args(argv)
    return cmd_build(a) if a.cmd == "build" else cmd_verify(a)


if __name__ == "__main__":
    sys.exit(main())
