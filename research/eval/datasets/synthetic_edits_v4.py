"""P1-B split-weighted holdout slice `d1hs` (docs/rounds/P1-B/GOAL.md Ö3,
TASKS B8; P1-A verifier recommendation 3).

holdout_split (`data/d1hs/`, MANIFEST.d1hs.json)
    SEED `20261013:holdout-split`; ≥ 60 documents with at least one paragraph
    of ≥ 3 sentences (the D1-hard sentence splitter, `synthetic_edits_v2.
    sentences`); 10 scored steps each from a split-weighted variant of the
    D1-hard op table, with the D1-hard op functions and the frozen generation
    loop of `synthetic_edits_v3.generate` (imported, not copied); K4 truth.
    Realised rates must reach split_edit ≥ 25 % and split_three ≥ 10 % — the
    build fails otherwise.

Source documents (docs/rounds/P1-B/progress/ASSUMPTIONS.md B8-1): D3 holds
only 12 eligible documents outside D1-easy ∪ D1-hard ∪ d1ho (15 with the
holdout's own sources), far below 60, so the bases are the `src/*.md`
chapters of `rust-lang/book` at the D2 revision (`eval/datasets/
git_history.py` REPO/REV, read with `git show <rev>:<path>`, never from the
working tree). No d1hs base text equals any D3 text, so d1hs is disjoint
from D1-easy, D1-hard and d1ho by source and by content (checked).

    python -m eval.datasets.synthetic_edits_v4 [--write-manifest]
"""

from __future__ import annotations

import argparse
import json
import random
from pathlib import Path

from eval.datasets import git_history as gh
from eval.datasets import synthetic_edits as se
from eval.datasets import synthetic_edits_v2 as s2
from eval.datasets import synthetic_edits_v3 as s3
from eval.datasets.common import (
    DATA,
    SEED,
    clone_at,
    die,
    git,
    load_manifest,
    verify_or_write_manifest,
)

GENERATOR_VERSION = "d1hs-p1b.1"
HOLDOUT_SPLIT_SEED = f"{SEED}:holdout-split"
N_DOCS = 60
MIN_SENTENCES = 3
MIN_BLOCKS = 3
MIN_SPLIT_EDIT = 0.25
MIN_SPLIT_THREE = 0.10
DOC_PREFIX = "book/"

OUT = DATA / "d1hs"
MANIFEST = DATA / "MANIFEST.d1hs.json"

# D1-hard op names and functions; split-weighted (fixed before any run)
OPS_HOLDOUT_SPLIT = [
    ("rephrase_light", 0.10),
    ("rephrase_heavy", 0.08),
    ("rephrase_extreme", 0.05),
    ("split_edit", 0.35),
    ("split_three", 0.20),
    ("merge_edit", 0.08),
    ("move_edit", 0.06),
    ("dup_delete_adjacent", 0.05),
    ("insert", 0.03),
]


def book_texts() -> tuple[str, dict[str, str]]:
    """`src/*.md` of rust-lang/book at the D2 revision, keyed `book/src/<f>`."""
    sha = clone_at(gh.REPO, gh.REV, gh.SRC)
    names = git("ls-tree", "--name-only", sha, "--", "src/", cwd=gh.SRC).splitlines()
    texts = {
        DOC_PREFIX + n: git("show", f"{sha}:{n}", cwd=gh.SRC)
        for n in sorted(names)
        if n.endswith(".md") and "/" not in n[len("src/") :]
    }
    return sha, texts


def eligible(text: str) -> bool:
    try:
        blocks, fm = se.parse_blocks(text)
    except RuntimeError:
        return False
    paras = [b for b in blocks if b.block_type == "Paragraph"]
    return (
        len(blocks) >= MIN_BLOCKS
        and any(len(s2.sentences(b)) >= MIN_SENTENCES for b in paras)
        and se.check_render(blocks, fm) is not None
    )


def d3_texts() -> dict[str, str]:
    d3 = DATA / "d3" / "docs"
    if not d3.is_dir():
        raise RuntimeError("data/d3/docs missing; run `make d3` first")
    return s3._texts(d3)


