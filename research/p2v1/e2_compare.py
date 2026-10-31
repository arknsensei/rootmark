"""P2-V.1/E2 (GOAL P2-V.1 Kapsam 3, V1-2): compares the runners' halves of
`scripts/embed_cpu.sh` byte for byte.

Every kept file but `cpu.json` (the processor model: information) must
have the same bytes on every runner — the vector files, the index
manifests, the flat-mode search records with their cosine scores, the
corpus list and the build reports. The verdict names the processor models
and whether they differ (a green comparison of identical processors would
prove nothing about processors).

    python3 research/p2v1/e2_compare.py <dir of embed-cpu-* artifacts> --out e2.json

Exit 0 when every file is equal on every runner, 3 when one differs, 1 when
a runner or a file is missing. Standard library only.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

INFO = {"cpu.json", "hashes.json"}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("dir", type=Path)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--min-runners", type=int, default=3)
    a = ap.parse_args()
    runners = {}
    for d in sorted(a.dir.glob("embed-cpu-*")):
        cpu = json.loads((d / "cpu.json").read_text(encoding="utf-8"))
        files = {
            str(p.relative_to(d)): hashlib.blake2b(p.read_bytes()).hexdigest()
            for p in sorted(d.rglob("*"))
            if p.is_file() and p.name not in INFO
        }
        # the runner's own record of its hashes must match its files
        own = json.loads((d / "hashes.json").read_text(encoding="utf-8"))
        own = {k: v for k, v in own.items() if Path(k).name not in INFO}
        if own != files:
            print(f"error: {d.name}: hashes.json does not match its files", file=sys.stderr)
            return 1
        runners[cpu["label"]] = {"cpu": cpu, "files": files}
    if len(runners) < a.min_runners:
        print(f"error: {len(runners)} runners, expected {a.min_runners}", file=sys.stderr)
        return 1
    names = sorted(set().union(*(r["files"] for r in runners.values())))
    differing = {}
    for n in names:
        vals = {lab: r["files"].get(n) for lab, r in runners.items()}
        if None in vals.values() or len(set(vals.values())) != 1:
            differing[n] = vals
    models = sorted({r["cpu"]["cpu_model"] for r in runners.values()})
    res = {
        "task": "P2-V.1/E2",
        "runners": {lab: r["cpu"] for lab, r in sorted(runners.items())},
        "cpu_models": models,
        "processors_differ": len(models) == len(runners),
        "files": len(names),
        "files_compared": {n: next(iter(runners.values()))["files"].get(n) for n in names},
        "differing": differing,
        "equal": not differing,
    }
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(res, indent=1, sort_keys=True, ensure_ascii=False) + "\n")
    verdict = "equal" if res["equal"] else f"DIFFERENT ({len(differing)} files)"
    print(f"e2: {len(runners)} runners, {len(names)} files: {verdict}")
    for lab, r in sorted(runners.items()):
        print(f"  {lab}: {r['cpu']['arch']} {r['cpu']['cpu_model']}")
    print(f"  processors {'differ' if res['processors_differ'] else 'NOT all different'}")
    for n, v in sorted(differing.items()):
        print(f"  differs: {n} {v}")
    return 0 if res["equal"] else 3


if __name__ == "__main__":
    sys.exit(main())
