"""D1-easy (K4 truth) and D1-hard — v3 synthetic edit sequences (EVAL_DELTA §Veri setleri).

Both builders reuse the frozen v2 generator `eval.datasets.synthetic_edits`
(imported, never copied):

- `d1k4`: the v2 builder is called unchanged (same SEED, same documents, same
  operation sequence) into `data/d1k4/`, so every `v*.md` is byte-identical to
  `data/d1/`. Only `truth.json` differs: each `split`/`merge` step gains a
  `dominant` reference (K4, ARCHITECTURE §10): for a split the longest new part
  (normalised characters; ties → first), which keeps the old lineage; for a
  merge the longest old block, whose lineage the merge result keeps.
- `d1h`: 200 other D3 documents (`random.Random(f"{SEED}:hard")`, disjoint from
  the easy set; tune 50 / test 150), 10 scored steps each, with the harder
  operations of the EVAL_DELTA table. Every split/merge also carries an edit,
  so no instance is an exact recombination.

Truth refers to blocks by content only (`{"version_id", "k"}`); render
round-trip, re-draws and the `redraws` record work exactly as in v2.

    python -m eval.datasets.synthetic_edits_v2 --set d1k4|d1h [--write-manifest]
"""

from __future__ import annotations

import argparse
import json
import random
import re
import shutil
from itertools import pairwise
from pathlib import Path

import mdkernel_core

from eval.datasets import synthetic_edits as se
from eval.datasets.common import (
    DATA,
    SEED,
    die,
    dump_json,
    file_blake3,
    load_manifest,
    verify_or_write_manifest,
)

D1K4_DIR = DATA / "d1k4"
D1H_DIR = DATA / "d1h"
MANIFEST_D1K4 = DATA / "MANIFEST.d1k4.json"
MANIFEST_D1H = DATA / "MANIFEST.d1h.json"

GENERATOR_VERSION_K4 = "d1k4-v3.0"
GENERATOR_VERSION_HARD = "d1h-v3.0"
HARD_SEED = f"{SEED}:hard"
OPS_HARD = [
    ("rephrase_light", 0.15),
    ("rephrase_heavy", 0.15),
    ("rephrase_extreme", 0.10),
    ("split_edit", 0.15),
    ("merge_edit", 0.15),
    ("split_three", 0.05),
    ("move_edit", 0.10),
    ("dup_delete_adjacent", 0.10),
    ("insert", 0.05),
]
SPLIT_OPS = ("split", "split_edit", "split_three")
MERGE_OPS = ("merge", "merge_edit")
LIGHT = (0.05, 0.15)
HEAVY = (0.15, 0.30)
EXTREME = (0.30, 0.45)
SHUFFLE_P = 0.5
# D1-hard eligibility (ASSUMPTIONS T22): the v2 rule leaves only 85 D3
# documents outside the easy sample, so the hard set uses a looser rule; ops
# that are infeasible in a document's current state are re-drawn as in v2.
HARD_MIN_BLOCKS = 4
HARD_MIN_PARAS = 2
HARD_MIN_PROSE = 2


# ------------------------------------------------------------------ helpers


def norm_len(b: se.Blk) -> int:
    """Characters of the block's normalised text (what the matcher measures)."""
    drafts, _ = mdkernel_core.parse_document(b.text + "\n", "x.md")
    if len(drafts) != 1:
        raise ValueError("block does not parse to one block")
    return len(drafts[0]["normalized_text"])


def longest(blocks: list[se.Blk], idx: list[int]) -> int:
    """Index (from `idx`) of the longest block; ties → first."""
    best = idx[0]
    for i in idx[1:]:
        if norm_len(blocks[i]) > norm_len(blocks[best]):
            best = i
    return best


def sentences(b: se.Blk) -> list[str]:
    return se.SENT_RE.split(b.text)


# split_three cuts at sentence or clause boundaries (after `,` / `;`): EVAL_DELTA
# fixes the sentence boundary only for split_edit, and too few paragraphs have
# three sentences to reach the 5 % rate (ASSUMPTIONS T22)
CLAUSE_RE = re.compile(r"(?<=[.!?,;])\s+(?=\S)")


def clauses(b: se.Blk) -> list[str]:
    return CLAUSE_RE.split(b.text)


# ------------------------------------------------------------------ D1-easy (K4)


