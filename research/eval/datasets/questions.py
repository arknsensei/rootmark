"""D4 — question set for H2 (EVAL §D4). Runs as its own process; writes
`data/d4/{single,multi}.jsonl`, `data/d4/generation_log.jsonl` and
`data/splits/d4.json`, all committed to git. Retrieval never sees prompts or
chunk texts — only the written questions and their content-referenced truths.

    python -m eval.datasets.questions [--limit-single N --limit-multi N] [--check]

`--check` (no API) re-resolves every stored truth against a freshly built D3
and re-applies the leakage and file-name filters; any failure is an error.
"""

from __future__ import annotations

import argparse
import concurrent.futures as cf
import json
import os
import random
import re
from pathlib import Path

import rootmark_core

from eval import truth as T
from eval.datasets.common import DATA, ROOT, SEED, die, dump_json

MODEL = "claude-sonnet-4-6"
TEMPERATURE = 0
MAX_TOKENS = 200
N_SINGLE = 200
N_MULTI = 150
N_TUNE = {"single": 25, "multi": 25}
MIN_TOKENS = 30
LEAK_WORDS = 8
ATTEMPTS = 3
WORKERS = 8

PROMPT_SINGLE = (
    "Based only on the passage below, write one natural user question whose answer is contained "
    "entirely in the passage. Do not mention the document title or file name. Return only the question."
)
PROMPT_MULTI = (
    "Two passages are given. Write one question that requires BOTH passages to answer and cannot be "
    "answered from either alone. Do not mention titles or file names. Return only the question."
)
# appended on a retry, because temperature 0 would otherwise repeat the rejected question
RETRY_LEAK = (
    "Your previous question copied a long phrase from the passage. Write a different question in your "
    "own words."
)
RETRY_NAME = "Your previous question mentioned a file name. Write a different question without it."

D3 = DATA / "d3"
D4 = DATA / "d4"
SPLITS = DATA / "splits" / "d4.json"
WORD_RE = re.compile(r"[a-z0-9]+")


def words(text: str) -> list[str]:
    return WORD_RE.findall(text.lower())


def leaks(question: str, passage: str, n: int = LEAK_WORDS) -> bool:
    """True if the question contains any run of `n` consecutive passage words."""
    q, p = words(question), words(passage)
    grams = {tuple(p[i : i + n]) for i in range(len(p) - n + 1)}
    return any(tuple(q[i : i + n]) in grams for i in range(len(q) - n + 1))


def name_stem(doc_path: str) -> str:
    stem = Path(doc_path).name
    stem = stem.removesuffix(".md")
    return stem.replace("-", " ").replace("_", " ").lower()


def mentions_name(question: str, doc_paths: list[str]) -> bool:
    q = " ".join(question.lower().split())
    return any(name_stem(d) in q for d in doc_paths)


def chunk_refs(vault, chunk: dict) -> list[dict]:
    """Content references of the chunk's blocks: (doc_path, version_id, k)."""
    blocks = vault.snapshot_blocks(chunk["doc_path"])
    by_lineage = {}
    count: dict[str, int] = {}
    for b in blocks:
        by_lineage[b["lineage_id"]] = (b["version_id"], count.get(b["version_id"], 0))
        count[b["version_id"]] = count.get(b["version_id"], 0) + 1
    refs = []
    for lin in chunk["lineage_ids"]:
        vid, k = by_lineage[lin]
        refs.append({"doc_path": chunk["doc_path"], "version_id": vid, "k": k})
    return refs


