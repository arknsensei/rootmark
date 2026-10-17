"""D1 — synthetic edit sequences over D3 documents (EVAL §D1, H1-S).

Each selected document is rendered block by block (`v00.md`), then edited 10
times with one operation per step. Truth refers to blocks by content only:
`{"version_id": <32 hex>, "k": <occurrence index>}`; no lineage id ever
appears. Every rendered version is re-parsed with the Rust parser and must
reproduce exactly the intended block list, otherwise the operation instance is
re-drawn — so truth and parser can never disagree silently.

    python -m eval.datasets.synthetic_edits [--write-manifest]
"""

from __future__ import annotations

import argparse
import json
import random
import re
import shutil
from dataclasses import dataclass
from pathlib import Path

import rootmark_core

from eval.datasets.common import (
    DATA,
    SEED,
    die,
    dump_json,
    file_blake3,
    load_manifest,
    verify_or_write_manifest,
)

GENERATOR_VERSION = "d1-v2.0"
N_DOCS = 200
N_TUNE = 50
STEPS = 10
OPS = [
    ("rephrase_light", 0.25),
    ("rephrase_heavy", 0.15),
    ("split", 0.10),
    ("merge", 0.10),
    ("move", 0.15),
    ("insert", 0.10),
    ("delete", 0.05),
    ("dup_delete", 0.10),
]
NOTES = [
    "Note: see the linked page for details.",
    "TODO: review this section.",
    "See also the related notes below.",
    "This section is still a draft.",
    "Check the glossary for definitions.",
    "Last reviewed by the maintainers.",
]
WORD_RE = re.compile(r"^([^A-Za-z]*)([A-Za-z]{3,})([^A-Za-z]*)$")
SENT_RE = re.compile(r"(?<=[.!?])\s+(?=[A-Z])")
MAX_TRIES = 50
MAX_REDRAWS = 30

D1_DIR = DATA / "d1"
MANIFEST = DATA / "MANIFEST.d1.json"


@dataclass(frozen=True)
class Blk:
    block_type: str
    text: str
    version_id: str


def parse_blocks(text: str) -> tuple[list[Blk], str | None]:
    drafts, fm = rootmark_core.parse_document(text, "x.md")
    return [Blk(d["block_type"], d["text"], d["version_id"]) for d in drafts], fm


def render(blocks: list[Blk], fm: str | None) -> str:
    body = "\n\n".join(b.text for b in blocks) + "\n"
    return f"---\n{fm}---\n\n{body}" if fm is not None else body


def ref(blocks: list[Blk], i: int) -> dict:
    vid = blocks[i].version_id
    k = sum(1 for b in blocks[:i] if b.version_id == vid)
    return {"version_id": vid, "k": k}


def make_block(block_type: str, text: str) -> Blk | None:
    """A block as the parser would see it, or None if `text` does not parse to one block of `block_type`."""
    got, _ = parse_blocks(text + "\n")
    if len(got) != 1 or got[0].block_type != block_type:
        return None
    return got[0]


def words_of(text: str) -> list[str]:
    return text.split()


def is_prose(b: Blk, min_words: int = 10) -> bool:
    return b.block_type in ("Paragraph", "ListItem") and len(words_of(b.text)) >= min_words


def rephrase(b: Blk, rng: random.Random, lo: float, hi: float, vocab: list[str]) -> Blk | None:
    toks = b.text.split(" ")
    cand = [i for i, t in enumerate(toks) if WORD_RE.match(t) and "[[" not in t and "]]" not in t]
    n_words = len(words_of(b.text))
    if not cand:
        return None
    frac = rng.uniform(lo, hi)
    n = min(len(cand), max(1, round(frac * n_words)))
    for i in rng.sample(cand, n):
        pre, w, post = WORD_RE.match(toks[i]).groups()
        choices = [v for v in vocab if v.lower() != w.lower()]
        toks[i] = pre + rng.choice(choices) + post
    return make_block(b.block_type, " ".join(toks))


