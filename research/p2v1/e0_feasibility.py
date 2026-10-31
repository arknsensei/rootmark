"""P2-V.1/E0 (TASKS E0, GOAL V1-1): feasibility of processor-independent
embedding vectors, from raw binary32 vectors produced on different
processors.

Inputs
- ``--ci DIR``: the artifacts of ``.github/workflows/e0-feasibility.yml``
  (``DIR/e0-<label>/{cpu.json,<model>/{ort.f32,det.f32,meta.json}}``);
- ``--v4b DIR``: P2-V V4b's two chains (``DIR/{A,B}/run/...``; run
  40613409396), ONNX Runtime vectors of two x86_64 processors;
- ``--ref``: ``research/data/embed_ref.json`` (GOAL P1-C C-10 reference) and
  ``--ref-vectors DIR/<model>/{ort.f32,det.f32}``: the 50 reference texts
  embedded both ways (``e0_embed`` JSON mode).

Candidates (GOAL Kapsam 1): (y) ORT vectors rounded to a grid 2^-k,
k = 10..20; (s) pooling/normalization in fixed-order Rust (the product
already pools in Rust: equal to the ORT rows); (k) pinning ORT/MLAS kernels
(not configurable, see ``feasibility.md``); (r) the whole forward pass in
fixed-order Rust (``index/src/detbert.rs``); (r)+(y).

Expected grid-boundary collisions (GOAL V1-1, K-P2V.4): components × mean
|δ| / step, at the E0 corpus and at 10^5 documents (384 × 10^5 × the E0
corpus' mean chunks per document).

Writes ``--out`` (JSON); prints a short summary.
"""

from __future__ import annotations

import argparse
import itertools
import json
from pathlib import Path

import numpy as np

DIM = 384
MODELS = ("minilm-l6-v2", "multilingual-minilm-l12-v2")
STEPS = list(range(10, 21))
SCALE_DOCS = 100_000


def load(p: Path) -> np.ndarray:
    a = np.fromfile(p, dtype="<f4")
    if a.size % DIM:
        raise ValueError(f"{p}: {a.size} values is not a multiple of {DIM}")
    return a.reshape(-1, DIM)


def delta_stats(a: np.ndarray, b: np.ndarray) -> dict:
    if a.shape != b.shape:
        raise ValueError(f"shapes {a.shape} != {b.shape}")
    d = np.abs(a.astype(np.float64) - b.astype(np.float64))
    diff = a.view(np.uint32) != b.view(np.uint32)
    return {
        "components": int(a.size),
        "vectors": int(a.shape[0]),
        "differing_components": int(diff.sum()),
        "differing_vectors": int(diff.any(axis=1).sum()),
        "max_abs_delta": float(d.max()),
        "mean_abs_delta": float(d.mean()),
        "nonzero_fraction": float((d > 0).mean()),
    }


def rounded(a: np.ndarray, k: int) -> np.ndarray:
    step = 2.0**-k
    return np.rint(a.astype(np.float64) / step)


def rounding_rows(a: np.ndarray, b: np.ndarray, mean_abs: float, n_scale: int) -> list[dict]:
    rows = []
    for k in STEPS:
        ra, rb = rounded(a, k), rounded(b, k)
        diff = ra != rb
        step = 2.0**-k
        rows.append(
            {
                "k": k,
                "step": step,
                "differing_components": int(diff.sum()),
                "differing_vectors": int(diff.any(axis=1).sum()),
                "expected_collisions_e0": a.size * mean_abs / step,
                "expected_collisions_1e5_docs": n_scale * mean_abs / step,
            }
        )
    return rows


def unit(v: np.ndarray) -> np.ndarray:
    return v / np.linalg.norm(v, axis=1, keepdims=True)


