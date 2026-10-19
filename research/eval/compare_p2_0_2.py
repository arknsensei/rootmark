"""P2-0.2 run comparisons (TASKS X5; H-P2-0.2i), with the frozen leaf
extraction and field comparison (`compare_runs.leaves`,
`metrics.max_relative_difference`) and the frozen comparators of earlier
rounds (`compare_p1e1`, `compare_p2_0`, `compare_p2_0_1`).

`pair <run_A> <run_B>` (`scripts/verify.sh pair`, also with
`VERIFY_LOAD=1`): the P1-E.1 pair set (`compare_p1e1.pair`: everything since
P1-A, fields and bytes) and the P2-0.2 result files (`P2_0_2_FILES`), every
field except the wall-clock ones (`P2_0_2_TIMING`: what the real-Obsidian
sets and the editor sessions measure, as recorded in `p2_0_2_checks.json`;
declared in research/eval/DETERMINISTIC_FIELDS.md), and the D-A, D-A2 and
D-C vault logs byte for byte (`P2_0_2_BYTES`: `events.jsonl` and
`attestations.jsonl`, H-P2-0.2i). The real-Obsidian records
(`p2_0_2/d_b.json`, `d_a_prime.json`, `d_c_prime.json`) and the editor
sessions (`p2_0_2/d_b2.json`) are wall-clock records as a whole and are not
compared. No other exclusion; the comparator is not loosened (GOAL T3).

`regress <p2_0_1_run> <run>`: the P1-E.1 pair set of files of the P2-0.1
official RUN against this RUN, every field (the earlier comparators' own
wall-clock and informational fields aside), **except** the subtrees of
`EXCLUDED` — each a difference that a P2-0.2 decision causes, written from a
development RUN before the official RUN (precedent `compare_p2_0_1.EXCLUDED`)
— and the conformance report by test id: the 38 tests of the reference
equal, C2-14 passed; spec 2.0.1 → 2.0.2 and 38 → 39 are the P2-0.2 decision
(K-P2-02.3). P2-0.1's own result files (`p2_0_1/*`, `p2_0_1_checks.json`)
are not in this RUN — its sets run as P2-0.2's (P2-0.1's D-B test set is
never run again, GOAL P2-0.2) — and are compared set by set: D-A and D-A2
(`p2_0_1/<set>.json` ↔ `p2_0_2/<set>.json`, every field, and their
`events.jsonl` byte for byte), D-E's five rows (the first five
`p2_0_2/d_e.json` rows) and D-C, whose ticks T3 orders (X2): its log is the
reference's as a multiset of events without `seq`, and its attestation
log the reference's but `matched: false` on the collide lines (X2-1). A
difference outside the exclusions fails; every excluded leaf that differs
is listed, and so is every exclusion that matched no difference.

    python -m eval.compare_p2_0_2 pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p2_0_2 regress <p2_0_1_run> <run> [--out path.json]

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

P2_0_2 = Path("p2_0_2")
P2_0_2_FILES = [
    "p2_0_2_checks.json",
    "p2_0_2/rules.json",
    "p2_0_2/d_e.json",
    "p2_0_2/d_a.json",
    "p2_0_2/d_a2.json",
    "p2_0_2/d_c.json",
    "p2_0_2/tamper_p2_0_2.json",
]
# wall-clock fields of the P2-0.2 files (file, path pattern; "*" = any
# key/index): the measurements of the real-Obsidian sets and the editor
# sessions as recorded in p2_0_2_checks.json
P2_0_2_TIMING: dict[str, list[tuple]] = {
    "p2_0_2_checks.json": [
        ("H-P2-0.2b",),
        ("H-P2-0.2b′",),
        ("H-P2-0.2c",),
        ("H-P2-0.2d", "d_c_prime"),
        ("H-P2-0.2d", "pass"),
        ("H-P2-0.2e",),
        ("H-P2-0.2f",),
    ],
}
# informational fields of the P2-0.2 files: none
P2_0_2_INFO: dict[str, list[tuple]] = {}
P2_0_2_BYTES = [
    P2_0_2 / "d_a" / "events.jsonl",
    P2_0_2 / "d_a" / "attestations.jsonl",
    P2_0_2 / "d_a2" / "events.jsonl",
    P2_0_2 / "d_a2" / "attestations.jsonl",
    P2_0_2 / "d_c" / "events.jsonl",
    P2_0_2 / "d_c" / "attestations.jsonl",
]
CONFORMANCE = E.CONFORMANCE
NEW_TESTS = ["C2-14"]
SPEC = "2.0.2"

# P2-0.1 official RUN → this RUN: subtrees whose difference a P2-0.2 decision
# causes, `(file, path prefix, decision, why)`; written from the development
# RUN before the official RUN (X5 record: development RUN
# 20261019T191656Z-7d4aa8b, X5.md; the subtrees of P2-0.1's rows, decisions
# remapped; narrowest subtree or leaf, no wildcard)
EXCLUDED: list[tuple[str, tuple, str, str]] = [
    (
        "conformance_p1d1.json",
        ("fixtures_manifest_hash",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    ("conformance_p1d1.json", ("passed",), "K-P2-02.3", "spec 2.0.2 and conformance 39 (C2-14)"),
    (
        "conformance_p1d1.json",
        ("spec_version",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    ("conformance_p1d1.json", ("tests",), "K-P2-02.3", "spec 2.0.2 and conformance 39 (C2-14)"),
    ("conformance_p1d1.json", ("total",), "K-P2-02.3", "spec 2.0.2 and conformance 39 (C2-14)"),
    (
        "p1b1_checks.json",
        ("criteria", "B1-3", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    ("p1b_checks.json", ("fixtures_manifest_hash",), "K-P2-02.3", "fixture manifest: C2-14"),
    (
        "p1c1_checks.json",
        ("C1-7", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1c_checks.json",
        ("C-1", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1d/scene/scene.json",
        ("steps", "0", "shows", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    ("p1d1_checks.json", ("D1-10", "passed"), "K-P2-02.3", "D1-10 expects 34 tests and spec 1.0.3"),
    (
        "p1d1_checks.json",
        ("D1-10", "spec_version"),
        "K-P2-02.3",
        "D1-10 expects 34 tests and spec 1.0.3",
    ),
    (
        "p1d1_checks.json",
        ("D1-12", "regress_vs_p1d", "conformance", "spec_version", "1"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1d_checks.json",
        ("D-9", "conformance", "spec_version", "1"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("fields_declared",),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("fields_in_code",),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("file_constants",),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("modules", "#len"),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("modules", "11"),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e/check_compare_fields.json",
        ("modules", "12"),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e/client.json",
        ("bundle_bytes",),
        "K-P2-02.2",
        "the plugin attests every rename (K-P2-02.2): bundle and tests",
    ),
    (
        "p1e/plugin_e2e/plugin_e2e.json",
        ("steps", "0", "shows", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/conformance_equiv.json",
        ("passed",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/conformance_equiv.json",
        ("spec_version",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/conformance_equiv.json",
        ("total",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("fixtures_manifest_hash",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("passed",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("spec_version",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("tests",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/conformance_rootmark.json",
        ("total",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/release_local.json",
        ("conformance", "passed"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/release_local.json",
        ("conformance", "total"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/release/release_local.json",
        ("spec_version",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/scene/scene.json",
        ("steps", "0", "shows", "mcp_spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e/scene/scene.json",
        ("steps", "0", "shows", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("fields_declared",),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("fields_in_code",),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("file_constants",),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("modules", "#len"),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("modules", "11"),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e1/check_compare_fields.json",
        ("modules", "12"),
        "P2-0.2/F6",
        "the field audit sees the new comparator compare_p2_0_2 (its module, rows and file constants)",
    ),
    (
        "p1e1/historical.json",
        ("fails",),
        "K-P2-02.3",
        "eval.historical (reference P1-E) sees the FAILs P2-0, P2-0.1 and P2-0.2 decisions cause; eval.historical_p2_0_2 classifies them",
    ),
    (
        "p1e1/release/release.json",
        ("commit",),
        "K-P2-02.1",
        "the release package of this round's commit (the reference is P2-0.1's)",
    ),
    (
        "p1e1/release/release.json",
        ("spec_version",),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1/release/release.json",
        ("targets", "0", "conformance", "passed"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1/release/release.json",
        ("targets", "0", "conformance", "total"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1/release/release.json",
        ("targets", "0", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "1", "item"),
        "K-P2-02.3",
        "the checklist item names 39/39 (P2-0.2/F4)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "14", "detail"),
        "K-P2-02.1",
        "R14's detail names the newest ROUND_START (this round's)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "3", "item"),
        "K-P2-02.1",
        "R4a's item names the commit of the package (the reference is P2-0.1's)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "4", "item"),
        "K-P2-02.3",
        "the checklist item names 39/39 (P2-0.2/F4)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "8", "detail"),
        "K-P2-02.2",
        "R8 records the plugin bundle size (every rename attested, K-P2-02.2)",
    ),
    (
        "p1e1/release/release_check.json",
        ("items", "9", "item"),
        "K-P2-02.3",
        "the checklist item names 39/39 (P2-0.2/F4)",
    ),
    (
        "p1e1_checks.json",
        ("E1-5", "required", "conformance", "passed"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1_checks.json",
        ("E1-5", "required", "conformance", "total"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1_checks.json",
        ("E1-5", "required", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1_checks.json",
        ("E1-6", "conformance", "passed"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1_checks.json",
        ("E1-6", "conformance", "total"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e1_checks.json",
        ("E1-6", "regress_vs_p1e"),
        "K-P2-02.3",
        "E1-6's regress against P1-E sees every P2-0, P2-0.1 and P2-0.2 difference",
    ),
    (
        "p1e_checks.json",
        ("E-10", "bundle_bytes"),
        "K-P2-02.2",
        "the plugin attests every rename (K-P2-02.2): bundle and tests",
    ),
    (
        "p1e_checks.json",
        ("E-7", "chain_local_target", "conformance", "passed"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e_checks.json",
        ("E-7", "chain_local_target", "conformance", "total"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e_checks.json",
        ("E-7", "chain_local_target", "spec_version"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e_checks.json",
        ("E-9", "conformance", "passed"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e_checks.json",
        ("E-9", "conformance", "total"),
        "K-P2-02.3",
        "spec 2.0.2 and conformance 39 (C2-14)",
    ),
    (
        "p1e_checks.json",
        ("E-9", "regress_vs_p1d1"),
        "K-P2-02.3",
        "E-9's regress against P1-D.1 sees every P2-0, P2-0.1 and P2-0.2 difference",
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
        or _match(P2_0_2_TIMING.get(name, []), path)
        or _match(P2_0_2_INFO.get(name, []), path)
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
    """`compare_p2_0_1.compare_files` with the P2-0.2 skips."""
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
    q = compare_files(a, b, P2_0_2_FILES, lambda _n, _p: None)
    res["files"] = [*res["files"], *P2_0_2_FILES]
    res["n_fields"] += q["n_fields"]
    res["differing_fields"].update(q["differing_fields"])
    pb = _bytes(a, b, P2_0_2_BYTES)
    res["bytes"].update(pb)
    res["h_p2_0_2i"] = {"files": pb, "equal": all(v["equal"] for v in pb.values())}
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def _json(p: Path) -> dict:
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def _lines(p: Path) -> list[dict]:
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return [json.loads(x) for x in p.read_text(encoding="utf-8").splitlines() if x]


def _set_equal(ref: Path, run: Path, name: str) -> dict:
    """A P2-0.1 set P2-0.2 runs again unchanged: every field and its
    `events.jsonl` byte for byte."""
    a, b = _json(ref / "p2_0_1" / f"{name}.json"), _json(run / "p2_0_2" / f"{name}.json")
    la, lb = C.leaves(a), C.leaves(b)
    diff = sorted(C._key(p) for p in set(la) | set(lb) if la.get(p) != lb.get(p))
    ea = (ref / "p2_0_1" / name / "events.jsonl").read_bytes()
    eb = (run / "p2_0_2" / name / "events.jsonl").read_bytes()
    return {"differing": diff[:50], "events_equal": ea == eb, "equal": not diff and ea == eb}


def _d_c(ref: Path, run: Path) -> dict:
    """D-C under T3 and X2-1 (X2): the same events as the reference but for
    their `seq` order; the same attestation lines but `matched: false` on
    the collide lines, which T3 leaves alone."""
    ea = sorted(
        json.dumps({k: v for k, v in e.items() if k != "seq"}, sort_keys=True)
        for e in _lines(ref / "p2_0_1" / "d_c" / "events.jsonl")
    )
    eb = sorted(
        json.dumps({k: v for k, v in e.items() if k != "seq"}, sort_keys=True)
        for e in _lines(run / "p2_0_2" / "d_c" / "events.jsonl")
    )
    aa = _lines(ref / "p2_0_1" / "d_c" / "attestations.jsonl")
    ab = _lines(run / "p2_0_2" / "d_c" / "attestations.jsonl")
    stripped = [{k: v for k, v in x.items() if k != "matched"} for x in ab]
    matched = sum(1 for x in ab if x.get("matched") is False)
    out = {
        "events_same_multiset": ea == eb,
        "attestations_equal_but_matched": stripped == aa,
        "matched_false_lines": matched,
        "collide_ticks": _json(run / "p2_0_2" / "d_c.json")["modes"]["collide"],
    }
    out["equal"] = (
        out["events_same_multiset"]
        and out["attestations_equal_but_matched"]
        and matched == out["collide_ticks"]
    )
    return out


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
        and cb["spec_version"] == SPEC
        and cb["passed"] == ca["passed"] + len(NEW_TESTS)
    )
    de_ref = _json(ref / "p2_0_1" / "d_e.json")["vaults"]
    de_run = _json(run / "p2_0_2" / "d_e.json")["vaults"][: len(de_ref)]
    sets = {
        "d_a": _set_equal(ref, run, "d_a"),
        "d_a2": _set_equal(ref, run, "d_a2"),
        "d_e_p2_0_1": {"equal": de_ref == de_run},
        "d_c": _d_c(ref, run),
    }
    sets["equal"] = all(v["equal"] for v in sets.values())
    res["conformance"] = conf
    res["p2_0_1_sets"] = sets
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
