"""P1-C criteria recorded in the RUN (GOAL C-1..C-13; TASKS C9), so that the
round report is built from one RUN directory only: `p1c_checks.json` and
`p1c_index.json`.

Computed here, through the binding, on a copy of D3 (`make d3`, verified
against `MANIFEST.d3.json`) in `--work` (outside the RUN: the vault is
~70 MB): one checkpoint, `index build`, the first 200 D4 test questions (id
order) in the modes flat, lexical and hybrid (k 10):

- C-4: 10 of them × 3 modes × 100 repetitions → identical rankings
- C-5: the index deleted and rebuilt from the same vault → identical
  rankings and byte-identical index files
- C-6 (input of the pair comparison): the rankings (with scores) and the
  blake3 of every index file, after the build and at the end
- C-9: 20 seeded `index watch` edits (hot tier), a checkpoint, the 200
  rankings at both checkpoints before and after `compact()` → identical;
  then 3 more watches so that the final directory holds a hot tier

Read from files `make p1c-checks` wrote into the RUN before: `conformance_full.json`
(C-1, C-2, C-12), `c13_tests.log` (C-13), `c3_quality.json` (C-3),
`c7_property.json` (C-7), `c10_embedding.json` (C-10), `models_verify.json`,
`regress_vs_p1b1.json` (C-11, `eval.compare_p1c regress`). C-8 is
machine-dependent: `c8_latency.json` is read by the report only and is not
part of the pair comparison. C-6 itself is decided by `eval.compare_p1c pair`.

    python -m eval.run_p1c_checks --run <RUN> --work <dir>
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
from pathlib import Path

import rootmark_core

from eval import runinfo
from eval.datasets.common import die, file_blake3
from eval.run_h2 import D3, load_split, verify_d3

MODES = ["flat", "lexical", "hybrid"]
K = 10
N_QUERIES = 200
C4_QUERIES = 10
C4_REPEATS = 100
HOT_EDITS = 20
FINAL_EDITS = 3
CONSUMER = "p1c-checks"
C13_TESTS = [
    "kc01_first_of_two_drops_in_one_snapshot_removed_is_an_error_human",
    "kc01_first_of_two_drops_in_one_snapshot_removed_is_an_error_synthetic",
    "kc01_single_drop_removed_opens_as_derived_prefix",
]


def _load(d: Path, name: str):
    p = d / name
    if not p.is_file():
        raise FileNotFoundError(f"{p} missing (make p1c-checks)")
    return json.loads(p.read_text(encoding="utf-8"))


def index_files(vault: Path) -> dict[str, str]:
    """blake3 of every index file; the reader's empty lock file is not data."""
    d = vault / ".rootmark" / "index"
    out = {
        p.relative_to(d).as_posix(): file_blake3(p)
        for p in sorted(d.rglob("*"))
        if p.is_file() and p.name != ".tantivy-meta.lock"
    }
    if not out:
        raise RuntimeError(f"{d} holds no index")
    return out


def rankings(v, queries: list[str], at: str) -> dict:
    out = {}
    for mode in MODES:
        res = v.search_many(queries, at, k=K, mode=mode, consumer=CONSUMER)
        out[mode] = [
            [[h["doc_path"], h["chunk_id"], h["layer"], h["score"]] for h in r["hits"]] for r in res
        ]
    return out


def edit(text: str, i: int) -> str:
    return (
        text.rstrip("\n")
        + f"\n\nHot tier note {i}: the espresso grind, water temperature and roast development were reviewed.\n"
    )