def min_cos(a: np.ndarray, b: np.ndarray) -> float:
    return float((unit(a.astype(np.float64)) * unit(b.astype(np.float64))).sum(axis=1).min())


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ci", type=Path, required=True)
    ap.add_argument("--v4b", type=Path, required=True)
    ap.add_argument("--ref", type=Path, required=True)
    ap.add_argument("--ref-vectors", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()

    runners = {}
    for d in sorted(args.ci.glob("e0-*")):
        cpu = json.loads((d / "cpu.json").read_text())
        runners[cpu["label"]] = {"dir": d, "cpu": cpu}
    if len(runners) < 3:
        raise SystemExit(f"expected the e0 artifacts of 4 runners, found {sorted(runners)}")

    ref = json.loads(args.ref.read_text())
    ref_vecs = np.array([c["vector"] for c in ref["chunks"]], dtype=np.float64)

    out: dict = {
        "runners": {k: v["cpu"] | {"cpu_flags": None} for k, v in runners.items()},
        "models": {},
    }
    for model in MODELS:
        vec = {
            lab: {m: load(r["dir"] / model / f"{m}.f32") for m in ("ort", "det")}
            for lab, r in runners.items()
        }
        meta = {
            lab: json.loads((r["dir"] / model / "meta.json").read_text())
            for lab, r in runners.items()
        }
        ids = {lab: m["record_ids"] for lab, m in meta.items()}
        first = next(iter(ids.values()))
        if any(v != first for v in ids.values()):
            raise SystemExit(f"{model}: runners embedded different record lists")
        n_records = len(first)
        docs = 401
        chunks_per_doc = n_records / docs
        n_scale = int(DIM * SCALE_DOCS * chunks_per_doc)
        m_out: dict = {
            "records": n_records,
            "tokens": int(sum(next(iter(meta.values()))["tokens"])),
            "chunks_per_doc": chunks_per_doc,
            "components_1e5_docs": n_scale,
            "timings": {lab: {"ort_s": m["ort_s"], "det_s": m["det_s"]} for lab, m in meta.items()},
            "pairs": {},
        }
        for a, b in itertools.combinations(sorted(runners), 2):
            same_cpu = runners[a]["cpu"]["cpu_model"] == runners[b]["cpu"]["cpu_model"]
            ort = delta_stats(vec[a]["ort"], vec[b]["ort"])
            det = delta_stats(vec[a]["det"], vec[b]["det"])
            m_out["pairs"][f"{a}~{b}"] = {
                "same_cpu_model": same_cpu,
                "ort": ort,
                "y_rounded_ort": rounding_rows(
                    vec[a]["ort"], vec[b]["ort"], ort["mean_abs_delta"], n_scale
                ),
                "r_det": det,
                "r_expected_collisions_1e5_docs": n_scale * det["mean_abs_delta"],
            }
        # quality: det against ORT on the E0 corpus (per runner), C-10 against
        # the sentence-transformers reference
        m_out["det_vs_ort_min_cosine"] = {
            lab: min_cos(v["det"], v["ort"]) for lab, v in vec.items()
        }
        rv = {m: load(args.ref_vectors / model / f"{m}.f32") for m in ("ort", "det")}
        if model == "minilm-l6-v2":
            c10 = {
                "ort": min_cos(rv["ort"], ref_vecs),
                "det": min_cos(rv["det"], ref_vecs),
                "y_rounded_ort": {
                    k: min_cos(rounded(rv["ort"], k) * 2.0**-k, ref_vecs) for k in STEPS
                },
                "threshold": 0.999,
            }
            m_out["c10_min_cosine"] = c10
        out["models"][model] = m_out

    # V4b: ORT on two x86_64 processors (product chains, small vaults)
    rels = sorted(
        str(p.relative_to(args.v4b / "A" / "run"))
        for p in (args.v4b / "A" / "run").rglob("vectors.f32")
    )
    seen = {}
    for rel in rels:
        a = (args.v4b / "A" / "run" / rel).read_bytes()
        if a in seen.values():
            continue
        seen[rel] = a
    a_all = np.concatenate([load(args.v4b / "A" / "run" / r) for r in seen])
    b_all = np.concatenate([load(args.v4b / "B" / "run" / r) for r in seen])
    v4 = delta_stats(a_all, b_all)
    n_scale_v4 = out["models"]["minilm-l6-v2"]["components_1e5_docs"]
    out["v4b"] = {
        "run": 40613409396,
        "files": sorted(seen),
        "ort": v4,
        "y_rounded_ort": rounding_rows(a_all, b_all, v4["mean_abs_delta"], n_scale_v4),
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n")

    for model, m in out["models"].items():
        print(f"== {model}: {m['records']} records, {m['tokens']} tokens")
        for pair, p in m["pairs"].items():
            best = min(p["y_rounded_ort"], key=lambda r: r["expected_collisions_1e5_docs"])
            print(
                f"  {pair} same_cpu={p['same_cpu_model']} ort diff {p['ort']['differing_components']}"
                f" mean|d| {p['ort']['mean_abs_delta']:.3g} | det diff {p['r_det']['differing_components']}"
                f" | best (y) k={best['k']} obs {best['differing_components']}"
                f" exp1e5 {best['expected_collisions_1e5_docs']:.3g}"
            )
    print("v4b", out["v4b"]["ort"])


if __name__ == "__main__":
    main()