def k4_truth(doc_dir: Path, truth: dict) -> dict:
    """Adds the K4 `dominant` reference to every split/merge step."""
    for s in truth["steps"]:
        if s["op"] not in SPLIT_OPS + MERGE_OPS:
            continue
        i = int(s["file"][1:3])
        old_blocks, _ = se.parse_blocks((doc_dir / f"v{i - 1:02d}.md").read_text(encoding="utf-8"))
        new_blocks, _ = se.parse_blocks((doc_dir / s["file"]).read_text(encoding="utf-8"))
        s["dominant"] = dominant_ref(s["op"], s["mapping"], old_blocks, new_blocks)
    return truth


def _index(blocks: list[se.Blk], ref: dict) -> int:
    hits = [i for i, b in enumerate(blocks) if b.version_id == ref["version_id"]]
    return hits[ref["k"]]


def dominant_ref(op: str, mapping: list, old: list[se.Blk], new: list[se.Blk]) -> dict:
    if op in SPLIT_OPS:
        ((_o, news),) = mapping
        idx = [_index(new, r) for r in news]
        return se.ref(new, longest(new, idx))
    idx = [_index(old, o) for o, _ in mapping]
    return se.ref(old, longest(old, idx))


def build_d1k4(d3_docs: Path, out: Path) -> dict:
    stats = se.build(d3_docs, out)
    for doc_dir in sorted((out / "docs").iterdir()):
        t = json.loads((doc_dir / "truth.json").read_text(encoding="utf-8"))
        t = k4_truth(doc_dir, t)
        (doc_dir / "truth.json").write_text(
            json.dumps(t, indent=1, sort_keys=True) + "\n", encoding="utf-8"
        )
    return stats


# ------------------------------------------------------------------ D1-hard ops


def _rephrase_at(doc_blocks, i, rng, vocab, rng_frac):
    nb = se.rephrase(doc_blocks[i], rng, rng_frac[0], rng_frac[1], vocab)
    if nb is None or nb.version_id == doc_blocks[i].version_id:
        return None
    return nb


def op_rephrase(doc: se.Doc, rng, vocab, frac, shuffle: bool = False):
    idx = [i for i, b in enumerate(doc.blocks) if se.is_prose(b)]
    if not idx:
        return None
    i = rng.choice(idx)
    nb = _rephrase_at(doc.blocks, i, rng, vocab, frac)
    if nb is None:
        return None
    if shuffle and nb.block_type == "Paragraph" and rng.random() < SHUFFLE_P:
        sents = sentences(nb)
        if len(sents) >= 2:
            order = list(range(len(sents)))
            while order == sorted(order):
                rng.shuffle(order)
            sb = se.make_block("Paragraph", " ".join(sents[k] for k in order))
            if sb is not None and sb.version_id != doc.blocks[i].version_id:
                nb = sb
    new = doc.blocks[:i] + [nb] + doc.blocks[i + 1 :]
    return new, [(i, [i])]


def op_split_n(doc: se.Doc, rng, vocab, parts: int):
    """Split a paragraph into `parts` paragraphs (2: sentence boundaries; 3:
    sentence or clause boundaries), then `rephrase_light` one part."""
    units = sentences if parts == 2 else clauses
    idx = [
        i
        for i, b in enumerate(doc.blocks)
        if b.block_type == "Paragraph" and len(units(b)) >= parts
    ]
    if not idx:
        return None
    i = rng.choice(idx)
    sents = units(doc.blocks[i])
    cuts = sorted(rng.sample(range(1, len(sents)), parts - 1))
    bounds = [0, *cuts, len(sents)]
    pieces = [se.make_block("Paragraph", " ".join(sents[a:b])) for a, b in pairwise(bounds)]
    if any(p is None for p in pieces):
        return None
    e = rng.randrange(parts)
    edited = _rephrase_at(pieces, e, rng, vocab, LIGHT)
    if edited is None:
        return None
    pieces[e] = edited
    if len({p.version_id for p in pieces}) != parts:
        return None
    new = doc.blocks[:i] + pieces + doc.blocks[i + 1 :]
    return new, [(i, list(range(i, i + parts)))]


def op_merge_edit(doc: se.Doc, rng, vocab):
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
    me = _rephrase_at([m], 0, rng, vocab, LIGHT)
    if me is None:
        return None
    new = bl[:i] + [me] + bl[i + 2 :]
    return new, [(i, [i]), (i + 1, [i])]


