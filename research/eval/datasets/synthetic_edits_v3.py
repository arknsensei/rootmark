"""P1-A synthetic sets (docs/rounds/P1-A/GOAL.md §Kapsam, TASKS A5).

holdout (`data/d1ho/`, MANIFEST.d1ho.json)
    A fresh test of the frozen v3 matcher: SEED `20261013:holdout`, 100 D3
    documents disjoint from D1-easy and D1-hard, 10 scored steps each with the
    D1-hard op table and the D1-hard op functions (imported, not copied),
    K4 truth. Never used for tuning.

split_hard (`data/d1sh/`, MANIFEST.d1sh.json)
    The split-threshold difficulty zone (v3 verifier recommendation 2):
    `split_edit/both_edited` (two-piece sentence split, both pieces then
    `rephrase_light`), `split_edit/short_piece` (two-piece split whose shorter
    piece has ≤ 8 words, both pieces edited), `merge_edit/heavy` (two adjacent
    paragraphs merged, then ≥ 15 % of the words changed), and
    `dup_delete_adjacent` (as in D1-hard; keeps the frozen objective defined).
    Sources: D3 outside D1-easy ∪ D1-hard (may share base texts with holdout).
    No split piece is a substring of the old block's normalised text
    (checked at generation). Information only (A-7): the THETA_SPLIT grid
    never changes the compiled constants.

    python -m eval.datasets.synthetic_edits_v3 --set holdout|split_hard [--write-manifest]
"""

from __future__ import annotations

import argparse
import json
import random
import shutil
from collections import Counter
from itertools import pairwise
from pathlib import Path

import rootmark_core

from eval.datasets import synthetic_edits as se
from eval.datasets import synthetic_edits_v2 as s2
from eval.datasets.common import (
    DATA,
    SEED,
    die,
    dump_json,
    file_blake3,
    load_manifest,
    verify_or_write_manifest,
)

GENERATOR_VERSION_HOLDOUT = "d1ho-p1a.1"
GENERATOR_VERSION_SPLIT_HARD = "d1sh-p1a.1"
HOLDOUT_SEED = f"{SEED}:holdout"
SPLIT_HARD_SEED = f"{SEED}:split_hard"
N_HOLDOUT = 100
N_SPLIT_HARD = 60
SPLIT_HARD_STEPS = 6
SHORT_MAX_WORDS = 8
MERGE_MIN_EDIT = 0.15

HOLDOUT_DIR = DATA / "d1ho"
SPLIT_HARD_DIR = DATA / "d1sh"
MANIFEST_HOLDOUT = DATA / "MANIFEST.d1ho.json"
MANIFEST_SPLIT_HARD = DATA / "MANIFEST.d1sh.json"

# Holdout eligibility (docs/rounds/P1-A/progress/ASSUMPTIONS.md A5): only 5
# D1-hard-eligible D3 documents remain outside D1-easy ∪ D1-hard, so the
# holdout uses a looser rule; infeasible ops are re-drawn exactly as in D1-hard.
HOLDOUT_MIN_BLOCKS = 3
HOLDOUT_MIN_PARAS = 1
HOLDOUT_MIN_PROSE = 1

OPS_SPLIT_HARD = [
    ("split_both_edited", 0.35),
    ("split_short_piece", 0.30),
    ("merge_heavy", 0.25),
    ("dup_delete_adjacent", 0.10),
]
# truth op name (what eval/metrics_v3.py scores) and variant of each draw
SPLIT_HARD_OP = {
    "split_both_edited": ("split_edit", "both_edited"),
    "split_short_piece": ("split_edit", "short_piece"),
    "merge_heavy": ("merge_edit", "heavy"),
    "dup_delete_adjacent": ("dup_delete_adjacent", None),
}


# ------------------------------------------------------------------ helpers


def _texts(d3_docs: Path) -> dict[str, str]:
    return {
        p.relative_to(d3_docs).as_posix(): p.read_text(encoding="utf-8")
        for p in sorted(d3_docs.rglob("*.md"))
    }