class Doc:
    def __init__(self, blocks: list[Blk], fm: str | None):
        self.blocks = blocks
        self.fm = fm


def op_rephrase(doc: Doc, rng, vocab, heavy: bool):
    idx = [i for i, b in enumerate(doc.blocks) if is_prose(b)]
    if not idx:
        return None
    i = rng.choice(idx)
    lo, hi = (0.15, 0.30) if heavy else (0.05, 0.15)
    nb = rephrase(doc.blocks[i], rng, lo, hi, vocab)
    if nb is None or nb.version_id == doc.blocks[i].version_id:
        return None
    new = doc.blocks[:i] + [nb] + doc.blocks[i + 1 :]
    return new, [(i, [i])]


def op_split(doc: Doc, rng, vocab):
    idx = [
        i
        for i, b in enumerate(doc.blocks)
        if b.block_type == "Paragraph" and len(SENT_RE.split(b.text)) >= 2
    ]
    if not idx:
        return None
    i = rng.choice(idx)
    sents = SENT_RE.split(doc.blocks[i].text)
    cut = rng.randint(1, len(sents) - 1)
    a = make_block("Paragraph", " ".join(sents[:cut]))
    b = make_block("Paragraph", " ".join(sents[cut:]))
    if a is None or b is None:
        return None
    new = doc.blocks[:i] + [a, b] + doc.blocks[i + 1 :]
    return new, [(i, [i, i + 1])]


def op_merge(doc: Doc, rng, vocab):
    bl = doc.blocks
    idx = [
        i
        for i in range(len(bl) - 1)
        if bl[i].block_type == "Paragraph" and bl[i + 1].block_type == "Paragraph"
    ]
    if not idx:
        return None
    i = rng.choice(idx)
    m = make_block("Paragraph", bl[i].text.rstrip() + " " + bl[i + 1].text.lstrip())
    if m is None:
        return None
    new = bl[:i] + [m] + bl[i + 2 :]
    return new, [(i, [i]), (i + 1, [i])]


def op_move(doc: Doc, rng, vocab):
    n = len(doc.blocks)
    if n < 3:
        return None
    i = rng.randrange(n)
    rest = doc.blocks[:i] + doc.blocks[i + 1 :]
    j = rng.choice([p for p in range(n) if p != i])
    new = rest[:j] + [doc.blocks[i]] + rest[j:]
    return new, [(i, [j])]


def op_insert(doc: Doc, rng, vocab, pool: list[Blk]):
    present = {b.version_id for b in doc.blocks}
    for _ in range(MAX_TRIES):
        nb = rng.choice(pool)
        if nb.version_id not in present:
            break
    else:
        return None
    j = rng.randrange(len(doc.blocks) + 1)
    new = doc.blocks[:j] + [nb] + doc.blocks[j:]
    return new, [(None, [j])]


def op_delete(doc: Doc, rng, vocab):
    if len(doc.blocks) < 3:
        return None
    i = rng.randrange(len(doc.blocks))
    return doc.blocks[:i] + doc.blocks[i + 1 :], [(i, [])]


def dup_prepare(doc: Doc, rng):
    """Insert 3 copies of a short block at 3 non-adjacent places (not scored)."""
    present = {b.version_id for b in doc.blocks}
    notes = [make_block("Paragraph", t) for t in NOTES]
    notes = [x for x in notes if x is not None and x.version_id not in present]
    n = len(doc.blocks)
    if not notes or n < 4:
        return None
    note = rng.choice(notes)
    # insertion slots in the original sequence; distinct slots ≥ 2 apart keep
    # every copy separated by at least one original block
    for _ in range(MAX_TRIES):
        slots = sorted(rng.sample(range(n + 1), 3))
        if slots[1] - slots[0] >= 2 and slots[2] - slots[1] >= 2:
            break
    else:
        return None
    new = list(doc.blocks)
    for s in reversed(slots):
        new.insert(s, note)
    pos = [s + k for k, s in enumerate(slots)]
    return new, pos, note


