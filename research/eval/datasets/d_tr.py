"""D-TR — the Turkish and Turkish/English set of round P2-L (GOAL P2-L 5,
TASKS L4). No LLM: documents are articles of a pinned Wikipedia dump,
queries are sampled from them with a fixed seed, and each query's truth is
the document it was sampled from. Everything below is fixed before any
measurement (pre-registration, ASSUMPTIONS P2-L L4-*): the source, its
revision and sha256, the selection rules, the seed and this generator.

**Source.** Hugging Face dataset `wikimedia/wikipedia` at revision
`b04c8d1ceb2f5cd4588862100d08de323dccfbaa`, the plain-text dump of
2023-11-01 (`20231101.tr`, `20231101.en`; licence CC BY-SA 3.0 and GFDL —
the texts are Wikipedia's, attributed by their `url`): the Turkish shard
`20231101.tr/train-00000-of-00002.parquet` and the English shard
`20231101.en/train-00000-of-00041.parquet`, each checked against its
sha256 (`SHARDS`) before use. Downloaded into `data/src/d_tr/` (git-ignored).

**Documents** (`data/d_tr/docs/`, git-ignored; their hashes in
`data/MANIFEST.d_tr.json`, committed):
- eligible Turkish article: title without `(` or `:`, text of at least
  `MIN_CHARS` characters in at least `MIN_PARAS` paragraphs (blocks
  separated by a blank line); an eligible English article likewise;
- `N_MIX` = 100 **TR/EN documents**: Turkish articles whose title is the
  title of an eligible English article of the English shard (the same
  subject in both languages: proper names, places, terms), sorted by
  Turkish id, sampled with `random.Random(SEED)`; each written as
  `mix/<tr id>.md`: `# <title>`, the Turkish lead, `## English`, the
  English lead;
- `N_TR` = 300 **Turkish documents**: the other eligible Turkish
  articles, sorted by id, sampled with the same generator after the
  first sample; each `tr/<id>.md`: `# <title>` and the Turkish lead;
- a lead is the article's first paragraphs, whole, until at least
  `LEAD_CHARS` characters (at most `LEAD_MAX` paragraphs).

**Queries** (`data/d_tr/queries.jsonl`; hashed in the manifest), two per
document, in document order, drawn from the same generator:
- `sentence`: a sentence of the document's body (split after `.`, `!`,
  `?` followed by a space; at least `MIN_SENT_WORDS` words), uniformly; a
  window of `WINDOW` = 4–7 consecutive words of it, uniformly; for a TR/EN
  document the sentence is drawn from the Turkish or the English lead with
  equal probability (`lang` records which); the words as written;
- `title`: the article's title, as written.
Each query has `id` (`<doc>#<type>`), `type`, `lang` (`tr` or `en`),
`query` and `truth` (the document path). A query whose text is empty
after stripping, or that duplicates another query's text, is an error
(fail fast: the set is pre-registered as it is).

**Metrics** (doc level, `eval.run_p2l`): the chunk hits of a search with
`k = K_CHUNKS` collapsed to their documents in rank order; with `r` the
rank of the truth document among them, nDCG@10 = `1/log2(r + 1)` for
`r ≤ 10` (one relevant document), recall@10 = `1` for `r ≤ 10`, MRR =
`1/r` (`0` when absent).

    python -m eval.datasets.d_tr            # download (if missing), build, write the manifest
    python -m eval.datasets.d_tr --check    # rebuild and compare with the committed manifest
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import re
import shutil
import urllib.request
from pathlib import Path

import blake3

from eval.datasets.common import DATA, die, dump_json

DATASET = "wikimedia/wikipedia"
REVISION = "b04c8d1ceb2f5cd4588862100d08de323dccfbaa"
LICENSE = "CC BY-SA 3.0 and GFDL (Wikipedia; attribution by url)"
SHARDS = {
    "tr": (
        "20231101.tr/train-00000-of-00002.parquet",
        "6994014607c0920f049ff62c3ea03bfaa218848fa31fa331b6ad1e17ef8f393a",
    ),
    "en": (
        "20231101.en/train-00000-of-00041.parquet",
        "382e7f6f09e488b24793a7f7cfc659879d5a22da2cf2efec6491665f0c019677",
    ),
}
# int(blake3("P2-L/D-TR")[:8], 16) (the precedent of P2-0.3/D-B)
SEED_TEXT = "P2-L/D-TR"
SEED = int(blake3.blake3(SEED_TEXT.encode()).hexdigest()[:8], 16)
N_TR = 300
N_MIX = 100
MIN_CHARS = 2000
MIN_PARAS = 4
LEAD_CHARS = 1200
LEAD_MAX = 6
MIN_SENT_WORDS = 8
WINDOW = (4, 7)
K_CHUNKS = 50

SRC = DATA / "src" / "d_tr"
OUT = DATA / "d_tr"
MANIFEST = DATA / "MANIFEST.d_tr.json"
SENT = re.compile(r"(?<=[.!?])\s+")


def sha256(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def shard(lang: str) -> Path:
    """The pinned shard of `lang`, downloaded if missing, sha256 checked."""
    name, want = SHARDS[lang]
    p = SRC / name.replace("/", "__")
    if not p.exists():
        SRC.mkdir(parents=True, exist_ok=True)
        url = f"https://huggingface.co/datasets/{DATASET}/resolve/{REVISION}/{name}"
        tmp = p.with_suffix(".part")
        urllib.request.urlretrieve(url, tmp)
        tmp.rename(p)
    got = sha256(p)
    if got != want:
        raise RuntimeError(f"{p}: sha256 {got} != pinned {want}")
    return p


def paragraphs(text: str) -> list[str]:
    return [p.strip() for p in text.split("\n\n") if p.strip()]


def eligible(title: str, text: str) -> bool:
    return (
        "(" not in title
        and ":" not in title
        and len(text) >= MIN_CHARS
        and len(paragraphs(text)) >= MIN_PARAS
    )


def lead(text: str) -> str:
    out: list[str] = []
    for p in paragraphs(text)[:LEAD_MAX]:
        out.append(p)
        if sum(len(x) for x in out) >= LEAD_CHARS:
            break
    return "\n\n".join(out)


def load(lang: str) -> list[dict]:
    import pyarrow.parquet as pq

    rows = pq.read_table(shard(lang), columns=["id", "url", "title", "text"]).to_pylist()
    return sorted((r for r in rows if eligible(r["title"], r["text"])), key=lambda r: int(r["id"]))


def sentences(text: str) -> list[list[str]]:
    out = []
    for p in paragraphs(text):
        for s in SENT.split(p):
            w = s.split()
            if len(w) >= MIN_SENT_WORDS:
                out.append(w)
    return out


def window(rng: random.Random, text: str, where: str) -> str:
    ss = sentences(text)
    if not ss:
        raise RuntimeError(f"{where}: no sentence of {MIN_SENT_WORDS} words")
    w = ss[rng.randrange(len(ss))]
    n = rng.randint(*WINDOW)
    i = rng.randrange(len(w) - n + 1)
    return " ".join(w[i : i + n])


def build() -> tuple[dict[str, str], list[dict], dict]:
    tr = load("tr")
    en = {r["title"]: r for r in load("en")}
    rng = random.Random(SEED)
    pairs = [r for r in tr if r["title"] in en]
    mix = sorted(rng.sample(pairs, N_MIX), key=lambda r: int(r["id"]))
    taken = {r["id"] for r in mix}
    rest = [r for r in tr if r["id"] not in taken]
    only = sorted(rng.sample(rest, N_TR), key=lambda r: int(r["id"]))
    docs: dict[str, str] = {}
    sources: list[dict] = []
    for r in mix:
        e = en[r["title"]]
        tr_lead, en_lead = lead(r["text"]), lead(e["text"])
        path = f"mix/{r['id']}.md"
        docs[path] = f"# {r['title']}\n\n{tr_lead}\n\n## English\n\n{en_lead}\n"
        sources.append(
            {"doc": path, "tr": r["url"], "en": e["url"], "tr_lead": tr_lead, "en_lead": en_lead}
        )
    for r in only:
        path = f"tr/{r['id']}.md"
        tr_lead = lead(r["text"])
        docs[path] = f"# {r['title']}\n\n{tr_lead}\n"
        sources.append(
            {"doc": path, "tr": r["url"], "en": None, "tr_lead": tr_lead, "en_lead": None}
        )
    title = {s["doc"]: docs[s["doc"]].split("\n", 1)[0][2:] for s in sources}
    queries = []
    seen: set[str] = set()
    for s in sorted(sources, key=lambda s: s["doc"]):
        if s["en_lead"] is not None and rng.random() < 0.5:
            lang, text = "en", s["en_lead"]
        else:
            lang, text = "tr", s["tr_lead"]
        for qtype, q in (("sentence", window(rng, text, s["doc"])), ("title", title[s["doc"]])):
            q = q.strip()
            if not q or q in seen:
                raise RuntimeError(f"{s['doc']}#{qtype}: empty or duplicate query {q!r}")
            seen.add(q)
            queries.append(
                {
                    "id": f"{s['doc']}#{qtype}",
                    "type": qtype,
                    "lang": lang if qtype == "sentence" else "tr",
                    "query": q,
                    "truth": s["doc"],
                }
            )
    attribution = {s["doc"]: {"tr": s["tr"], "en": s["en"]} for s in sources}
    return docs, queries, attribution


def digest(docs: dict[str, str], queries: list[dict]) -> dict:
    h = blake3.blake3()
    for p in sorted(docs):
        h.update(p.encode() + b"\0" + docs[p].encode() + b"\0")
    q = "".join(json.dumps(x, ensure_ascii=False, sort_keys=True) + "\n" for x in queries)
    return {
        "docs": len(docs),
        "docs_blake3": h.hexdigest(),
        "queries": len(queries),
        "queries_blake3": blake3.blake3(q.encode()).hexdigest(),
    }


def manifest(docs: dict[str, str], queries: list[dict]) -> dict:
    gen = Path(__file__)
    return {
        "set": "D-TR",
        "round": "P2-L",
        "dataset": DATASET,
        "revision": REVISION,
        "license": LICENSE,
        "shards": {k: {"file": f, "sha256": s} for k, (f, s) in SHARDS.items()},
        "seed_text": SEED_TEXT,
        "seed": SEED,
        "rules": {
            "n_tr": N_TR,
            "n_mix": N_MIX,
            "min_chars": MIN_CHARS,
            "min_paragraphs": MIN_PARAS,
            "lead_chars": LEAD_CHARS,
            "lead_max_paragraphs": LEAD_MAX,
            "min_sentence_words": MIN_SENT_WORDS,
            "window_words": list(WINDOW),
            "k_chunks": K_CHUNKS,
        },
        "generator": {"file": "research/eval/datasets/d_tr.py", "sha256": sha256(gen)},
        **digest(docs, queries),
        "types": {
            t: sum(1 for q in queries if q["type"] == t)
            for t in sorted({q["type"] for q in queries})
        },
        "langs": {
            lang: sum(1 for q in queries if q["lang"] == lang)
            for lang in sorted({q["lang"] for q in queries})
        },
    }


def write(docs: dict[str, str], queries: list[dict], attribution: dict) -> None:
    if OUT.exists():
        shutil.rmtree(OUT)
    for p, t in docs.items():
        f = OUT / "docs" / p
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text(t, encoding="utf-8")
    (OUT / "queries.jsonl").write_text(
        "".join(json.dumps(q, ensure_ascii=False, sort_keys=True) + "\n" for q in queries),
        encoding="utf-8",
    )
    (OUT / "attribution.json").write_text(dump_json(attribution), encoding="utf-8")


def load_set() -> tuple[Path, list[dict]]:
    """The built set (docs directory, queries), checked against the manifest."""
    m = json.loads(MANIFEST.read_text(encoding="utf-8"))
    docs = {
        str(p.relative_to(OUT / "docs")): p.read_text(encoding="utf-8")
        for p in sorted((OUT / "docs").rglob("*.md"))
    }
    queries = [
        json.loads(x) for x in (OUT / "queries.jsonl").read_text(encoding="utf-8").splitlines()
    ]
    got = digest(docs, queries)
    for k, v in got.items():
        if m[k] != v:
            raise RuntimeError(
                f"data/d_tr: {k} {v} != manifest {m[k]} (python -m eval.datasets.d_tr)"
            )
    return OUT / "docs", queries


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    docs, queries, attribution = build()
    m = manifest(docs, queries)
    if a.check:
        want = json.loads(MANIFEST.read_text(encoding="utf-8"))
        if want != m:
            diff = sorted(k for k in set(want) | set(m) if want.get(k) != m.get(k))
            die(f"D-TR differs from {MANIFEST.name}: {diff}")
        write(docs, queries, attribution)
        print(f"d_tr: {m['docs']} documents, {m['queries']} queries — equal to {MANIFEST.name}")
        return
    write(docs, queries, attribution)
    MANIFEST.write_text(dump_json(m), encoding="utf-8")
    print(f"d_tr: {m['docs']} documents, {m['queries']} queries, seed {SEED} → {MANIFEST}")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