def hard_sources() -> list[str]:
    docs = json.loads((DATA / "d1h" / "docs.json").read_text(encoding="utf-8"))
    return [d["source"] for d in docs]


def holdout_sources() -> list[str]:
    docs = json.loads((HOLDOUT_DIR / "docs.json").read_text(encoding="utf-8"))
    return [d["source"] for d in docs]


def used_sources() -> dict[str, set[str]]:
    """D1-easy (re-derived from the v2 rule) and D1-hard (from its docs.json)."""
    easy = set(s2.easy_sources())
    hard = set(hard_sources())
    if len(easy) != se.N_DOCS or len(hard) != se.N_DOCS:
        raise RuntimeError(f"easy/hard sources: {len(easy)}/{len(hard)} != {se.N_DOCS}")
    return {"easy": easy, "hard": hard}


def holdout_eligible(text: str) -> bool:
    try:
        blocks, fm = se.parse_blocks(text)
    except RuntimeError:
        return False
    return (
        len(blocks) >= HOLDOUT_MIN_BLOCKS
        and sum(1 for b in blocks if b.block_type == "Paragraph") >= HOLDOUT_MIN_PARAS
        and sum(1 for b in blocks if se.is_prose(b)) >= HOLDOUT_MIN_PROSE
        and se.check_render(blocks, fm) is not None
    )


def norm_text(b: se.Blk) -> str:
    drafts, _ = rootmark_core.parse_document(b.text + "\n", "x.md")
    if len(drafts) != 1:
        raise ValueError("block does not parse to one block")
    return drafts[0]["normalized_text"]


def draw(rng: random.Random, table: list[tuple[str, float]]) -> str:
    x = rng.random()
    acc = 0.0
    for name, p in table:
        acc += p
        if x < acc:
            return name
    return table[-1][0]


# ------------------------------------------------------------------ split_hard ops


def _sentence_cuts(b: se.Blk) -> list[str]:
    return s2.sentences(b)


def op_split_both_edited(doc: se.Doc, rng, vocab, short: bool):
    """Two-piece sentence split; both pieces `rephrase_light`; with `short`
    the shorter piece has ≤ SHORT_MAX_WORDS words."""
    cands = []
    for i, b in enumerate(doc.blocks):
        if b.block_type != "Paragraph":
            continue
        sents = _sentence_cuts(b)
        for c in range(1, len(sents)):
            left, right = " ".join(sents[:c]), " ".join(sents[c:])
            shortest = min(len(left.split()), len(right.split()))
            if short and shortest > SHORT_MAX_WORDS:
                continue
            cands.append((i, c))
    if not cands:
        return None
    i, c = rng.choice(cands)
    sents = _sentence_cuts(doc.blocks[i])
    pieces = [
        se.make_block("Paragraph", " ".join(sents[a:z])) for a, z in pairwise([0, c, len(sents)])
    ]
    if any(p is None for p in pieces):
        return None
    for k in range(2):
        edited = s2._rephrase_at(pieces, k, rng, vocab, s2.LIGHT)
        if edited is None:
            return None
        pieces[k] = edited
    old_norm = norm_text(doc.blocks[i])
    if (
        any(norm_text(p) in old_norm for p in pieces)
        or pieces[0].version_id == pieces[1].version_id
    ):
        return None
    present = {b.version_id for k, b in enumerate(doc.blocks) if k != i}
    if any(p.version_id in present for p in pieces):
        return None
    new = doc.blocks[:i] + pieces + doc.blocks[i + 1 :]
    return new, [(i, [i, i + 1])]