def op_dup_delete(doc: Doc, pos: list[int]):
    first, second, third = pos
    new = doc.blocks[:first] + doc.blocks[first + 1 :]
    return new, [(second, [second - 1]), (third, [third - 1]), (first, [])]


def check_render(blocks: list[Blk], fm: str | None) -> str | None:
    text = render(blocks, fm)
    got, got_fm = parse_blocks(text)
    if [b.version_id for b in got] != [b.version_id for b in blocks] or got_fm != fm:
        return None
    return text


def mapping_refs(old: list[Blk], new: list[Blk], pairs) -> list:
    return [[None if o is None else ref(old, o), [ref(new, j) for j in ns]] for o, ns in pairs]


def draw_op(rng: random.Random) -> str:
    x = rng.random()
    acc = 0.0
    for name, p in OPS:
        acc += p
        if x < acc:
            return name
    return OPS[-1][0]


def generate_doc(
    doc_path: str, text: str, pool: list[Blk], out_dir: Path, n_steps: int = STEPS
) -> dict:
    rng = random.Random(f"{SEED}:{doc_path}")
    blocks, fm = parse_blocks(text)
    base = check_render(blocks, fm)
    if base is None:
        raise ValueError(f"{doc_path}: block rendering does not round-trip")
    vocab = sorted(
        {m.group(2) for t in base.split() if (m := WORD_RE.match(t)) and len(m.group(2)) >= 4}
    )
    doc = Doc(blocks, fm)
    out_dir.mkdir(parents=True, exist_ok=True)
    versions = [base]
    steps = []
    scored = 0
    redraws: list[dict] = []
    while scored < n_steps:
        op = draw_op(rng)
        for _attempt in range(MAX_TRIES):
            if op == "dup_delete":
                prep = dup_prepare(doc, rng)
                if prep is None:
                    res = None
                else:
                    pblocks, pos, _note = prep
                    ptext = check_render(pblocks, doc.fm)
                    res = None
                    if ptext is not None:
                        dblocks, pairs = op_dup_delete(Doc(pblocks, doc.fm), pos)
                        dtext = check_render(dblocks, doc.fm)
                        if dtext is not None:
                            prep_pairs = [(None, [p]) for p in pos]
                            steps.append(
                                {
                                    "op": "dup_prepare",
                                    "scored": False,
                                    "mapping": mapping_refs(doc.blocks, pblocks, prep_pairs),
                                }
                            )
                            versions.append(ptext)
                            steps.append(
                                {
                                    "op": op,
                                    "scored": True,
                                    "mapping": mapping_refs(pblocks, dblocks, pairs),
                                }
                            )
                            versions.append(dtext)
                            doc.blocks = dblocks
                            res = True
                if res:
                    break
                continue
            fn = {
                "rephrase_light": lambda: op_rephrase(doc, rng, vocab, heavy=False),
                "rephrase_heavy": lambda: op_rephrase(doc, rng, vocab, heavy=True),
                "split": lambda: op_split(doc, rng, vocab),
                "merge": lambda: op_merge(doc, rng, vocab),
                "move": lambda: op_move(doc, rng, vocab),
                "insert": lambda: op_insert(doc, rng, vocab, pool),
                "delete": lambda: op_delete(doc, rng, vocab),
            }[op]
            res = fn()
            if res is None:
                continue
            nblocks, pairs = res
            ntext = check_render(nblocks, doc.fm)
            if ntext is None:
                continue
            steps.append(
                {"op": op, "scored": True, "mapping": mapping_refs(doc.blocks, nblocks, pairs)}
            )
            versions.append(ntext)
            doc.blocks = nblocks
            break
        else:
            # the op is infeasible in the document's current state (e.g. no
            # adjacent paragraphs left to merge): draw another op
            redraws.append({"after_step": len(steps), "op": op})
            if len(redraws) > MAX_REDRAWS:
                raise ValueError(f"{doc_path}: more than {MAX_REDRAWS} infeasible op draws")
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


