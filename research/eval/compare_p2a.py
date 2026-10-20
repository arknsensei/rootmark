"""P2-A run comparisons (TASKS A5; GOAL P2-A A-8), with the frozen
comparators of earlier rounds (`compare_p2_0_2`: leaf extraction, field
comparison, the P1-E.1 pair set and the P2-0.2 files).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): `compare_p2_0_2.pair`
(everything since P1-A, fields and bytes; the P2-0.2 files with their
wall-clock fields aside) and the P2-A result files (`P2A_FILES`), every
field. No exclusion: P2-A's sets are deterministic (the vaults' temporary
paths are written as `<vault>`).

`regress <p2_0_2_run> <run>`: the files the P2-0.2 official RUN and this
RUN both hold — the P1-E.1 pair set and the P2-0.2 files — every field,
**except** the subtrees of `EXCLUDED`, each a difference a P2-A decision
causes, written from a development RUN before the official RUN (precedent
`compare_p2_0_2.EXCLUDED`); the conformance report by test id (the 39 tests
equal, spec 2.0.2: P2-A adds none); and the vault logs of D-A, D-A2 and
D-C byte for byte (`compare_p2_0_2.P2_0_2_BYTES`: the explicit recipe, the
(A) fix and `--attest-log` change none of them). A difference outside the
exclusions fails; every excluded leaf that differs is listed, and so is
every exclusion that matched no difference.

    python -m eval.compare_p2a pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2a regress <p2_0_2_run> <run> [--out path.json]

Exit code: 0 when nothing differs (outside the exclusions), 3 when
something does (the comparison is still written), 1 on a missing file or
field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_p1d as P
from eval import compare_p1d1 as D1
from eval import compare_p1e as E
from eval import compare_p1e1 as E1
from eval import compare_p2_0_2 as X
from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1c1 import PAIR_FILES as P1C1_FILES
from eval.datasets.common import dump_json

P2A_FILES = [
    "p2a_checks.json",
    "p2a/migrate_old.json",
    "p2a/fixture.json",
    "p2a/tamper.json",
    "p2a/tests.json",
]
SPEC = "2.0.2"

# P2-0.2 official RUN → this RUN: `(file, path prefix, decision, why)`;
# written from the development RUN before the official RUN (A5.md)
EXCLUDED: list[tuple[str, tuple, str, str]] = []


def _excl_match(name: str, path: tuple) -> tuple | None:
    for e in EXCLUDED:
        f, pre = e[0], e[1]
        if (
            f == name
            and len(path) >= len(pre)
            and all(p == "*" or p == str(x) for p, x in zip(pre, path, strict=False))
        ):
            return e
    return None


def compare_files(a: Path, b: Path, files: list[str], exclude) -> dict:
    """`compare_p2_0_2.compare_files` with this module's exclusions."""
    n = 0
    differing: dict = {}
    excluded: dict = {}
    for name in files:
        la = {p: v for p, v in C.leaves(P._load(a, name)).items() if not X._skip(name, p)}
        lb = {p: v for p, v in C.leaves(P._load(b, name)).items() if not X._skip(name, p)}
        for p in sorted(set(la) | set(lb), key=C._key):
            e = exclude(name, p)
            va, vb = la.get(p, "absent"), lb.get(p, "absent")
            if e is not None:
                if va != vb:
                    k = f"{e[0]}:{'/'.join(map(str, e[1]))}"
                    excluded.setdefault(k, {"decision": e[2], "leaves": 0})
                    excluded[k]["leaves"] += 1
                la.pop(p, None)
                lb.pop(p, None)
                continue
            if p not in la or p not in lb:
                differing[f"{name}:{C._key(p)}"] = {"a": va, "b": vb}
                la.pop(p, None)
                lb.pop(p, None)
        keys = sorted(la, key=C._key)
        fa = {C._key(p): la[p] for p in keys}
        fb = {C._key(p): lb[p] for p in keys}
        r = M.max_relative_difference(fa, fb, sorted(fa))
        n += r["n_fields"]
        for k, v in r["differing_fields"].items():
            differing[f"{name}:{k}"] = v
    return {"n_fields": n, "differing_fields": differing, "excluded_differences": excluded}


def pair(a: Path, b: Path) -> dict:
    res = X.pair(a, b)
    q = compare_files(a, b, P2A_FILES, lambda _n, _p: None)
    res["files"] = [*res["files"], *P2A_FILES]
    res["n_fields"] += q["n_fields"]
    res["differing_fields"].update(q["differing_fields"])
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def regress(ref: Path, run: Path) -> dict:
    files = [
        *P1C1_FILES,
        *P.P1D_FILES,
        *D1.P1D1_FILES,
        *E.P1E_FILES,
        *E1.P1E1_FILES,
        *X.P2_0_2_FILES,
    ]
    res = compare_files(ref, run, files, _excl_match)
    ca, cb = P._load(ref, X.CONFORMANCE), P._load(run, X.CONFORMANCE)
    by_a = {t["id"]: t for t in P._tests(ca)}
    by_b = {t["id"]: t for t in P._tests(cb)}
    conf = {
        "tests": len(by_b),
        "reference_tests": len(by_a),
        "differing_tests": sorted(i for i in set(by_a) | set(by_b) if by_a.get(i) != by_b.get(i)),
        "spec_version": [ca["spec_version"], cb["spec_version"]],
        "level_claimed": [ca["level_claimed"], cb["level_claimed"]],
        "passed": [ca["passed"], cb["passed"]],
    }
    conf["equal"] = (
        not conf["differing_tests"]
        and ca["level_claimed"] == cb["level_claimed"] == 3
        and ca["spec_version"] == cb["spec_version"] == SPEC
        and ca["passed"] == cb["passed"]
    )
    bts = X._bytes(ref, run, X.P2_0_2_BYTES)
    res["conformance"] = conf
    res["p2_0_2_bytes"] = {"files": bts, "equal": all(v["equal"] for v in bts.values())}
    res["reference_run"] = ref.name
    res["files"] = files
    keyed = [f"{e[0]}:{'/'.join(map(str, e[1]))}" for e in EXCLUDED]
    res["exclusions"] = [
        {"file": e[0], "path": "/".join(map(str, e[1])), "decision": e[2], "why": e[3]}
        for e in EXCLUDED
    ]
    res["exclusions_without_difference"] = [
        k for k in keyed if k not in res["excluded_differences"]
    ]
    res["equal"] = not res["differing_fields"] and conf["equal"] and res["p2_0_2_bytes"]["equal"]
    return res


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("mode", choices=["pair", "regress"])
    ap.add_argument("a", type=Path)
    ap.add_argument("b", type=Path)
    ap.add_argument("--out", type=Path)
    x = ap.parse_args(argv)
    try:
        res = pair(x.a, x.b) if x.mode == "pair" else regress(x.a, x.b)
    except (FileNotFoundError, M.MetricError, json.JSONDecodeError) as e:
        print(f"error: {e}", file=sys.stderr)
        sys.exit(1)
    text = dump_json(C._finite(res))
    if x.out:
        x.out.write_text(text)
    print(text, end="")
    if not res["equal"]:
        print(f"{x.mode} FAILED: differing {sorted(res['differing_fields'])}", file=sys.stderr)
        sys.exit(C.NOT_EQUAL_EXIT)


if __name__ == "__main__":
    main()