def op_move_edit(doc: se.Doc, rng, vocab):
    n = len(doc.blocks)
    idx = [i for i, b in enumerate(doc.blocks) if se.is_prose(b)]
    if n < 3 or not idx:
        return None
    i = rng.choice(idx)
    nb = _rephrase_at(doc.blocks, i, rng, vocab, LIGHT)
    if nb is None:
        return None
    rest = doc.blocks[:i] + doc.blocks[i + 1 :]
    j = rng.choice([p for p in range(n) if p != i])
    new = rest[:j] + [nb] + rest[j:]
    return new, [(i, [j])]


def dup_prepare_adjacent(doc: se.Doc, rng):
    """Insert 3 adjacent copies of a short note (not scored)."""
    present = {b.version_id for b in doc.blocks}
    notes = [se.make_block("Paragraph", t) for t in se.NOTES]
    notes = [x for x in notes if x is not None and x.version_id not in present]
    if not notes:
        return None
    note = rng.choice(notes)
    s = rng.randrange(len(doc.blocks) + 1)
    new = doc.blocks[:s] + [note, note, note] + doc.blocks[s:]
    return new, [s, s + 1, s + 2]


def op_dup_delete_adjacent(doc: se.Doc, pos: list[int]):
    first, mid, third = pos
    new = doc.blocks[:mid] + doc.blocks[mid + 1 :]
    return new, [(first, [first]), (third, [third - 1]), (mid, [])]


def draw_hard(rng: random.Random) -> str:
    x = rng.random()
    acc = 0.0
    for name, p in OPS_HARD:
        acc += p
        if x < acc:
            return name
    return OPS_HARD[-1][0]


def _step(op, old, new, pairs):
    s = {"op": op, "scored": True, "mapping": se.mapping_refs(old, new, pairs)}
    if op in SPLIT_OPS + MERGE_OPS:
        s["dominant"] = dominant_ref(op, s["mapping"], old, new)
    return s


def generate_hard_doc(
    doc_path: str, text: str, pool: list[se.Blk], out_dir: Path, n_steps: int = se.STEPS
) -> dict:
    rng = random.Random(f"{HARD_SEED}:{doc_path}")
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
    scored = 0
    redraws: list[dict] = []
    fns = {
        "rephrase_light": lambda: op_rephrase(doc, rng, vocab, LIGHT),
        "rephrase_heavy": lambda: op_rephrase(doc, rng, vocab, HEAVY),
        "rephrase_extreme": lambda: op_rephrase(doc, rng, vocab, EXTREME, shuffle=True),
        "split_edit": lambda: op_split_n(doc, rng, vocab, 2),
        "split_three": lambda: op_split_n(doc, rng, vocab, 3),
        "merge_edit": lambda: op_merge_edit(doc, rng, vocab),
        "move_edit": lambda: op_move_edit(doc, rng, vocab),
        "insert": lambda: se.op_insert(doc, rng, vocab, pool),
    }
    while scored < n_steps:
        op = draw_hard(rng)
        for _attempt in range(se.MAX_TRIES):
            if op == "dup_delete_adjacent":
                prep = dup_prepare_adjacent(doc, rng)
                if prep is None:
                    continue
                pblocks, pos = prep
                ptext = se.check_render(pblocks, doc.fm)
                if ptext is None:
                    continue
                dblocks, pairs = op_dup_delete_adjacent(se.Doc(pblocks, doc.fm), pos)
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
                steps.append(_step(op, pblocks, dblocks, pairs))
                versions.append(dtext)
                doc.blocks = dblocks
                break
            res = fns[op]()
            if res is None:
                continue
            nblocks, pairs = res
            ntext = se.check_render(nblocks, doc.fm)
            if ntext is None:
                continue
            steps.append(_step(op, doc.blocks, nblocks, pairs))
            versions.append(ntext)
            doc.blocks = nblocks
            break
        else:
            redraws.append({"after_step": len(steps), "op": op})
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


def hard_eligible(text: str) -> bool:
    try:
        blocks, fm = se.parse_blocks(text)
    except RuntimeError:
        return False
    paras = [i for i, b in enumerate(blocks) if b.block_type == "Paragraph"]
    return (
        len(blocks) >= HARD_MIN_BLOCKS
        and len(paras) >= HARD_MIN_PARAS
        and sum(1 for b in blocks if se.is_prose(b)) >= HARD_MIN_PROSE
        and any(len(se.SENT_RE.split(blocks[i].text)) >= 2 for i in paras)
        and se.check_render(blocks, fm) is not None
    )