def disjointness(chosen: list[str], texts: dict[str, str]) -> dict:
    """By source path and by base-text content against D1-easy, D1-hard, d1ho."""
    used = s3.used_sources()
    holdout = set(s3.holdout_sources())
    d3 = d3_texts()
    src = set(chosen)
    out = {
        "n": {
            "d1hs": len(src),
            "easy": len(used["easy"]),
            "hard": len(used["hard"]),
            "d1ho": len(holdout),
        },
        "d1hs_and_easy": len(src & used["easy"]),
        "d1hs_and_hard": len(src & used["hard"]),
        "d1hs_and_d1ho": len(src & holdout),
        "d1ho_and_easy": len(holdout & used["easy"]),
        "d1ho_and_hard": len(holdout & used["hard"]),
        "easy_and_hard": len(used["easy"] & used["hard"]),
        "d1hs_texts_equal_to_d3": len({texts[c] for c in chosen} & set(d3.values())),
    }
    if any(v for k, v in out.items() if k != "n"):
        raise RuntimeError(f"d1hs / d1ho / D1-easy / D1-hard are not disjoint: {out}")
    return out


def build(out: Path) -> dict:
    sha, texts = book_texts()
    cands = sorted(p for p, t in texts.items() if eligible(t))
    if len(cands) < N_DOCS:
        raise RuntimeError(f"only {len(cands)} d1hs-eligible documents (< {N_DOCS})")
    chosen = random.Random(HOLDOUT_SPLIT_SEED).sample(cands, N_DOCS)
    dis = disjointness(chosen, texts)
    pool = se.insert_pool(texts, set(chosen))
    s3._write_docs(out, chosen, texts, pool, HOLDOUT_SPLIT_SEED, OPS_HOLDOUT_SPLIT, se.STEPS)
    rates = s3.op_rates(out)
    r = rates["rates"]
    if r.get("split_edit", 0) < MIN_SPLIT_EDIT or r.get("split_three", 0) < MIN_SPLIT_THREE:
        raise RuntimeError(
            f"realised split rates below GOAL Ö3 (split_edit ≥ {MIN_SPLIT_EDIT}, "
            f"split_three ≥ {MIN_SPLIT_THREE}): {r}"
        )
    return {
        "rev": sha,
        "eligible": len(cands),
        "available": len(texts),
        "pool": len(pool),
        "disjoint": dis,
        "realised": rates,
    }


def manifest(out: Path, stats: dict) -> dict:
    return {
        "generator_version": GENERATOR_VERSION,
        "seed": HOLDOUT_SPLIT_SEED,
        "n_docs": N_DOCS,
        "steps": se.STEPS,
        "ops": dict(OPS_HOLDOUT_SPLIT),
        "op_functions": "eval.datasets.synthetic_edits_v2 (D1-hard), loop eval.datasets.synthetic_edits_v3.generate",
        "truth_semantics": "K4",
        "source": {"repo": gh.REPO, "rev": stats["rev"], "pathspec": "src/*.md"},
        "eligibility": {"min_blocks": MIN_BLOCKS, "min_sentences_in_a_paragraph": MIN_SENTENCES},
        "min_realised": {"split_edit": MIN_SPLIT_EDIT, "split_three": MIN_SPLIT_THREE},
        "d3_events_hash": load_manifest(DATA / "MANIFEST.d3.json")["events_hash"],
        "available_docs": stats["available"],
        "eligible_docs": stats["eligible"],
        "insert_pool": stats["pool"],
        "disjoint": stats["disjoint"],
        "realised": stats["realised"],
        "files": s3._files(out),
    }


def sources() -> list[str]:
    docs = json.loads((OUT / "docs.json").read_text(encoding="utf-8"))
    return [d["source"] for d in docs]


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write-manifest", action="store_true")
    a = ap.parse_args(argv)
    stats = build(OUT)
    verify_or_write_manifest(MANIFEST, manifest(OUT, stats), a.write_manifest)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(str(e))
