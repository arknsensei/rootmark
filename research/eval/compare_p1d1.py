"""P1-D.1 run comparisons (GOAL D1-12), with the frozen leaf extraction and
field comparison (`compare_runs.leaves`, `metrics.max_relative_difference`)
and the frozen P1-D comparator (`compare_p1d`).

`pair <run_A> <run_B>` (D1-12, `scripts/verify.sh pair`): the P1-D pair set
(`compare_p1d.pair`: the P1-C.1 set, the P1-D files, the P1-D artefacts
byte for byte) and the P1-D.1 result files (`P1D1_FILES`), every field
except the P1-D.1 wall-clock fields (`P1D1_TIMING`), and the P1-D.1
artefacts byte for byte (`P1D1_BYTES`). No other exclusion: two chains of
one commit must agree everywhere.

`regress <p1d_run> <run>` (D1-12, "P1-D kalite alanları"): the P1-D pair
set of files (`compare_p1d`: P1-C.1 set + P1-D files) of the P1-D official
RUN against this RUN, every field, **except** the subtrees of
`EXCLUDED` — each one a difference that a P1-D.1 decision causes, written
before the official RUN (TASKS D1.7; precedent K-C1.6) — and the
conformance report's P1-C.1 view (`conformance_p1c1.json`: C1, C2-01..09,
C3-01..05 by id; spec version and manifest hash excluded by K-D1.2/K-D1.6/
K-D1.8 and the added fixtures). A difference outside the exclusions fails.
Every excluded leaf that does differ is listed, and so is every exclusion
that matched no difference (so the list cannot hide more than it names).

    python -m eval.compare_p1d1 pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p1d1 regress <p1d_run> <run> [--out path.json]

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
from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1c1 import PAIR_FILES as P1C1_FILES
from eval.datasets.common import dump_json, file_blake3

P1D1 = Path("p1d1")
P1D1_FILES = [
    "p1d1_checks.json",
    "conformance_p1d1.json",
    "p1d1/p1d1.json",
    *[f"p1d1/d1_{n}.json" for n in (1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 13)],
    "p1d1/v3_compat.json",
    "p1d1/v3_compat_mixed.json",
    "p1d1/tamper_p1d1.json",
]
# wall-clock fields (file, path pattern; "*" = any key/index)
P1D1_TIMING = {
    "p1d1/d1_5.json": [("elapsed_ms_pipelined",), ("elapsed_ms_lockstep",)],
    "p1d1/d1_6.json": [
        ("elapsed_ms_serial",),
        ("elapsed_ms_pipelined",),
        ("ratio",),
        ("ratio_below_0_6",),
    ],
    "p1d1_checks.json": [("D1-6", "measured")],
    # absolute fixture root of the chain's clone
    "conformance_p1d1.json": [("fixtures_root",)],
}
P1D1_BYTES = [
    P1D1 / "d1_3.json",
    # d1_5.json holds wall-clock fields: compared field by field only (F16)
    P1D1 / "d1_8.json",
    P1D1 / "d1_13.json",
]

# P1-D official RUN → this RUN: subtrees whose difference a P1-D.1 decision
# causes, `(file, path prefix, decision, why)`: written from the development
# RUN 20261016T105235Z-7a4718b (52 differing leaves, every one listed by a
# prefix here) before the official RUN (D1.7 record; precedent K-C1.6)
EXCLUDED: list[tuple[str, tuple, str, str]] = [
    (
        "p1b1_checks.json",
        ("criteria", "B1-3", "spec_version"),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "the frozen B1-3 record names the spec version",
    ),
    (
        "p1b_checks.json",
        ("fixtures_manifest_hash",),
        "F2, F6 (C2-10, C3-06, C3-07)",
        "MANIFEST.json gains the new fixtures' rows",
    ),
    (
        "p1c_checks.json",
        ("C-1", "spec_version"),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "the frozen C-1 record names the spec version",
    ),
    (
        "p1c_index.json",
        ("index_files_build", "manifest.json"),
        "K-D1.7",
        "manifest lexical.dl, format mdkernel-index/3",
    ),
    (
        "p1c_index.json",
        ("index_files_compact", "manifest.json"),
        "K-D1.7",
        "manifest lexical.dl, format mdkernel-index/3",
    ),
    (
        "p1c_index.json",
        ("index_files_final", "manifest.json"),
        "K-D1.7",
        "manifest lexical.dl, format mdkernel-index/3",
    ),
    ("p1c1_checks.json", ("C1-6", "lexical", "dl"), "K-D1.7", "the lexical record gains dl"),
    ("p1c1_checks.json", ("C1-6", "rejected_cases"), "K-D1.7 (F1)", "two rejection cases for dl"),
    (
        "p1c1_checks.json",
        ("C1-7",),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "C1-7 checks the conformance report's spec version (1.0.2)",
    ),
    (
        "p1d/http/http_e2e.json",
        ("cases", "5"),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "D7-3 checks status spec_version 1.0.2",
    ),
    (
        "p1d/http/http_e2e.json",
        ("cases", "8"),
        "K-D1.5",
        "H-1 expects every tool error as 422; codes now set the status",
    ),
    (
        "p1d/http/http_e2e.json",
        ("cases", "11"),
        "K-D1.5",
        "W-3 expects blame of an unknown document as 422; now 404 not_found",
    ),
    (
        "p1d/http/http_e2e.json",
        ("d7_passed",),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "count of D7-3 above",
    ),
    ("p1d/http/http_e2e.json", ("passed",), "K-D1.5, spec 1.0.3", "count of the cases above"),
    ("p1d/http/http_e2e.json", ("pass",), "K-D1.5, spec 1.0.3", "conjunction of the cases above"),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("checks", "3"),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "status: versions (spec 1.0.2)",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("checks", "41"),
        "K-D1.1",
        "heading-less append to the human espresso.md is denied (append_only)",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("checks", "49"),
        "K-D1.1",
        "replace_section of the human Recipe section is denied (append_only)",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("checks", "50"),
        "K-D1.1",
        "follows from the denied replace_section",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("checks", "52"),
        "K-D1.1",
        "follows from the denied append and replace_section",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("checks", "53"),
        "K-D1.6",
        "checkpoint {name} after a search names the state: one CheckpointNamed event",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("per_tool", "propose"),
        "K-D1.1",
        "count of the propose checks above",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("per_tool", "status"),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "count of the status check above",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("tools_passed",),
        "K-D1.1, spec 1.0.3",
        "count of the tools above",
    ),
    (
        "p1d/mcp_e2e/mcp_e2e.json",
        ("pass",),
        "K-D1.1, K-D1.6, spec 1.0.3",
        "conjunction of the checks above",
    ),
    (
        "p1d/scene/scene.json",
        ("steps", "0", "shows", "spec_version"),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "the scene shows status spec_version",
    ),
    (
        "p1d_checks.json",
        ("D-2",),
        "K-D1.1, spec 1.0.3",
        "P1-D's D-2 record of the e2e counts above",
    ),
    (
        "p1d_checks.json",
        ("D-7",),
        "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3 by CRITERION)",
        "P1-D's D-7 record of D7-3 and the HTTP counts above",
    ),
    (
        "p1d_checks.json",
        ("D-9",),
        "K-D1.7, spec 1.0.3, F2/F6",
        "P1-D's D-9 (against P1-C.1) records the spec version, manifest and lexical.dl differences above",
    ),
]


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


def _timing(name: str, path: tuple) -> bool:
    for pat in P1D1_TIMING.get(name, []):
        if len(path) >= len(pat) and all(
            p == "*" or p == str(x) for p, x in zip(pat, path, strict=False)
        ):
            return True
    return False


def compare_files(a: Path, b: Path, files: list[str], exclude) -> dict:
    """Every leaf of every file except `exclude(name, path)`; a field set that
    differs outside the exclusions is a difference of a field named
    `<file>:<path>` with value "absent"."""
    n = 0
    differing: dict = {}
    excluded: dict = {}
    for name in files:

        def keep(p: tuple, name: str = name) -> bool:
            return (
                not C._excluded(name, p) and not P._p1d_excluded(name, p) and not _timing(name, p)
            )

        la = {p: v for p, v in C.leaves(P._load(a, name)).items() if keep(p)}
        lb = {p: v for p, v in C.leaves(P._load(b, name)).items() if keep(p)}
        for p in sorted(set(la) | set(lb), key=C._key):
            e = exclude(name, p)
            va, vb = la.get(p, "absent"), lb.get(p, "absent")
            if e is not None:
                if va != vb:
                    excluded.setdefault(
                        f"{e[0]}:{'/'.join(map(str, e[1]))}", {"decision": e[2], "leaves": 0}
                    )
                    excluded[f"{e[0]}:{'/'.join(map(str, e[1]))}"]["leaves"] += 1
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


def p1d1_bytes(a: Path, b: Path) -> dict:
    out = {}
    for f in P1D1_BYTES:
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
    res = P.pair(a, b)
    q = compare_files(a, b, P1D1_FILES, lambda _n, _p: None)
    res["files"] = [*res["files"], *P1D1_FILES]
    res["n_fields"] += q["n_fields"]
    res["differing_fields"].update(q["differing_fields"])
    res["bytes"].update(p1d1_bytes(a, b))
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def regress(ref: Path, run: Path) -> dict:
    files = [*P1C1_FILES, *P.P1D_FILES]
    res = compare_files(ref, run, files, _excl_match)
    ca, cb = P._load(ref, P.CONFORMANCE), P._load(run, P.CONFORMANCE)
    by_a = {t["id"]: t for t in P._tests(ca)}
    by_b = {t["id"]: t for t in P._tests(cb)}
    conf = {
        "tests": len(by_b),
        "differing_tests": sorted(i for i in set(by_a) | set(by_b) if by_a.get(i) != by_b.get(i)),
        "spec_version": [ca["spec_version"], cb["spec_version"]],
        "level_claimed": [ca["level_claimed"], cb["level_claimed"]],
        "excluded": {
            "spec_version": "K-D1.2, K-D1.6, K-D1.8 (spec 1.0.3, CRITERION yama kuralı)",
            "fixtures_manifest_hash": "C2-10, C3-06, C3-07 fixtures added (F2, F6)",
        },
    }
    conf["equal"] = not conf["differing_tests"] and ca["level_claimed"] == cb["level_claimed"]
    res["conformance"] = conf
    res["reference_run"] = ref.name
    res["files"] = files
    unused = [
        f"{e[0]}:{'/'.join(map(str, e[1]))}"
        for e in EXCLUDED
        if f"{e[0]}:{'/'.join(map(str, e[1]))}" not in res["excluded_differences"]
    ]
    res["exclusions"] = [
        {"file": e[0], "path": "/".join(map(str, e[1])), "decision": e[2], "why": e[3]}
        for e in EXCLUDED
    ]
    res["exclusions_without_difference"] = unused
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