def easy_sources() -> list[str]:
    """The D1-easy documents, re-derived from the v2 selection rule."""
    d3_docs = DATA / "d3" / "docs"
    texts = {
        p.relative_to(d3_docs).as_posix(): p.read_text(encoding="utf-8")
        for p in sorted(d3_docs.rglob("*.md"))
    }
    cands = sorted(p for p, t in texts.items() if se.eligible(t))
    return random.Random(SEED).sample(cands, se.N_DOCS)


def build_d1h(d3_docs: Path, out: Path) -> dict:
    texts = {
        p.relative_to(d3_docs).as_posix(): p.read_text(encoding="utf-8")
        for p in sorted(d3_docs.rglob("*.md"))
    }
    easy = set(easy_sources())
    cands = sorted(p for p, t in texts.items() if p not in easy and hard_eligible(t))
    if len(cands) < se.N_DOCS:
        raise RuntimeError(f"only {len(cands)} hard-eligible D3 documents (< {se.N_DOCS})")
    chosen = random.Random(HARD_SEED).sample(cands, se.N_DOCS)
    if set(chosen) & easy:
        raise RuntimeError("D1-hard intersects D1-easy")
    pool = se.insert_pool(texts, set(chosen))
    if out.exists():
        shutil.rmtree(out)
    docs = []
    for i, path in enumerate(chosen):
        doc_id = f"{i:03d}"
        generate_hard_doc(path, texts[path], pool, out / "docs" / doc_id)
        docs.append({"id": doc_id, "source": path, "split": "tune" if i < se.N_TUNE else "test"})
    splits = {
        "tune": [d["id"] for d in docs if d["split"] == "tune"],
        "test": [d["id"] for d in docs if d["split"] == "test"],
    }
    if set(splits["tune"]) & set(splits["test"]):
        raise RuntimeError("tune/test overlap")
    (out / "docs.json").write_text(dump_json(docs), encoding="utf-8")
    (out / "splits.json").write_text(dump_json(splits), encoding="utf-8")
    return {"eligible": len(cands), "pool": len(pool)}


# ------------------------------------------------------------------ manifests


def _files(out: Path) -> dict:
    return {
        p.relative_to(out).as_posix(): file_blake3(p) for p in sorted(out.rglob("*")) if p.is_file()
    }


def manifest_d1k4(out: Path, stats: dict) -> dict:
    d3 = load_manifest(DATA / "MANIFEST.d3.json")
    return {
        "generator_version": GENERATOR_VERSION_K4,
        "base_generator_version": se.GENERATOR_VERSION,
        "seed": SEED,
        "n_docs": se.N_DOCS,
        "n_tune": se.N_TUNE,
        "steps": se.STEPS,
        "ops": dict(se.OPS),
        "truth_semantics": "K4",
        "d3_events_hash": d3["events_hash"],
        "eligible_docs": stats["eligible"],
        "insert_pool": stats["pool"],
        "files": _files(out),
    }


def manifest_d1h(out: Path, stats: dict) -> dict:
    d3 = load_manifest(DATA / "MANIFEST.d3.json")
    return {
        "generator_version": GENERATOR_VERSION_HARD,
        "seed": HARD_SEED,
        "n_docs": se.N_DOCS,
        "n_tune": se.N_TUNE,
        "steps": se.STEPS,
        "ops": dict(OPS_HARD),
        "truth_semantics": "K4",
        "eligibility": {
            "min_blocks": HARD_MIN_BLOCKS,
            "min_paragraphs": HARD_MIN_PARAS,
            "min_prose": HARD_MIN_PROSE,
            "multi_sentence_paragraph": True,
            "excludes_d1_easy": True,
        },
        "d3_events_hash": d3["events_hash"],
        "eligible_docs": stats["eligible"],
        "insert_pool": stats["pool"],
        "files": _files(out),
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--set", choices=["d1k4", "d1h"], required=True)
    ap.add_argument("--write-manifest", action="store_true")
    a = ap.parse_args(argv)
    d3_docs = DATA / "d3" / "docs"
    if not d3_docs.is_dir():
        die("data/d3/docs missing; run `make d3` first")
    if a.set == "d1k4":
        stats = build_d1k4(d3_docs, D1K4_DIR)
        verify_or_write_manifest(MANIFEST_D1K4, manifest_d1k4(D1K4_DIR, stats), a.write_manifest)
    else:
        stats = build_d1h(d3_docs, D1H_DIR)
        verify_or_write_manifest(MANIFEST_D1H, manifest_d1h(D1H_DIR, stats), a.write_manifest)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(str(e))