def candidates(vault) -> tuple[list[dict], list[tuple[dict, dict]]]:
    """Single candidates (structural chunks ≥ 30 tokens) and multi candidates:
    one (A chunk with a wikilink, B chunk) pair per distinct document pair."""
    chunks = [c for d in vault.doc_paths() for c in vault.chunk(d, "structural")]
    for c in chunks:
        c["key"] = f"{c['doc_path']}::{c['chunk_id']}"
    big = [c for c in chunks if c["token_count"] >= MIN_TOKENS]
    by_doc: dict[str, list[dict]] = {}
    for c in big:
        by_doc.setdefault(c["doc_path"], []).append(c)
    stems: dict[str, list[str]] = {}
    for d in sorted(by_doc):
        n = Path(d).name
        stems.setdefault((n.removesuffix(".md")).lower(), []).append(d)
    pairs: dict[tuple[str, str], tuple[dict, dict]] = {}
    rng = random.Random(f"{SEED}:d4:pairs")
    for a in big:
        for w in a["wikilinks"]:
            for b_doc in stems.get(w["target"].lower(), []):
                if b_doc == a["doc_path"]:
                    continue
                key = tuple(sorted((a["doc_path"], b_doc)))
                if key in pairs:
                    continue
                cands = by_doc[b_doc]
                if w["heading"]:
                    under = [
                        c
                        for c in cands
                        if w["heading"].lower() in [h.lower() for h in c["heading_path"]]
                    ]
                    cands = under or cands
                pairs[key] = (a, rng.choice(cands))
    return big, [pairs[k] for k in sorted(pairs)]


class Generator:
    def __init__(self):
        import anthropic

        key = os.environ.get("ANTHROPIC_API_KEY") or _dotenv_key()
        if not key:
            raise RuntimeError("ANTHROPIC_API_KEY missing (T12 must be PARKed)")
        self.client = anthropic.Anthropic(api_key=key, max_retries=6)

    def ask(self, prompt: str) -> tuple[str, dict]:
        r = self.client.messages.create(
            model=MODEL,
            max_tokens=MAX_TOKENS,
            # anthropic 1.x dropped the typed `temperature` argument; the API still
            # honours it for claude-sonnet-4-6, so it travels in the request body
            extra_body={"temperature": TEMPERATURE},
            messages=[{"role": "user", "content": prompt}],
        )
        text = "".join(b.text for b in r.content if b.type == "text").strip()
        return text, {"model": r.model, "id": r.id, "stop_reason": r.stop_reason}


def _dotenv_key() -> str | None:
    env = ROOT / ".env"
    if not env.exists():
        return None
    for line in env.read_text().splitlines():
        if line.startswith("ANTHROPIC_API_KEY="):
            return line.split("=", 1)[1].strip().strip('"').strip("'")
    return None


def make_question(gen: Generator, kind: str, passages: list[dict]) -> tuple[str | None, list[dict]]:
    """Up to 3 attempts; returns (question | None, log entries)."""
    base = PROMPT_SINGLE if kind == "single" else PROMPT_MULTI
    if kind == "single":
        body = f"{base}\n\nPassage:\n{passages[0]['text']}"
    else:
        body = f"{base}\n\nPassage 1:\n{passages[0]['text']}\n\nPassage 2:\n{passages[1]['text']}"
    docs = [p["doc_path"] for p in passages]
    log, note = [], None
    for attempt in range(1, ATTEMPTS + 1):
        prompt = body if note is None else f"{body}\n\n{note}"
        q, meta = gen.ask(prompt)
        leak = any(leaks(q, p["text"]) for p in passages)
        name = mentions_name(q, docs)
        log.append({"attempt": attempt, "question": q, "leak": leak, "name": name, **meta})
        if not leak and not name and q:
            return q, log
        note = RETRY_LEAK if leak else RETRY_NAME
    return None, log


