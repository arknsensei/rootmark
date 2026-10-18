"""P2-0 run comparisons (TASKS Z7; H-P2-0g), with the frozen leaf extraction
and field comparison (`compare_runs.leaves`, `metrics.max_relative_difference`)
and the frozen P1-E.1 comparator (`compare_p1e1`).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): the P1-E.1 pair set
(`compare_p1e1.pair`: everything since P1-A, fields and bytes) and the P2-0
result files (`P2_0_FILES`), every field except the wall-clock ones
(`P2_0_TIMING`: what the real-Obsidian D-B sessions measure — the
false-unverified rate, late raises, latency — and the records of them;
declared in research/eval/DETERMINISTIC_FIELDS.md), and the D-A and D-C
vault logs byte for byte (`P2_0_BYTES`: `events.jsonl` and
`attestations.jsonl`, H-P2-0g). `p2_0/d_b.json` is a wall-clock record as a
whole and is not compared. No other exclusion.

`regress <p1e1_run> <run>`: the P1-E.1 pair set of files (P1-C.1 set +
P1-D, P1-D.1, P1-E and P1-E.1 files) of the P1-E.1 official RUN against
this RUN, every field (the earlier comparators' own wall-clock and
informational fields aside), **except** the subtrees of `EXCLUDED` — each a
difference that a P2-0 decision causes, written from a development RUN
before the official RUN (precedent K-C1.6, compare_p1e1.EXCLUDED) — and
the conformance report by test id: the 34 tests of the reference equal,
the three new ones (C2-11, C2-12, C3-08) passed; spec 1.0.3 → 2.0.0 and 34
→ 37 are the P2-0 decision (K-P2-0.1). A difference outside the exclusions
fails; every excluded leaf that differs is listed, and so is every
exclusion that matched no difference.

    python -m eval.compare_p2_0 pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2_0 regress <p1e1_run> <run> [--out path.json]

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
from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1c1 import PAIR_FILES as P1C1_FILES
from eval.datasets.common import dump_json, file_blake3

P2_0 = Path("p2_0")
P2_0_FILES = [
    "p2_0_checks.json",
    "p2_0/tuning.json",
    "p2_0/d_e.json",
    "p2_0/d_a.json",
    "p2_0/d_c.json",
    "p2_0/tamper_p2_0.json",
]
# wall-clock fields of the P2-0 files (file, path pattern; "*" = any key/index):
# the real-Obsidian D-B measurements (H-P2-0b, b′, c) as recorded in
# p2_0_checks.json, and the D-B tuning rows of the committed tuning record
# (copied, but measured on a wall clock)
P2_0_TIMING: dict[str, list[tuple]] = {
    "p2_0_checks.json": [("H-P2-0b",), ("H-P2-0b′",), ("H-P2-0c",)],
    "p2_0/tuning.json": [("windows",)],
}
P2_0_INFO: dict[str, list[tuple]] = {}
P2_0_BYTES = [
    P2_0 / "d_a" / "events.jsonl",
    P2_0 / "d_a" / "attestations.jsonl",
    P2_0 / "d_c" / "events.jsonl",
    P2_0 / "d_c" / "attestations.jsonl",
]
CONFORMANCE = E.CONFORMANCE
NEW_TESTS = ["C2-11", "C2-12", "C3-08"]

# P1-E.1 official RUN → this RUN: subtrees whose difference a P2-0 decision
# causes, `(file, path prefix, decision, why)`; written from the development
# RUN before the official RUN (Z7 record)
EXCLUDED: list[tuple[str, tuple, str, str]] = []


def _match(pats: list[tuple], path: tuple) -> bool:
    return any(
        len(path) >= len(pat)
        and all(p == "*" or p == str(x) for p, x in zip(pat, path, strict=False))
        for pat in pats
    )


def _skip(name: str, path: tuple) -> bool:
    return (
        E1._skip(name, path)
        or _match(P2_0_TIMING.get(name, []), path)
        or _match(P2_0_INFO.get(name, []), path)
    )


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
    """`compare_p1e1.compare_files` with the P2-0 skips."""
    n = 0
    differing: dict = {}
    excluded: dict = {}
    for name in files:
        la = {p: v for p, v in C.leaves(P._load(a, name)).items() if not _skip(name, p)}
        lb = {p: v for p, v in C.leaves(P._load(b, name)).items() if not _skip(name, p)}
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


def p2_0_bytes(a: Path, b: Path) -> dict:
    out = {}
    for f in P2_0_BYTES:
        pa, pb = a / f, b / f
        for p in (pa, pb):
            if not p.is_file():
                raise FileNotFoundError(f"{p} missing")
        out[f.as_posix()] = {
            "blake3": file_blake3(pa),
            "bytes": pa.stat().st_size,
            "equal": pa.read_bytes() == pb.read_bytes(),
        }
    return out


def pair(a: Path, b: Path) -> dict:
    res = E1.pair(a, b)
    q = compare_files(a, b, P2_0_FILES, lambda _n, _p: None)
    res["files"] = [*res["files"], *P2_0_FILES]
    res["n_fields"] += q["n_fields"]
    res["differing_fields"].update(q["differing_fields"])
    pb = p2_0_bytes(a, b)
    res["bytes"].update(pb)
    res["h_p2_0g"] = {"files": pb, "equal": all(v["equal"] for v in pb.values())}
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def regress(ref: Path, run: Path) -> dict:
    files = [*P1C1_FILES, *P.P1D_FILES, *D1.P1D1_FILES, *E.P1E_FILES, *E1.P1E1_FILES]
    res = compare_files(ref, run, files, _excl_match)
    ca, cb = P._load(ref, CONFORMANCE), P._load(run, CONFORMANCE)
    by_a = {t["id"]: t for t in P._tests(ca)}
    by_b = {t["id"]: t for t in P._tests(cb)}
    new = {i: by_b.get(i) for i in NEW_TESTS}
    conf = {
        "tests": len(by_b),
        "reference_tests": len(by_a),
        "differing_tests": sorted(i for i in by_a if by_a[i] != by_b.get(i)),
        "unexpected_new_tests": sorted(set(by_b) - set(by_a) - set(NEW_TESTS)),
        "new_tests_passed": {i: (t or {}).get("result") == "pass" for i, t in new.items()},
        "spec_version": [ca["spec_version"], cb["spec_version"]],
        "level_claimed": [ca["level_claimed"], cb["level_claimed"]],
        "passed": [ca["passed"], cb["passed"]],
    }
    conf["equal"] = (
        not conf["differing_tests"]
        and not conf["unexpected_new_tests"]
        and all(v is True for v in conf["new_tests_passed"].values())
        and ca["level_claimed"] == cb["level_claimed"] == 3
        and cb["spec_version"] == "2.0.0"
        and cb["passed"] == ca["passed"] + len(NEW_TESTS)
    )
    res["conformance"] = conf
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
    res["equal"] = not res["differing_fields"] and conf["equal"]
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