def determinism(work: Path) -> tuple[dict, dict]:
    d3 = rootmark_core.Vault(str(D3))
    verify_d3(d3)
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    shutil.copytree(D3 / ".rootmark", work / ".rootmark", ignore=shutil.ignore_patterns("index*"))
    v = rootmark_core.Vault(str(work), matcher=d3.matcher_lock())
    if v.events_hash() != d3.events_hash():
        raise RuntimeError("copied vault differs from data/d3")
    qs = sorted(load_split("test"), key=lambda q: q["id"])[:N_QUERIES]
    if len(qs) != N_QUERIES:
        raise RuntimeError(f"{len(qs)} test questions < {N_QUERIES}")
    queries = [q["question"] for q in qs]
    cp = v.checkpoint("p1c")["checkpoint_id"]
    build = v.index_build()
    r1 = rankings(v, queries, "p1c")
    files_build = index_files(work)
    # C-4: identical over 100 repetitions
    sub = queries[:C4_QUERIES]
    first = rankings(v, sub, "p1c")
    c4 = all(rankings(v, sub, "p1c") == first for _ in range(C4_REPEATS - 1))
    c4 = c4 and all(first[m] == r1[m][:C4_QUERIES] for m in MODES)
    # C-5: delete and rebuild from the same checkpoint
    shutil.rmtree(work / ".rootmark" / "index")
    rebuild = v.index_build()
    r2 = rankings(v, queries, "p1c")
    files_rebuild = index_files(work)
    c5 = r2 == r1 and files_rebuild == files_build and rebuild["embedded"] == rebuild["records"]
    # C-9: hot tier, compact
    docs = v.doc_paths()
    picks = [docs[i * len(docs) // HOT_EDITS] for i in range(HOT_EDITS)]
    watched = []
    for i, doc in enumerate(picks):
        text = edit((D3 / "docs" / doc).read_text(encoding="utf-8"), i)
        watched.append(v.index_watch(doc, text, "Human", "p1c-checks"))
    cp_hot = v.checkpoint("p1c-hot")["checkpoint_id"]
    status_hot = v.index_status()
    before = {"p1c": rankings(v, queries, "p1c"), "p1c-hot": rankings(v, queries, "p1c-hot")}
    compacted = v.index_compact()
    after = {"p1c": rankings(v, queries, "p1c"), "p1c-hot": rankings(v, queries, "p1c-hot")}
    files_compact = index_files(work)
    c9 = before == after and before["p1c"] == r1 and compacted["moved_from_hot"] > 0
    for i, doc in enumerate(picks[:FINAL_EDITS]):
        text = edit(edit((D3 / "docs" / doc).read_text(encoding="utf-8"), i), HOT_EDITS + i)
        v.index_watch(doc, text, "Human", "p1c-checks")
    files_final = index_files(work)
    checks = {
        "queries": N_QUERIES,
        "modes": MODES,
        "k": K,
        "checkpoint_id": cp,
        "checkpoint_hot_id": cp_hot,
        "build": build,
        "C-4": {
            "queries": C4_QUERIES,
            "repeats": C4_REPEATS,
            "modes": MODES,
            "pass": c4,
        },
        "C-5": {
            "rankings_equal": r2 == r1,
            "index_files_equal": files_rebuild == files_build,
            "rebuild": rebuild,
            "pass": c5,
        },
        "C-9": {
            "edits": HOT_EDITS,
            "embedded": sum(w["embedded"] for w in watched),
            "hot_status": status_hot,
            "compact": compacted,
            "rankings_equal": before == after,
            "pass": c9,
        },
        "events_hash": v.events_hash(),
        "traces": None,
    }
    traces = (work / ".rootmark" / "traces.jsonl").read_bytes()
    checks["traces"] = {
        "lines": traces.count(b"\n"),
        "blake3": file_blake3(work / ".rootmark" / "traces.jsonl"),
    }
    index = {
        "rankings_200": r1,
        "rankings_after_compact": after,
        "index_files_build": files_build,
        "index_files_compact": files_compact,
        "index_files_final": files_final,
    }
    return checks, index


def c13(d: Path) -> dict:
    log = (d / "c13_tests.log").read_text(encoding="utf-8")
    ok = {
        t: re.search(rf"^test {re.escape(t)} \.\.\. ok$", log, re.MULTILINE) is not None
        for t in C13_TESTS
    }
    return {"tests": ok, "pass": all(ok.values())}


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--run", required=True)
    ap.add_argument("--work", required=True, type=Path)
    a = ap.parse_args(argv)
    d = runinfo.run_dir(a.run, create=False)
    conf = _load(d, "conformance_full.json")
    by = {lvl: [t for t in conf["tests"] if t["level"] == lvl] for lvl in (1, 2, 3)}
    passed = {lvl: sum(t["result"] == "pass" for t in by[lvl]) for lvl in by}
    c209 = [t for t in by[2] if t["id"] == "C2-09"]
    q = _load(d, "c3_quality.json")
    c7 = _load(d, "c7_property.json")
    c10 = _load(d, "c10_embedding.json")
    models = _load(d, "models_verify.json")
    reg = _load(d, "regress_vs_p1b1.json")
    checks, index = determinism(a.work.resolve())
    out = {
        "C-1": {
            "C3": f"{passed[3]}/{len(by[3])}",
            "level_claimed": conf["level_claimed"],
            "spec_version": conf["spec_version"],
            "fixtures_manifest": conf["fixtures_manifest"],
            "pass": conf["level_claimed"] == 3
            and passed[3] == len(by[3]) == 4
            and conf["fixtures_manifest"] == "ok",
        },
        "C-2": {
            "C1": f"{passed[1]}/{len(by[1])}",
            "C2": f"{passed[2]}/{len(by[2])}",
            "pass": passed[1] == len(by[1]) == 17 and passed[2] == len(by[2]) == 9,
        },
        "C-3": {
            "single": q["C-3"]["single"],
            "multi": q["C-3"]["multi"],
            "threshold": q["threshold_abs_delta_ndcg@10"],
            "pass": q["pass"],
        },
        "C-4": checks["C-4"],
        "C-5": checks["C-5"],
        "C-6": {"decided_by": "eval.compare_p1c pair (pair_compare.json)"},
        "C-7": c7,
        "C-8": {"decided_by": "report from c8_latency.json (machine-dependent)"},
        "C-9": checks["C-9"],
        "C-10": {
            "n": c10["n"],
            "min_cosine": c10["min_cosine"],
            "threshold": c10["threshold"],
            "pass": c10["pass"] and c10["n"] == 50,
        },
        "C-11": {
            "reference_run": reg["reference_run"],
            "differing_outside_exclusions": reg["differing_outside_exclusions"],
            "pass": reg["C-11_equal"],
        },
        "C-12": {
            "C2-09": c209[0]["result"] if c209 else None,
            "pass": len(c209) == 1 and c209[0]["result"] == "pass" and conf["level_claimed"] >= 2,
        },
        "C-13": c13(d),
        "models": {"embedder_id": models["embedder_id"], "ok": models["ok"]},
        "determinism_vault": {
            k: checks[k]
            for k in ("checkpoint_id", "checkpoint_hot_id", "build", "events_hash", "traces")
        },
    }
    runinfo.write_json(a.run, "p1c_checks.json", out)
    runinfo.write_json(a.run, "p1c_index.json", index)
    runinfo.record_phase(
        a.run,
        "p1c_checks",
        {"queries": N_QUERIES, "modes": MODES, "k": K, "hot_edits": HOT_EDITS},
        ["p1c_checks.json", "p1c_index.json"],
    )
    print(
        "; ".join(
            f"{c} {'pass' if out[c]['pass'] else 'FAIL'}"
            for c in out
            if isinstance(out[c], dict) and "pass" in out[c]
        )
    )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