def op_merge_heavy(doc: se.Doc, rng, vocab):
    """Merge two adjacent paragraphs, then change ≥ 15 % of the words."""
    bl = doc.blocks
    idx = [
        i
        for i in range(len(bl) - 1)
        if bl[i].block_type == "Paragraph" and bl[i + 1].block_type == "Paragraph"
    ]
    if not idx:
        return None
    i = rng.choice(idx)
    m = se.make_block("Paragraph", bl[i].text.rstrip() + " " + bl[i + 1].text.lstrip())
    if m is None:
        return None
    me = s2._rephrase_at([m], 0, rng, vocab, s2.HEAVY)
    if me is None:
        return None
    before, after = m.text.split(" "), me.text.split(" ")
    changed = sum(1 for a, z in zip(before, after, strict=True) if a != z)
    if changed / len(se.words_of(m.text)) < MERGE_MIN_EDIT:
        return None
    present = {b.version_id for k, b in enumerate(bl) if k not in (i, i + 1)}
    if me.version_id in present:
        return None
    new = bl[:i] + [me] + bl[i + 2 :]
    return new, [(i, [i]), (i + 1, [i])]


# ------------------------------------------------------------------ generator


def generate(
    doc_path: str,
    text: str,
    pool: list[se.Blk],
    out_dir: Path,
    seed: str,
    table: list[tuple[str, float]],
    n_steps: int,
) -> dict:
    """The D1-hard generation loop (eval/datasets/synthetic_edits_v2.py
    `generate_hard_doc`) with the seed and op table as parameters."""
    rng = random.Random(f"{seed}:{doc_path}")
    blocks, fm = se.parse_blocks(text)
    base = se.check_render(blocks, fm)
    if base is None:
        raise ValueError(f"{doc_path}: block rendering does not round-trip")
    vocab = sorted(
        {m.group(2) for t in base.split() if (m := se.WORD_RE.match(t)) and len(m.group(2)) >= 4}
    )
    doc = se.Doc(blocks, fm)
    out_dir.mkdir(parents=True, exist_ok=True)
    versions = [base]
    steps: list[dict] = []
    redraws: list[dict] = []
    scored = 0
    fns = {
        # D1-hard ops (holdout)
        "rephrase_light": lambda: s2.op_rephrase(doc, rng, vocab, s2.LIGHT),
        "rephrase_heavy": lambda: s2.op_rephrase(doc, rng, vocab, s2.HEAVY),
        "rephrase_extreme": lambda: s2.op_rephrase(doc, rng, vocab, s2.EXTREME, shuffle=True),
        "split_edit": lambda: s2.op_split_n(doc, rng, vocab, 2),
        "split_three": lambda: s2.op_split_n(doc, rng, vocab, 3),
        "merge_edit": lambda: s2.op_merge_edit(doc, rng, vocab),
        "move_edit": lambda: s2.op_move_edit(doc, rng, vocab),
        "insert": lambda: se.op_insert(doc, rng, vocab, pool),
        # split_hard ops
        "split_both_edited": lambda: op_split_both_edited(doc, rng, vocab, short=False),
        "split_short_piece": lambda: op_split_both_edited(doc, rng, vocab, short=True),
        "merge_heavy": lambda: op_merge_heavy(doc, rng, vocab),
    }
    while scored < n_steps:
        drawn = draw(rng, table)
        op, variant = SPLIT_HARD_OP.get(drawn, (drawn, None))
        for _attempt in range(se.MAX_TRIES):
            if op == "dup_delete_adjacent":
                prep = s2.dup_prepare_adjacent(doc, rng)
                if prep is None:
                    continue
                pblocks, pos = prep
                ptext = se.check_render(pblocks, doc.fm)
                if ptext is None:
                    continue
                dblocks, pairs = s2.op_dup_delete_adjacent(se.Doc(pblocks, doc.fm), pos)
                dtext = se.check_render(dblocks, doc.fm)
                if dtext is None:
                    continue
                steps.append(
                    {
                        "op": "dup_prepare_adjacent",
                        "scored": False,
                        "mapping": se.mapping_refs(doc.blocks, pblocks, [(None, [p]) for p in pos]),
                    }
                )
                versions.append(ptext)
                steps.append(s2._step(op, pblocks, dblocks, pairs))
                versions.append(dtext)
                doc.blocks = dblocks
                break
            res = fns[drawn]()
            if res is None:
                continue
            nblocks, pairs = res
            ntext = se.check_render(nblocks, doc.fm)
            if ntext is None:
                continue
            step = s2._step(op, doc.blocks, nblocks, pairs)
            if variant is not None:
                step["variant"] = variant
            steps.append(step)
            versions.append(ntext)
            doc.blocks = nblocks
            break
        else:
            redraws.append({"after_step": len(steps), "op": drawn})
            if len(redraws) > se.MAX_REDRAWS:
                raise ValueError(f"{doc_path}: more than {se.MAX_REDRAWS} infeasible op draws")
            continue
        scored += 1
    files = []
    for i, v in enumerate(versions):
        name = f"v{i:02d}.md"
        (out_dir / name).write_text(v, encoding="utf-8")
        files.append(name)
    for i, s in enumerate(steps):
        s["step"] = i + 1
        s["file"] = files[i + 1]
    truth = {
        "doc_path": doc_path,
        "source": doc_path,
        "base_file": files[0],
        "steps": steps,
        "redraws": redraws,
    }
    (out_dir / "truth.json").write_text(
        json.dumps(truth, indent=1, sort_keys=True) + "\n", encoding="utf-8"
    )
    return truth