def generate(limit_single: int, limit_multi: int) -> None:
    vault = rootmark_core.Vault(str(D3))
    if not vault.doc_paths():
        raise RuntimeError("data/d3 is empty; run `make d3` first")
    singles, pairs = candidates(vault)
    rng = random.Random(f"{SEED}:d4")
    single_pick = rng.sample(singles, min(len(singles), limit_single))
    multi_pick = rng.sample(pairs, min(len(pairs), limit_multi))
    gen = Generator()
    jobs = [("single", i, [c]) for i, c in enumerate(single_pick)] + [
        ("multi", i, [a, b]) for i, (a, b) in enumerate(multi_pick)
    ]
    with cf.ThreadPoolExecutor(WORKERS) as ex:
        results = list(ex.map(lambda j: make_question(gen, j[0], j[2]), jobs))
    D4.mkdir(parents=True, exist_ok=True)
    out = {"single": [], "multi": []}
    logs = []
    for (kind, i, passages), (q, log) in zip(jobs, results, strict=True):
        qid = f"{kind[0]}{i:03d}"
        logs.append(
            {
                "id": qid,
                "kind": kind,
                "chunks": [p["key"] for p in passages],
                "accepted": q is not None,
                "attempts": log,
            }
        )
        if q is None:
            continue
        out[kind].append(
            {
                "id": qid,
                "type": kind,
                "question": q,
                "truth": [chunk_refs(vault, p) for p in passages],
            }
        )
    for kind in ("single", "multi"):
        (D4 / f"{kind}.jsonl").write_text(
            "".join(json.dumps(x, sort_keys=True) + "\n" for x in out[kind])
        )
    (D4 / "generation_log.jsonl").write_text(
        "".join(json.dumps(x, sort_keys=True) + "\n" for x in logs)
    )
    srng = random.Random(f"{SEED}:d4:split")
    splits = {"tune": [], "test": []}
    for kind in ("single", "multi"):
        ids = [x["id"] for x in out[kind]]
        tune = set(srng.sample(ids, min(N_TUNE[kind], len(ids))))
        splits["tune"] += [i for i in ids if i in tune]
        splits["test"] += [i for i in ids if i not in tune]
    SPLITS.parent.mkdir(parents=True, exist_ok=True)
    SPLITS.write_text(dump_json(splits))
    print({k: len(v) for k, v in out.items()}, {k: len(v) for k, v in splits.items()})


def load() -> dict[str, list[dict]]:
    return {
        k: [json.loads(line) for line in (D4 / f"{k}.jsonl").read_text().splitlines()]
        for k in ("single", "multi")
    }


def check() -> None:
    """No-API consistency check of the committed question set (used by tests and verify)."""
    vault = rootmark_core.Vault(str(D3))
    qs = load()
    splits = json.loads(SPLITS.read_text())
    ids = [q["id"] for k in qs for q in qs[k]]
    if len(ids) != len(set(ids)):
        raise RuntimeError("duplicate question ids")
    if set(splits["tune"]) & set(splits["test"]) or set(splits["tune"]) | set(
        splits["test"]
    ) != set(ids):
        raise RuntimeError("splits do not partition the question set")
    if len(qs["single"]) < 180 or len(qs["multi"]) < 130:
        raise RuntimeError(
            f"too few questions: {len(qs['single'])} single, {len(qs['multi'])} multi"
        )
    for k in qs:
        for q in qs[k]:
            texts = []
            for group in q["truth"]:
                T.resolve_all(vault, group)
                texts.append(
                    " ".join(vault.block(r["version_id"])["normalized_text"] for r in group)
                )
            if any(leaks(q["question"], t) for t in texts):
                raise RuntimeError(f"{q['id']}: leaks an {LEAK_WORDS}-word run")
            if mentions_name(q["question"], [g[0]["doc_path"] for g in q["truth"]]):
                raise RuntimeError(f"{q['id']}: mentions a file name")
    print(f"D4 check OK: {len(qs['single'])} single, {len(qs['multi'])} multi, all truths resolve")


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--limit-single", type=int, default=N_SINGLE)
    ap.add_argument("--limit-multi", type=int, default=N_MULTI)
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    if a.check:
        check()
    else:
        generate(a.limit_single, a.limit_multi)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(f"{type(e).__name__}: {e}")
