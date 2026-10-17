"""P1-D run comparisons (GOAL D-8, D-9), with the frozen leaf extraction
and field comparison (`compare_runs.leaves`, `metrics.max_relative_difference`).

`pair <run_A> <run_B>` (D-8, `scripts/verify.sh pair`): the P1-C.1 pair set
(`compare_p1c1.PAIR_FILES`, its latency exclusions) and the P1-D result
files, every field except the P1-D wall-clock fields (`P1D_LATENCY`), the
governance vault's bytes (`compare_p1b.bytes_equal`) and the P1-D artefacts
byte for byte (`P1D_BYTES`: the MCP e2e transcripts of both sessions, the
CLI promotions, the e2e vault's events.jsonl, traces.jsonl,
proposals.jsonl and proposed Markdown, the scene, the Inspector's tool
list). The P1-D files carry no time stamp and no token; a run directory's
absolute path appears in none of them.

`regress <p1c1_run> <run>` (D-9): the P1-C.1 pair set of the P1-C.1
official RUN against this RUN, every field (no exclusion besides the frozen
latency paths), and the conformance report's tests (C1, C2, C3: id, level,
result, observations) and spec version equal.

    python -m eval.compare_p1d pair <run_A> <run_B> [--out path.json]
    python -m eval.compare_p1d regress <p1c1_run> <run> [--out path.json]

Exit code: 0 when nothing differs, 3 when something does (the comparison is
still written), 1 on a missing file or field.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from eval import compare_runs as C
from eval import metrics as M
from eval.compare_p1b import bytes_equal
from eval.compare_p1c1 import PAIR_FILES as P1C1_FILES
from eval.datasets.common import dump_json, file_blake3

P1D = Path("p1d")
P1D_FILES = [
    "p1d_checks.json",
    "p1d/inspector/inspector.json",
    "p1d/mcp_e2e/mcp_e2e.json",
    "p1d/http/http_e2e.json",
    "p1d/scene/scene.json",
    "p1d/tamper_p1d.json",
    "p1d/d10.json",
    "p1d/sigterm.json",
]
# wall-clock fields of the P1-D files (file, path pattern; "*" = any key/index)
P1D_LATENCY = {
    "p1d/d10.json": [("latencies_ms",), ("p95_ms",), ("max_ms",)],
    "p1d/sigterm.json": [("trials", "*", "timer_compactions"), ("timer_compactions",)],
    "p1d_checks.json": [("D-10", "measured")],
}
E2E = P1D / "mcp_e2e" / "run1"
P1D_BYTES = [
    E2E / "transcript.jsonl",
    E2E / "transcript_b.jsonl",
    E2E / "cli_promote.jsonl",
    E2E / "vault" / ".rootmark" / "events.jsonl",
    E2E / "vault" / ".rootmark" / "traces.jsonl",
    E2E / "vault" / ".rootmark" / "proposals.jsonl",
    E2E / "vault" / "espresso.md",
    E2E / "vault" / "agent" / "plan.md",
    P1D / "scene" / "scene.json",
    P1D / "inspector" / "tools_list.json",
]
CONFORMANCE = "conformance_p1c1.json"


def _p1d_excluded(name: str, path: tuple) -> bool:
    for pat in P1D_LATENCY.get(name, []):
        if len(path) >= len(pat) and all(
            p == "*" or p == str(x) for p, x in zip(pat, path, strict=False)
        ):
            return True
    return False


def _load(d: Path, name: str):
    p = C.resolve(d, name)  # a pre-rename RUN's names (K-GH.1, GH-0/F3)
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing")
    return json.loads(p.read_text(encoding="utf-8"))


def compare(a: Path, b: Path, files: list[str]) -> dict:
    """Every leaf of every file (latency paths aside); a field one side lacks
    is a differing field `{"a", "b"}` with "absent" (P1-D.1/F15)."""
    n = 0
    differing: dict = {}
    for name in files:

        def keep(p: tuple, name: str = name) -> bool:
            return not C._excluded(name, p) and not _p1d_excluded(name, p)

        la = {p: v for p, v in C.leaves(_load(a, name)).items() if keep(p)}
        lb = {p: v for p, v in C.leaves(_load(b, name)).items() if keep(p)}
        # P1-D.1/F15: a field only one side has is a difference (recorded,
        # exit 3), not an error; equal field sets compare as before
        for p in sorted(set(la) ^ set(lb), key=C._key):
            differing[f"{name}:{C._key(p)}"] = {"a": la.pop(p, "absent"), "b": lb.pop(p, "absent")}
        keys = sorted(la, key=C._key)
        fa = {C._key(p): la[p] for p in keys}
        fb = {C._key(p): lb[p] for p in keys}
        r = M.max_relative_difference(fa, fb, sorted(fa))
        n += r["n_fields"]
        for k, v in r["differing_fields"].items():
            differing[f"{name}:{k}"] = v
    return {
        "run_a": str(a),
        "run_b": str(b),
        "files": files,
        "n_fields": n,
        "differing_fields": differing,
    }


def p1d_bytes(a: Path, b: Path) -> dict:
    out = {}
    for f in P1D_BYTES:
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
    res = compare(a, b, [*P1C1_FILES, *P1D_FILES])
    res["bytes"] = {**bytes_equal(a, b), **p1d_bytes(a, b)}
    res["bytes_equal"] = all(v["equal"] for v in res["bytes"].values())
    res["equal"] = not res["differing_fields"] and res["bytes_equal"]
    return res


def _tests(conf: dict) -> list:
    return [
        {k: t[k] for k in sorted(t) if k not in ("duration_ms", "elapsed_ms")}
        for t in sorted(conf["tests"], key=lambda t: t["id"])
    ]


def regress(ref: Path, run: Path) -> dict:
    res = compare(ref, run, list(P1C1_FILES))
    ca, cb = _load(ref, CONFORMANCE), _load(run, CONFORMANCE)
    ta, tb = _tests(ca), _tests(cb)
    by_a = {t["id"]: t for t in ta}
    by_b = {t["id"]: t for t in tb}
    conf = {
        "tests": len(tb),
        "by_level": {
            str(lvl): f"{sum(t['result'] == 'pass' for t in tb if t['level'] == lvl)}/"
            f"{sum(1 for t in tb if t['level'] == lvl)}"
            for lvl in (1, 2, 3)
        },
        "differing_tests": sorted(i for i in set(by_a) | set(by_b) if by_a.get(i) != by_b.get(i)),
        "spec_version": [ca["spec_version"], cb["spec_version"]],
        "level_claimed": [ca["level_claimed"], cb["level_claimed"]],
        "fixtures_manifest_hash_equal": ca["fixtures_manifest_hash"]
        == cb["fixtures_manifest_hash"],
    }
    conf["equal"] = (
        not conf["differing_tests"]
        and ca["spec_version"] == cb["spec_version"]
        and ca["level_claimed"] == cb["level_claimed"]
        and conf["fixtures_manifest_hash_equal"]
    )
    res["conformance"] = conf
    res["reference_run"] = ref.name
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
    except (FileNotFoundError, M.MetricError) as e:
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
