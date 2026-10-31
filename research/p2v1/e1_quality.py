"""P2-V.1/E1 (GOAL P2-V.1 Kapsam 2, V1-3): the quality criteria with the
fixed-order embedder, and every ranking change against the ONNX Runtime
reference.

- C-3 (`c3_quality.json`), C-10 (`c10_embedding.json`), C1-1
  (`c1_variant.json`) of a RUN made by the frozen recorders (`make
  p1c-checks`, `make p1c1-checks` up to their regress steps), C1-9 by the
  frozen comparison (`run_p1c1_checks.quality_vs_ref` against
  `data/quality_ref_rust.json`, 618 values);
- the 200 rankings of C-4/C-5/C-6 (`run_p1c_checks.rankings`, the first 200
  D4 test questions, three modes, k 10, on a copy of D3 at one checkpoint)
  computed here through the binding, against `rankings_200` of the P1-C
  official RUN (`p1c_index.json`, ONNX Runtime on the release Mac): every
  query whose `(doc_path, chunk_id, layer)` sequence differs is listed by
  mode; scores are compared separately (largest absolute difference);
- `c3_quality.json` leaf by leaf against the P1-C official RUN's;
- P2-L's D-TR and English reference (`make p2l-quality` into the same RUN:
  `p2l/d_tr.json`, `p2l/english.json`; L-2..L-6 with both models) leaf by
  leaf against the P2-L official RUN's (gözden geçirici koşulu 1).

    python -m p2v1.e1_quality --run <RUN> --work <scratch dir> --out quality.json

(run from `research/` with `PYTHONPATH=.`). Exit 0 always when the record is
written; the verdict is in the record (V1-3 is read in FINAL).
"""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

import rootmark_core

from eval.datasets.common import ROOT
from eval.run_h2 import D3, load_split, verify_d3
from eval.run_p1c1_checks import quality_vs_ref
from eval.run_p1c_checks import MODES, N_QUERIES, rankings

P1C_RUN = ROOT.parent / "reports/rounds/P1-C/runs/20261015T014417Z-9aa326b"
P2L_RUN = ROOT.parent / "reports/rounds/P2-L/runs/20261024T184609Z-ced61f9"
P2L_FILES = ("p2l/d_tr.json", "p2l/english.json")


def leaves(x, p=()):
    if isinstance(x, dict):
        for k, v in x.items():
            yield from leaves(v, (*p, k))
    elif isinstance(x, list):
        for i, v in enumerate(x):
            yield from leaves(v, (*p, str(i)))
    else:
        yield "/".join(p), x


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", type=Path, required=True)
    ap.add_argument("--work", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    a = ap.parse_args()
    run = a.run
    c3 = json.loads((run / "c3_quality.json").read_text())
    c10 = json.loads((run / "c10_embedding.json").read_text())
    c11 = json.loads((run / "c1_variant.json").read_text())
    ref_c3 = json.loads((P1C_RUN / "c3_quality.json").read_text())
    c19 = quality_vs_ref(c3, json.loads((ROOT / "data/quality_ref_rust.json").read_text()))
    ln, lo = dict(leaves(c3)), dict(leaves(ref_c3))
    c3_diff = sorted(p for p in set(ln) | set(lo) if ln.get(p) != lo.get(p))

    d3 = rootmark_core.Vault(str(D3))
    verify_d3(d3)
    work = a.work
    if work.exists():
        shutil.rmtree(work)
    work.mkdir(parents=True)
    shutil.copytree(D3 / ".rootmark", work / ".rootmark", ignore=shutil.ignore_patterns("index*"))
    v = rootmark_core.Vault(str(work), matcher=d3.matcher_lock())
    qs = sorted(load_split("test"), key=lambda q: q["id"])[:N_QUERIES]
    queries = [q["question"] for q in qs]
    v.checkpoint("p1c")
    build = v.index_build()
    new = rankings(v, queries, "p1c")
    ref = json.loads((P1C_RUN / "p1c_index.json").read_text())["rankings_200"]
    changes, max_score = {}, {}
    for m in MODES:
        changes[m] = [
            {
                "query": i,
                "question": queries[i],
                "reference": [h[:3] for h in r],
                "new": [h[:3] for h in n],
            }
            for i, (r, n) in enumerate(zip(ref[m], new[m], strict=True))
            if [h[:3] for h in r] != [h[:3] for h in n]
        ]
        max_score[m] = max(
            (abs(h[3] - g[3]) for r, n in zip(ref[m], new[m], strict=True) for h, g in zip(r, n))
            if not changes[m]
            else [float("nan")],
            default=0.0,
        )
    p2l = {}
    for f in P2L_FILES:
        if not (run / f).is_file():
            p2l[f] = "not measured in this RUN"
            continue
        n = dict(leaves(json.loads((run / f).read_text())))
        o = dict(leaves(json.loads((P2L_RUN / f).read_text())))
        p2l[f] = {
            "leaves": len(n),
            "differing": {
                k: [o.get(k), n.get(k)] for k in sorted(set(n) | set(o)) if n.get(k) != o.get(k)
            },
        }
    out = {
        "task": "P2-V.1/E1 (V1-3)",
        "run": run.name,
        "embedder_id": build["embedder_id"],
        "C-3": {
            "values": c3["C-3"],
            "pass": c3["pass"],
            "threshold": c3["threshold_abs_delta_ndcg@10"],
            "vs_p1c_official_run": {"leaves": len(ln), "differing": c3_diff},
        },
        "C-10": {k: c10[k] for k in ("min_cosine", "threshold", "pass", "n", "embedder_id")},
        "C1-1": {"pass": c11["pass"], "cells_pass": {k: v["pass"] for k, v in c11["C1-1"].items()}},
        "C1-9": c19,
        "p2l_vs_p2l_official_run": {"reference_run": P2L_RUN.name, **p2l},
        "rankings_200_vs_p1c_official_run": {
            "reference_run": P1C_RUN.name,
            "queries": len(queries),
            "changed_queries": {m: len(changes[m]) for m in MODES},
            "max_abs_score_delta_unchanged_rankings": max_score,
            "changes": changes,
        },
    }
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n")
    print(json.dumps({k: out[k] for k in ("C-3", "C-10", "C1-1")}, ensure_ascii=False)[:800])
    print("C1-9", c19["compared"], "differing", len(c19["differing"]))
    print(
        "rankings changed",
        out["rankings_200_vs_p1c_official_run"]["changed_queries"],
        "max score delta",
        max_score,
    )


if __name__ == "__main__":
    main()