def _write_docs(out: Path, chosen: list[str], texts, pool, seed, table, n_steps) -> list[dict]:
    if out.exists():
        shutil.rmtree(out)
    docs = []
    for i, path in enumerate(chosen):
        doc_id = f"{i:03d}"
        generate(path, texts[path], pool, out / "docs" / doc_id, seed, table, n_steps)
        docs.append({"id": doc_id, "source": path})
    (out / "docs.json").write_text(dump_json(docs), encoding="utf-8")
    return docs


def op_rates(out: Path) -> dict:
    """Realised scored-op rates (after re-draws) and variants."""
    c: Counter = Counter()
    for t in sorted((out / "docs").glob("*/truth.json")):
        for s in json.loads(t.read_text(encoding="utf-8"))["steps"]:
            if s["scored"]:
                c[s["op"] + (f"/{s['variant']}" if "variant" in s else "")] += 1
    n = sum(c.values())
    return {"n_scored": n, "rates": {k: round(v / n, 6) for k, v in sorted(c.items())}}


def build_holdout(d3_docs: Path, out: Path) -> dict:
    texts = _texts(d3_docs)
    used = used_sources()
    excluded = used["easy"] | used["hard"]
    cands = sorted(p for p, t in texts.items() if p not in excluded and holdout_eligible(t))
    if len(cands) < N_HOLDOUT:
        raise RuntimeError(f"only {len(cands)} holdout-eligible documents (< {N_HOLDOUT})")
    chosen = random.Random(HOLDOUT_SEED).sample(cands, N_HOLDOUT)
    if set(chosen) & excluded:
        raise RuntimeError("holdout intersects D1-easy or D1-hard")
    pool = se.insert_pool(texts, set(chosen))
    _write_docs(out, chosen, texts, pool, HOLDOUT_SEED, s2.OPS_HARD, se.STEPS)
    return {"eligible": len(cands), "pool": len(pool)}


def split_hard_eligible(text: str) -> bool:
    """A two-piece sentence split is possible at step 1 (merges are re-drawn
    when infeasible, as every D1-hard op)."""
    try:
        blocks, fm = se.parse_blocks(text)
    except RuntimeError:
        return False
    paras = [i for i, b in enumerate(blocks) if b.block_type == "Paragraph"]
    multi = any(len(s2.sentences(blocks[i])) >= 2 for i in paras)
    return len(blocks) >= HOLDOUT_MIN_BLOCKS and multi and se.check_render(blocks, fm) is not None


