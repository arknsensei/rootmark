"""P1-E.1 run comparisons (GOAL P1-E.1 E1-6, E1-7), with the frozen leaf
extraction and field comparison (`compare_runs.leaves`,
`metrics.max_relative_difference`) and the frozen P1-E comparator
(`compare_p1e`).

`pair <run_A> <run_B>` (E1-7, `scripts/verify.sh pair`): the P1-E pair set
(`compare_p1e.pair`: everything since P1-A, fields and bytes) and the P1-E.1
result files (`P1E1_FILES`), every field except the contract- or build-
nondeterministic ones (`P1E1_INFO`) and the wall-clock ones (`P1E1_TIMING`;
both declared in research/eval/DETERMINISTIC_FIELDS.md), and the P1-E.1
artefacts byte for byte (`P1E1_BYTES`). No other exclusion. A real-Obsidian
record that says `skipped` (no Obsidian binary) compares as such: two
skipped records are equal, a skipped and a run one are not.

`regress <p1e_run> <run>` (E1-6, "P1-E kalite alanları"): the P1-E pair set
of files (P1-C.1 set + P1-D, P1-D.1 and P1-E files) of the P1-E official
RUN against this RUN, every field (the earlier comparators' own wall-clock
and informational fields aside), **except** the subtrees of `EXCLUDED` —
each a difference that a P1-E.1 decision causes (K-E1.2, F2, F4), written
from a development RUN before the official RUN (precedent K-C1.6,
compare_p1e.EXCLUDED) — and the conformance report by test id
(`conformance_p1d1.json`, 34 tests). A difference outside the exclusions
fails; every excluded leaf that differs is listed, and so is every
exclusion that matched no difference.

    python -m eval.compare_p1e1 pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p1e1 regress <p1e_run> <run> [--out path.json]

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
from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1c1 import PAIR_FILES as P1C1_FILES
from eval.datasets.common import dump_json, file_blake3

P1E1 = Path("p1e1")
P1E1_FILES = [
    "p1e1_checks.json",
    "p1e1/status/e1_1.json",
    "p1e1/status/e1_2.json",
    "p1e1/contract.json",
    "p1e1/tamper_p1e1.json",
    "p1e1/tamper_p1e1_e2e.json",
    "p1e1/release/release.json",
    "p1e1/release/release_check.json",
    "p1e1/release/plugin_e2e_real.json",
    "p1e1/release/access_log_real.json",
    "p1e1/historical.json",
    "p1e1/check_compare_fields.json",
]
# wall-clock fields of the P1-E.1 files (file, path pattern; "*" = any key/index)
P1E1_TIMING: dict[str, list[tuple]] = {}
# fields that depend on the build environment, not on the commit: the
# sha256 of a release binary (absolute paths of the build directory may be
# embedded; RELEASE.md human step 3 compares hashes across machines only as
# a record)
P1E1_INFO: dict[str, list[tuple]] = {
    "p1e1/release/release.json": [("targets", "*", "sha256")],
}
P1E1_BYTES = [
    P1E1 / "release" / "plugin_e2e_real.json",
    P1E1 / "release" / "access_log_real.json",
]
CONFORMANCE = E.CONFORMANCE

# P1-E official RUN → this RUN: subtrees whose difference a P1-E.1 decision
# causes, `(file, path prefix, decision, why)`; written from the development
# RUN before the official RUN (E1.5 record; precedent K-C1.6, compare_p1e)
EXCLUDED: list[tuple[str, tuple, str, str]] = []


def _match(pats: list[tuple], path: tuple) -> bool:
    return any(
        len(path) >= len(pat)
        and all(p == "*" or p == str(x) for p, x in zip(pat, path, strict=False))
        for pat in pats
    )


def _skip(name: str, path: tuple) -> bool:
    return (
        E._skip(name, path)
        or _match(P1E1_TIMING.get(name, []), path)
        or _match(P1E1_INFO.get(name, []), path)
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
    """`compare_p1e.compare_files` with the P1-E.1 skips."""
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


def p1e1_bytes(a: Path, b: Path) -> dict:
    out = {}
    for f in P1E1_BYTES:
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
    res = E.pair(a, b)
    q = compare_files(a, b, P1E1_FILES, lambda _n, _p: None)
    res["files"] = [*res["files"], *P1E1_FILES]
    res["n_fields"] += q["n_fields"]
    res["differing_fields"].update(q["differing_fields"])
    res["bytes"].update(p1e1_bytes(a, b))
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def regress(ref: Path, run: Path) -> dict:
    files = [*P1C1_FILES, *P.P1D_FILES, *D1.P1D1_FILES, *E.P1E_FILES]
    res = compare_files(ref, run, files, _excl_match)
    ca, cb = P._load(ref, CONFORMANCE), P._load(run, CONFORMANCE)
    by_a = {t["id"]: t for t in P._tests(ca)}
    by_b = {t["id"]: t for t in P._tests(cb)}
    conf = {
        "tests": len(by_b),
        "differing_tests": sorted(i for i in set(by_a) | set(by_b) if by_a.get(i) != by_b.get(i)),
        "spec_version": [ca["spec_version"], cb["spec_version"]],
        "level_claimed": [ca["level_claimed"], cb["level_claimed"]],
        "passed": [ca["passed"], cb["passed"]],
    }
    conf["equal"] = (
        not conf["differing_tests"]
        and ca["level_claimed"] == cb["level_claimed"]
        and ca["spec_version"] == cb["spec_version"]
        and ca["passed"] == cb["passed"]
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
