"""P2-0.1 run comparisons (TASKS Y6; H-P2-0.1i), with the frozen leaf
extraction and field comparison (`compare_runs.leaves`,
`metrics.max_relative_difference`) and the frozen comparators of earlier
rounds (`compare_p1e1`, `compare_p2_0`).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`): the P1-E.1 pair set
(`compare_p1e1.pair`: everything since P1-A, fields and bytes) and the
P2-0.1 result files (`P2_0_1_FILES`), every field except the wall-clock
ones (`P2_0_1_TIMING`: what the real-Obsidian sets and the editor sessions
measure — latencies, the false-unverified counts of timing-dependent saves —
declared in research/eval/DETERMINISTIC_FIELDS.md), and the D-A, D-A2 and
D-C vault logs byte for byte (`P2_0_1_BYTES`: `events.jsonl` and
`attestations.jsonl`, H-P2-0.1i). The real-Obsidian records
(`p2_0_1/d_b.json`, `d_a_prime.json`, `d_c_prime.json`) and the editor
sessions (`p2_0_1/d_b2.json`) are wall-clock records as a whole and are not
compared. No other exclusion.

`regress <p2_0_run> <run>`: the P1-E.1 pair set of files of the P2-0
official RUN against this RUN, every field (the earlier comparators' own
wall-clock and informational fields aside), **except** the subtrees of
`EXCLUDED` — each a difference that a P2-0.1 decision causes, written from a
development RUN before the official RUN (precedent `compare_p2_0.EXCLUDED`)
— and the conformance report by test id: the 37 tests of the reference
equal, C2-13 passed; spec 2.0.0 → 2.0.1 and 37 → 38 are the P2-0.1 decision
(K-P2-01.3). P2-0's own result files (`p2_0/*`, `p2_0_checks.json`) are not
in this RUN — their sets run as P2-0.1's (P2-0's D-B test set is never run
again, GOAL P2-0.1) — and are compared instead set by set: D-A
(`p2_0/d_a.json` ↔ `p2_0_1/d_a.json`, every field, and its `events.jsonl`
byte for byte) and D-E's three `v0.1.0` vaults (`p2_0/d_e.json` rows ↔ the
first three `p2_0_1/d_e.json` rows). A difference outside the exclusions
fails; every excluded leaf that differs is listed, and so is every
exclusion that matched no difference.

    python -m eval.compare_p2_0_1 pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2_0_1 regress <p2_0_run> <run> [--out path.json]

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
from eval import compare_p2_0 as Z
from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1c1 import PAIR_FILES as P1C1_FILES
from eval.datasets.common import dump_json, file_blake3

P2_0_1 = Path("p2_0_1")
P2_0_1_FILES = [
    "p2_0_1_checks.json",
    "p2_0_1/d_e.json",
    "p2_0_1/d_a.json",
    "p2_0_1/d_a2.json",
    "p2_0_1/d_c.json",
    "p2_0_1/tamper_p2_0_1.json",
]
# wall-clock fields of the P2-0.1 files (file, path pattern; "*" = any
# key/index): the measurements of the real-Obsidian sets and the editor
# sessions as recorded in p2_0_1_checks.json
P2_0_1_TIMING: dict[str, list[tuple]] = {
    "p2_0_1_checks.json": [
        ("H-P2-0.1b",),
        ("H-P2-0.1b′",),
        ("H-P2-0.1c",),
        ("H-P2-0.1d′",),
        ("H-P2-0.1e",),
        ("H-P2-0.1f",),
    ],
}
# informational fields of the P2-0.1 files: none
P2_0_1_INFO: dict[str, list[tuple]] = {}
P2_0_1_BYTES = [
    P2_0_1 / "d_a" / "events.jsonl",
    P2_0_1 / "d_a" / "attestations.jsonl",
    P2_0_1 / "d_a2" / "events.jsonl",
    P2_0_1 / "d_a2" / "attestations.jsonl",
    P2_0_1 / "d_c" / "events.jsonl",
    P2_0_1 / "d_c" / "attestations.jsonl",
]
CONFORMANCE = E.CONFORMANCE
NEW_TESTS = ["C2-13"]

# P2-0 official RUN → this RUN: subtrees whose difference a P2-0.1 decision
# causes, `(file, path prefix, decision, why)`; written from the development
# RUN before the official RUN (Y6 record)
EXCLUDED: list[tuple[str, tuple, str, str]] = [
    (
        "conformance_p1d1.json",
        ("fixtures_manifest_hash",),
        "K-P2-01.3",
        "spec 2.0.1 and conformance 38 (C2-13)",
    ),
    (
        "conformance_p1d1.json",
        ("passed",),
        "K-P2-01.3",
        "spec 2.0.1 and conformance 38 (C2-13)",
    ),
    (
        "conformance_p1d1.json",
        ("spec_version",),
        "K-P2-01.3",
        "spec 2.0.1 and conformance 38 (C2-13)",
    ),
]


def _match(pats: list[tuple], path: tuple) -> bool:
    return any(
        len(path) >= len(pat)
        and all(p == "*" or p == str(x) for p, x in zip(pat, path, strict=False))
        for pat in pats
    )


def _skip(name: str, path: tuple) -> bool:
    return (
        Z._skip(name, path)
        or _match(P2_0_1_TIMING.get(name, []), path)
        or _match(P2_0_1_INFO.get(name, []), path)
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
    """`compare_p2_0.compare_files` with the P2-0.1 skips."""
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


def _bytes(a: Path, b: Path, files: list[Path]) -> dict:
    out = {}
    for f in files:
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
    q = compare_files(a, b, P2_0_1_FILES, lambda _n, _p: None)
    res["files"] = [*res["files"], *P2_0_1_FILES]
    res["n_fields"] += q["n_fields"]
    res["differing_fields"].update(q["differing_fields"])
    pb = _bytes(a, b, P2_0_1_BYTES)
    res["bytes"].update(pb)
    res["h_p2_0_1i"] = {"files": pb, "equal": all(v["equal"] for v in pb.values())}
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def _json(p: Path) -> dict:
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


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
        and cb["spec_version"] == "2.0.1"
        and cb["passed"] == ca["passed"] + len(NEW_TESTS)
    )
    # P2-0's sets that P2-0.1 runs again unchanged: D-A and D-E's v0.1.0 rows
    da_ref, da_run = _json(ref / "p2_0" / "d_a.json"), _json(run / "p2_0_1" / "d_a.json")
    la, lb = C.leaves(da_ref), C.leaves(da_run)
    d_a_diff = sorted(C._key(p) for p in set(la) | set(lb) if la.get(p) != lb.get(p))
    ea = (ref / "p2_0" / "d_a" / "events.jsonl").read_bytes()
    eb = (run / "p2_0_1" / "d_a" / "events.jsonl").read_bytes()
    de_ref = _json(ref / "p2_0" / "d_e.json")["vaults"]
    de_run = _json(run / "p2_0_1" / "d_e.json")["vaults"][: len(de_ref)]
    sets = {
        "d_a": {"differing": d_a_diff[:50], "events_equal": ea == eb},
        "d_e_v0_1_0": {"equal": de_ref == de_run},
    }
    sets["equal"] = not d_a_diff and ea == eb and de_ref == de_run
    res["conformance"] = conf
    res["p2_0_sets"] = sets
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
    res["equal"] = not res["differing_fields"] and conf["equal"] and sets["equal"]
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