def build_split_hard(d3_docs: Path, out: Path) -> dict:
    texts = _texts(d3_docs)
    used = used_sources()
    # information-only slice: excludes the sets used in v3 design and tuning;
    # base texts may coincide with holdout sources (different seed and edit
    # sequences; ASSUMPTIONS A5) — too few D3 documents remain otherwise
    excluded = used["easy"] | used["hard"]
    cands = sorted(p for p, t in texts.items() if p not in excluded and split_hard_eligible(t))
    n = min(N_SPLIT_HARD, len(cands))
    if n < 20:
        raise RuntimeError(f"only {len(cands)} split_hard-eligible documents (< 20)")
    chosen = random.Random(SPLIT_HARD_SEED).sample(cands, n)
    pool = se.insert_pool(texts, set(chosen))
    _write_docs(out, chosen, texts, pool, SPLIT_HARD_SEED, OPS_SPLIT_HARD, SPLIT_HARD_STEPS)
    return {"eligible": len(cands), "pool": len(pool), "n_docs": n}


def _files(out: Path) -> dict:
    return {
        p.relative_to(out).as_posix(): file_blake3(p) for p in sorted(out.rglob("*")) if p.is_file()
    }


def manifest_holdout(out: Path, stats: dict) -> dict:
    d3 = load_manifest(DATA / "MANIFEST.d3.json")
    return {
        "generator_version": GENERATOR_VERSION_HOLDOUT,
        "seed": HOLDOUT_SEED,
        "n_docs": N_HOLDOUT,
        "steps": se.STEPS,
        "ops": dict(s2.OPS_HARD),
        "op_functions": "eval.datasets.synthetic_edits_v2 (D1-hard)",
        "truth_semantics": "K4",
        "eligibility": {
            "min_blocks": HOLDOUT_MIN_BLOCKS,
            "min_paragraphs": HOLDOUT_MIN_PARAS,
            "min_prose": HOLDOUT_MIN_PROSE,
            "excludes_d1_easy": True,
            "excludes_d1_hard": True,
        },
        "d3_events_hash": d3["events_hash"],
        "eligible_docs": stats["eligible"],
        "insert_pool": stats["pool"],
        "realised": op_rates(out),
        "files": _files(out),
    }


def manifest_split_hard(out: Path, stats: dict) -> dict:
    d3 = load_manifest(DATA / "MANIFEST.d3.json")
    return {
        "generator_version": GENERATOR_VERSION_SPLIT_HARD,
        "seed": SPLIT_HARD_SEED,
        "n_docs": stats["n_docs"],
        "steps": SPLIT_HARD_STEPS,
        "ops": dict(OPS_SPLIT_HARD),
        "short_piece_max_words": SHORT_MAX_WORDS,
        "merge_min_edit": MERGE_MIN_EDIT,
        "truth_semantics": "K4",
        "excludes": ["d1_easy", "d1_hard"],
        "d3_events_hash": d3["events_hash"],
        "eligible_docs": stats["eligible"],
        "insert_pool": stats["pool"],
        "realised": op_rates(out),
        "files": _files(out),
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--set", choices=["holdout", "split_hard"], required=True)
    ap.add_argument("--write-manifest", action="store_true")
    a = ap.parse_args(argv)
    d3_docs = DATA / "d3" / "docs"
    if not d3_docs.is_dir():
        die("data/d3/docs missing; run `make d3` first")
    if a.set == "holdout":
        stats = build_holdout(d3_docs, HOLDOUT_DIR)
        verify_or_write_manifest(
            MANIFEST_HOLDOUT, manifest_holdout(HOLDOUT_DIR, stats), a.write_manifest
        )
    else:
        stats = build_split_hard(d3_docs, SPLIT_HARD_DIR)
        verify_or_write_manifest(
            MANIFEST_SPLIT_HARD, manifest_split_hard(SPLIT_HARD_DIR, stats), a.write_manifest
        )


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(str(e))