# ---------------------------------------------------------------- selection

MIN_BLOCKS = 8


def eligible(text: str) -> bool:
    try:
        blocks, fm = parse_blocks(text)
    except RuntimeError:
        return False
    paras = [i for i, b in enumerate(blocks) if b.block_type == "Paragraph"]
    return (
        len(blocks) >= MIN_BLOCKS
        and len(paras) >= 3
        and sum(1 for b in blocks if is_prose(b)) >= 2
        and any(len(SENT_RE.split(blocks[i].text)) >= 2 for i in paras)
        and any(blocks[i + 1].block_type == "Paragraph" for i in paras if i + 1 < len(blocks))
        and check_render(blocks, fm) is not None
    )


def insert_pool(texts: dict[str, str], exclude: set[str]) -> list[Blk]:
    """Paragraphs (8–60 words) of documents outside the sample, unique by version."""
    seen: dict[str, Blk] = {}
    for path in sorted(texts):
        if path in exclude:
            continue
        for b in parse_blocks(texts[path])[0]:
            if b.block_type == "Paragraph" and 8 <= len(words_of(b.text)) <= 60:
                seen.setdefault(b.version_id, b)
    return [seen[k] for k in sorted(seen)]


def build(d3_docs: Path, out: Path) -> dict:
    texts = {
        p.relative_to(d3_docs).as_posix(): p.read_text(encoding="utf-8")
        for p in sorted(d3_docs.rglob("*.md"))
    }
    cands = sorted(p for p, t in texts.items() if eligible(t))
    if len(cands) < N_DOCS:
        raise RuntimeError(f"only {len(cands)} eligible D3 documents (< {N_DOCS})")
    rng = random.Random(SEED)
    chosen = rng.sample(cands, N_DOCS)
    pool = insert_pool(texts, set(chosen))
    if out.exists():
        shutil.rmtree(out)
    docs = []
    for i, path in enumerate(chosen):
        doc_id = f"{i:03d}"
        generate_doc(path, texts[path], pool, out / "docs" / doc_id)
        docs.append({"id": doc_id, "source": path, "split": "tune" if i < N_TUNE else "test"})
    splits = {
        "tune": [d["id"] for d in docs if d["split"] == "tune"],
        "test": [d["id"] for d in docs if d["split"] == "test"],
    }
    if set(splits["tune"]) & set(splits["test"]):
        raise RuntimeError("tune/test overlap")
    (out / "docs.json").write_text(dump_json(docs), encoding="utf-8")
    (out / "splits.json").write_text(dump_json(splits), encoding="utf-8")
    return {"eligible": len(cands), "pool": len(pool)}


def manifest_for(out: Path, stats: dict) -> dict:
    files = {
        p.relative_to(out).as_posix(): file_blake3(p) for p in sorted(out.rglob("*")) if p.is_file()
    }
    d3 = load_manifest(DATA / "MANIFEST.d3.json")
    return {
        "generator_version": GENERATOR_VERSION,
        "seed": SEED,
        "n_docs": N_DOCS,
        "n_tune": N_TUNE,
        "steps": STEPS,
        "ops": dict(OPS),
        "d3_events_hash": d3["events_hash"],
        "eligible_docs": stats["eligible"],
        "insert_pool": stats["pool"],
        "files": files,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write-manifest", action="store_true")
    a = ap.parse_args(argv)
    d3_docs = DATA / "d3" / "docs"
    if not d3_docs.is_dir():
        die("data/d3/docs missing; run `make d3` first")
    stats = build(d3_docs, D1_DIR)
    verify_or_write_manifest(MANIFEST, manifest_for(D1_DIR, stats), a.write_manifest)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(str(e))
