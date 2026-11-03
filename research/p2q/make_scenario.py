"""P2-Q/Q0: writes the pre-registered scenario `research/p2q/scenario.json`
(GOAL P2-Q Kapsam 1, TASKS Q0) from the read-only snapshots of the corpora.

    python -m p2q.make_scenario --qorbit <snapshot> --obsidian <snapshot> [--out <file>]
    python -m p2q.make_scenario ... --check     (the committed file is what
                                                 these snapshots give, byte for byte)

Everything is a function of the seed and of the snapshots' sorted path lists
(personal paths enter the file only as blake2b). Agent writes, Obsidian edits
and queries are synthetic text built from fixed templates (no LLM, no API).
"""

from __future__ import annotations

import argparse
import json
import posixpath
import random
import re
import sys
from pathlib import Path
from urllib.parse import unquote

from p2q.common import b2, corpus_summary, list_md, seeded_sample

SEED = 20261103
REPO = Path(__file__).resolve().parents[2]
DNOTES = REPO / "research/data/d_notes/notes"
DOCKER_IMAGE = (
    "debian:trixie-slim@sha256:f6b08b09a05725f9431502d3b5714f5c4576e82c1a84fa1debe0c24a7e0c3ec4"
)
DOCKER_IMAGE_AMD64 = "sha256:1e1a22175fbf1b32157ddaf0219e1db4e5401fab2b94ed285add64d18587cd2d"

TOPICS = {
    "tr": [
        "faiz kararı",
        "enflasyon verisi",
        "kur hareketi",
        "tahvil getirisi",
        "bilanço dönemi",
        "sanayi üretimi",
        "cari açık",
        "kredi büyümesi",
    ],
    "en": [
        "rate decision",
        "inflation print",
        "currency move",
        "bond yield",
        "earnings season",
        "industrial output",
        "current account",
        "credit growth",
    ],
}
SOURCES = {
    "tr": ["merkez bankası bülteni", "istatistik kurumu", "borsa duyurusu", "bakanlık açıklaması"],
    "en": ["central bank bulletin", "statistics office", "exchange notice", "ministry statement"],
}

TEMPLATES = {
    ("toplayici-notu", "tr"): (
        "# Toplayıcı notu {code}\n\nKaynak: {source}. Konu: {topic}.\n\n"
        "{code} kodlu derlemede {topic} için {n} veri noktası toplandı; değerler "
        "{source} kaynağından olduğu gibi aktarıldı.\n"
    ),
    ("toplayici-notu", "en"): (
        "# Collector note {code}\n\nSource: {source}. Topic: {topic}.\n\n"
        "Collection {code} gathered {n} data points on the {topic}; the values were "
        "copied as published by the {source}.\n"
    ),
    ("resmi-ozet", "tr"): (
        "# Resmi kaynak özeti {code}\n\n{source} {topic} hakkında bir metin yayımladı.\n\n"
        "Özet {code}: metin {n} madde içeriyor; ilk madde {topic} ile ilgili beklentiyi "
        "değiştirmiyor.\n"
    ),
    ("resmi-ozet", "en"): (
        "# Official source summary {code}\n\nThe {source} published a text on the {topic}.\n\n"
        "Summary {code}: the text has {n} items; the first one leaves the outlook on the "
        "{topic} unchanged.\n"
    ),
    ("llm-yorumu", "tr"): (
        "# Yorum {code}\n\nBu yorum bir dil modelinin {topic} üzerine değerlendirmesidir "
        "ve onaylanmamıştır.\n\nYorum {code}: {source} verisine göre {topic} önümüzdeki "
        "{n} haftada dalgalı kalabilir.\n"
    ),
    ("llm-yorumu", "en"): (
        "# Commentary {code}\n\nThis is a language model's reading of the {topic} and is "
        "not approved.\n\nCommentary {code}: going by the {source}, the {topic} may stay "
        "volatile for {n} weeks.\n"
    ),
    ("brif", "tr"): (
        "# Brif {code}\n\n- Konu: {topic}\n- Dayanak: {source}\n- Not {code}: {n} başlıkta "
        "özetlendi.\n"
    ),
    ("brif", "en"): (
        "# Brief {code}\n\n- Topic: {topic}\n- Basis: {source}\n- Note {code}: summarised "
        "under {n} headings.\n"
    ),
    ("dogrudan", "tr"): (
        "# Doğrudan not {code}\n\nAjan bu dosyayı sunucuyu atlayarak doğrudan yazdı: "
        "{topic}, {source}.\n\nKayıt {code}: {n} satır.\n"
    ),
    ("dogrudan", "en"): (
        "# Direct note {code}\n\nThe agent wrote this file directly, bypassing the server: "
        "{topic}, {source}.\n\nRecord {code}: {n} lines.\n"
    ),
    ("dogrudan-ek", "tr"): (
        "\n## Ajan notu {code}\n\nAjan bu bölümü dosyaya doğrudan ekledi: {topic} ({source}), "
        "{n} gözlem.\n"
    ),
    ("dogrudan-ek", "en"): (
        "\n## Agent note {code}\n\nThe agent appended this section to the file directly: "
        "{topic} ({source}), {n} observations.\n"
    ),
    ("obs-yeni", "tr"): (
        "# Pilot notu {code}\n\nBu not pilotta Obsidian'da yazıldı: {topic}.\n\n"
        "Pilot {code}: {n} satırlık bir deneme.\n"
    ),
    ("obs-ek", "tr"): "\n\nPilot ekleme {code}: Obsidian'da {topic} üzerine {n} sözcük eklendi.\n",
    ("obs-degis", "tr"): (
        "# Pilot notu {code}\n\nBu not pilotta Obsidian'da yazıldı ve sonra değiştirildi: "
        "{topic}.\n\nPilot {code}: {n} satırlık ikinci sürüm.\n"
    ),
    ("obs-oneri", "tr"): (
        "# Ajan önerisi {code}\n\nAjan {topic} için bir öneri yazdı ({source}).\n\n"
        "Öneri {code}: {n} madde.\n"
    ),
}

RECIPE_HEAD = """\
[promotion]
policy = "human_only"
version = "p2q-1"

[[promotion.rules]]
id = "toplayici"
actor = "agent:toplayici"
path_prefix = "toplayici/"
to = "canonical"

[edit]
policy = "append_only"

[version_trigger]
idle_ms = 30000
min_sim_change = 0.2

[journal]
retention_hours = 168
"""
RECIPE_IDENTITY = """
[identity]
human_id = "pilot"

[attestation]
unattested = "unverified"
"""
RECIPES = {
    # Turkish/English vault (GUIDE "Turkish vaults"): qorbit, the Obsidian copy
    "tr": RECIPE_HEAD
    + """
[server]
token_rotate_days = 30
max_propose_bytes = 1048576

[index]
compact_every_s = 300
tokenizer = "uni_fold_tr"
embedder = "multilingual-minilm-l12-v2"
"""
    + RECIPE_IDENTITY,
    # English vault, default index: D-NOTES (Mac, Docker, CI)
    "default": RECIPE_HEAD
    + """
[server]
token_rotate_days = 30
max_propose_bytes = 1048576

[index]
compact_every_s = 300
"""
    + RECIPE_IDENTITY,
}
OBSIDIAN_PORT = 27357
RECIPES["obsidian"] = RECIPES["tr"].replace(
    "max_propose_bytes = 1048576\n", f"max_propose_bytes = 1048576\nhttp_port = {OBSIDIAN_PORT}\n"
)


def fill(kind: str, lang: str, code: str, rng: random.Random) -> str:
    return TEMPLATES[(kind, lang)].format(
        code=code,
        topic=rng.choice(TOPICS[lang]),
        source=rng.choice(SOURCES[lang]),
        n=rng.randint(3, 40),
    )


def headless(seed: int, corpus_paths: list[str], salt: str) -> dict:
    """The headless agent scene (GOAL Q2 (b), (d)); `corpus_paths` only picks
    the six documents the agent appends to directly."""
    rng = random.Random(f"{seed}:writes")
    propose = []
    for i in range(100):
        rule = i < 50
        kind = (
            ("toplayici-notu" if i % 2 == 0 else "resmi-ozet")
            if rule
            else ("llm-yorumu" if i % 2 == 0 else "brif")
        )
        lang = "tr" if (i // 2) % 2 == 0 else "en"
        code = f"pqk{i:03d}" if rule else f"pqy{i:03d}"
        folder = "toplayici" if rule else "yorum"
        propose.append(
            {
                "i": i,
                "doc_path": f"{folder}/{kind}/{code}.md",
                "kind": kind,
                "lang": lang,
                "code": code,
                "rule": rule,
                "content": fill(kind, lang, code, rng),
                "reason": f"p2q {kind} {code}",
            }
        )
    appends = seeded_sample(corpus_paths, 6, seed, f"{salt}:append")
    direct = []
    for j in range(20):
        lang = "tr" if j % 2 == 0 else "en"
        code = f"pqd{j:03d}"
        if j < 14:
            direct.append(
                {
                    "j": j,
                    "op": "create",
                    "doc_path": f"ajan-dogrudan/{code}.md",
                    "lang": lang,
                    "code": code,
                    "content": fill("dogrudan", lang, code, rng),
                }
            )
        else:
            direct.append(
                {
                    "j": j,
                    "op": "append",
                    "doc_path_b2": b2(appends[j - 14]),
                    "lang": lang,
                    "code": code,
                    "content": fill("dogrudan-ek", lang, code, rng),
                }
            )
    yorum = [w for w in propose if not w["rule"]]
    rules = [w for w in propose if w["rule"]]
    promote = sorted(w["doc_path"] for w in random.Random(f"{seed}:promote").sample(yorum, 12))
    demote = sorted(w["doc_path"] for w in random.Random(f"{seed}:demote").sample(rules, 3))
    return {
        "propose": propose,
        "direct": direct,
        "promote_docs": promote,
        "demote_docs": demote,
        "_appends": appends,
    }


def layer_at(doc: dict, phase: str, promote: list[str], demote: list[str]) -> str:
    if "rule" in doc:
        if doc["rule"]:
            return "proposal" if phase == "son" and doc["doc_path"] in demote else "canonical"
        return "canonical" if phase == "son" and doc["doc_path"] in promote else "proposal"
    return "unverified"


def headless_queries(h: dict, seed: int) -> list[dict]:
    rng = random.Random(f"{seed}:queries")
    rules = [w for w in h["propose"] if w["rule"]]
    yorum = [w for w in h["propose"] if not w["rule"]]
    demoted = [w for w in rules if w["doc_path"] in h["demote_docs"]]
    promoted = [w for w in yorum if w["doc_path"] in h["promote_docs"]]
    pick_r = demoted + rng.sample([w for w in rules if w not in demoted], 13)
    pick_y = promoted[:8] + rng.sample([w for w in yorum if w not in promoted], 8)
    pick_d = h["direct"][:4] + h["direct"][14:16]
    qs = []

    def target(doc):
        return doc.get("doc_path") or ("b2:" + doc["doc_path_b2"])

    def exp(doc):
        out = {}
        for ph in ("ajan-yazdi", "son"):
            layer = layer_at(doc, ph, h["promote_docs"], h["demote_docs"])
            out[ph] = {
                "layer": layer,
                "default": [target(doc)] if layer == "canonical" else [],
                "include_proposal": [target(doc)],
            }
        return out

    n = 0
    for doc in pick_r + pick_y + pick_d:
        n += 1
        code = doc["code"]
        text = code.upper() if n % 3 == 0 else code  # case folding on the way
        qs.append(
            {
                "id": f"q{n:02d}",
                "mode": "lexical",
                "match": "exact",
                "query": text,
                "lang": doc["lang"],
                "targets": [target(doc)],
                "expect": exp(doc),
            }
        )
    mixed = [
        ("tr", "{code} özet faiz kararı"),
        ("en", "{code} summary of the rate decision"),
        ("mix", "{code} resmi kaynak official summary"),
        ("tr", "{code} yorum enflasyon"),
        ("en", "{code} commentary on the bond yield"),
        ("mix", "{code} brif brief kur"),
    ]
    for (lang, pat), doc in zip(
        mixed, [pick_r[3], pick_r[5], pick_y[0], pick_y[9], pick_y[11], pick_r[7]]
    ):
        n += 1
        qs.append(
            {
                "id": f"q{n:02d}",
                "mode": "hybrid",
                "match": "contains",
                "query": pat.format(code=doc["code"]),
                "lang": lang,
                "targets": [target(doc)],
                "expect": exp(doc),
            }
        )
    for word in ("zqxv kelebekyok", "wqyk notfoundword", "pqz999", "qqjx ışıkyok"):
        n += 1
        qs.append(
            {
                "id": f"q{n:02d}",
                "mode": "lexical",
                "match": "exact",
                "query": word,
                "lang": "mix",
                "targets": [],
                "expect": {
                    ph: {"layer": None, "default": [], "include_proposal": []}
                    for ph in ("ajan-yazdi", "son")
                },
            }
        )
    return qs


HEADLESS_STEPS = [
    ("H01", "copy", "fresh copy of the snapshot; corpus_summary equals the registered one"),
    (
        "H02",
        "ingest_dir",
        "`rootmark ingest-dir <vault> <vault>` exit 0; ingested files = corpus files",
    ),
    (
        "H03",
        "recipe",
        "the corpus's recipe written to .rootmark/policy.toml; `rootmark policy lint` exit 0, 1 rule",
    ),
    (
        "H04",
        "index_build",
        "`rootmark index build` exit 0; `rootmark index status` names no unindexed document",
    ),
    (
        "H05",
        "serve",
        "`rootmark serve --mcp stdio --http 127.0.0.1:0 --watch`; HTTP status: attestation.unattested unverified, human_id pilot, index ok",
    ),
    ("H06", "checkpoint", "HTTP checkpoint `baslangic`"),
    (
        "H07",
        "propose",
        "100 MCP propose (client toplayici/1.0): every call ok; blame: rule docs every block canonical, yorum docs every block proposal",
    ),
    (
        "H08",
        "searchable",
        "after each rule propose a lexical search of its code finds it canonical (latency recorded)",
    ),
    (
        "H09",
        "direct",
        "20 direct file writes (14 new, 6 appends); after the watcher: every new block unverified, canonical from these writes 0",
    ),
    ("H10", "checkpoint", "HTTP checkpoint `ajan-yazdi`; the phase's query results recorded"),
    (
        "H11",
        "queries",
        "phase ajan-yazdi: every query's expected sets; default hits never proposal/unverified",
    ),
    ("H12", "promote", "HTTP promote of every lineage of the 12 promote_docs: all promoted"),
    ("H13", "demote", "HTTP demote of every lineage of the 3 demote_docs: all demoted"),
    ("H14", "checkpoint", "HTTP checkpoint `insan-onayladi`"),
    (
        "H15",
        "queries",
        "phase son: every query's expected sets; default hits never proposal/unverified",
    ),
    ("H16", "blame", "blame of every synthetic document: layers as expected at phase son"),
    (
        "H17",
        "trace",
        "5 brief searches over MCP; `trace` lists each, returned blocks = the search's hits",
    ),
    (
        "H18",
        "checkpoint_return",
        "search and blame `at: ajan-yazdi` now = what was recorded at H10/H11",
    ),
    (
        "H19",
        "status",
        "status over HTTP and over MCP: same index, policy, attestation, last_checkpoint",
    ),
    (
        "H20",
        "prune_dry_run",
        "HTTP prune {dry_run: true}: ok; events_hash and every .rootmark file unchanged",
    ),
    (
        "H21",
        "sigterm",
        "SIGTERM: serve exits 0 within 60 s; reopened serve: status ok, events_hash and a query result unchanged",
    ),
    (
        "H22",
        "final",
        "serve stopped, `rootmark index compact`; events_hash, index files, query results recorded (blake2b)",
    ),
    (
        "H23",
        "observe",
        "observe.py read-only (before/after blake2b equal); its counts = the driver's",
    ),
]
TRACE_QUERIES = [
    {"id": "t1", "query": "brif faiz kararı", "lang": "tr"},
    {"id": "t2", "query": "brief on the inflation print", "lang": "en"},
    {"id": "t3", "query": "resmi kaynak özeti tahvil getirisi", "lang": "tr"},
    {"id": "t4", "query": "official source summary of the current account", "lang": "en"},
    {"id": "t5", "query": "toplayıcı notu kredi büyümesi credit growth", "lang": "mix"},
]


def obsidian(seed: int, notes: list[str], linked: list[str]) -> dict:
    """The Obsidian scene (GOAL Q2 (c)): edits through the real Obsidian."""
    rng = random.Random(f"{seed}:obsidian")
    mod = seeded_sample(notes, 8, seed, "obs:modify")
    rest = [p for p in notes if p not in mod]
    dele = seeded_sample(rest, 2, seed, "obs:delete")
    ren_pool = [p for p in linked if p not in mod and p not in dele]
    ren = seeded_sample(ren_pool, 2, seed, "obs:rename")
    edits = []
    k = 0

    def code():
        nonlocal k
        k += 1
        return f"pqo{k:03d}"

    for n in range(8):
        c = code()
        edits.append(
            {
                "e": len(edits) + 1,
                "op": "create",
                "doc_path": f"pilot/yeni-{n + 1}.md",
                "code": c,
                "content": fill("obs-yeni", "tr", c, rng),
            }
        )
    for p in mod:
        c = code()
        edits.append(
            {
                "e": len(edits) + 1,
                "op": "append",
                "doc_path_b2": b2(p),
                "code": c,
                "content": fill("obs-ek", "tr", c, rng),
            }
        )
    for n in (1, 2):
        c = code()
        edits.append(
            {
                "e": len(edits) + 1,
                "op": "replace",
                "doc_path": f"pilot/yeni-{n}.md",
                "code": c,
                "content": fill("obs-degis", "tr", c, rng),
            }
        )
    for n in (7, 8):
        edits.append({"e": len(edits) + 1, "op": "delete", "doc_path": f"pilot/yeni-{n}.md"})
    for p in dele:
        edits.append({"e": len(edits) + 1, "op": "delete", "doc_path_b2": b2(p)})
    renames = [
        {"r": i + 1, "doc_path_b2": b2(p), "new_name": f"pilot-adlandirma-{i + 1}"}
        for i, p in enumerate(ren)
    ]
    proposals = []
    for n in range(4):
        c = code()
        proposals.append(
            {
                "doc_path": f"ajan/oneri-{n + 1}.md",
                "code": c,
                "content": fill("obs-oneri", "tr", c, rng),
                "reason": f"p2q obsidian {c}",
            }
        )
    layer_ops = [{"op": "promote", "doc_path": p["doc_path"]} for p in proposals] + [
        {"op": "demote", "doc_path": proposals[0]["doc_path"]},
        {"op": "demote", "doc_path": proposals[1]["doc_path"]},
    ]
    queries = []
    for e in edits:
        if "code" in e and e["op"] in ("create", "append", "replace"):
            gone = e["op"] == "create" and e["doc_path"] in ("pilot/yeni-7.md", "pilot/yeni-8.md")
            replaced = e["op"] == "create" and e["doc_path"] in (
                "pilot/yeni-1.md",
                "pilot/yeni-2.md",
            )
            target = e.get("doc_path") or "b2:" + e["doc_path_b2"]
            queries.append(
                {
                    "id": f"o{len(queries) + 1:02d}",
                    "mode": "lexical",
                    "query": e["code"],
                    "expect_default": [] if (gone or replaced) else [target],
                }
            )
    for p, final in zip(proposals, ["proposal", "proposal", "canonical", "canonical"]):
        queries.append(
            {
                "id": f"o{len(queries) + 1:02d}",
                "mode": "lexical",
                "query": p["code"],
                "expect_default": [p["doc_path"]] if final == "canonical" else [],
            }
        )
    # hybrid, mixed language: the target is among the hits, every default hit
    # canonical
    lexical = [q for q in queries if q["expect_default"]]
    pats = ["{code} pilot notu", "{code} pilot note in Obsidian", "{code} deneme ekleme added"]
    for i, q in enumerate(lexical):
        queries.append(
            {
                "id": f"o{len(queries) + 1:02d}",
                "mode": "hybrid",
                "match": "contains",
                "query": pats[i % 3].format(code=q["query"]),
                "expect_default": q["expect_default"],
            }
        )
    for word in ("zqxv kelebekyok", "wqyk notfoundword", "pqz999", "qqjx ışıkyok"):
        queries.append(
            {
                "id": f"o{len(queries) + 1:02d}",
                "mode": "lexical",
                "query": word,
                "expect_default": [],
            }
        )
    return (
        {
            "edits": edits,
            "renames": renames,
            "agent_client": "yardimci/1.0",
            "proposals": proposals,
            "layer_ops": layer_ops,
            "queries": queries,
            "expect": {
                "attested_edit_layer": "canonical",
                "md_written_by_rootmark": 0,
                "panics": 0,
                "proposal_final_layers": {
                    p["doc_path"]: f
                    for p, f in zip(proposals, ["proposal", "proposal", "canonical", "canonical"])
                },
            },
            "steps": [
                (
                    "O01",
                    "copy",
                    "fresh copy of the Obsidian snapshot; summary equals the registered one",
                ),
                ("O02", "ingest_dir", "`rootmark ingest-dir` exit 0 (time)"),
                (
                    "O03",
                    "recipe",
                    "recipe `obsidian` (http_port 27357, unverified, identity pilot); policy lint exit 0",
                ),
                (
                    "O04",
                    "serve",
                    "`rootmark serve --mcp stdio --http 127.0.0.1 --watch --attest-log <outside the vault>`",
                ),
                (
                    "O05",
                    "plugin",
                    "Obsidian (own --user-data-dir, CDP); plugin installed from the 0.3.0 zip by GUIDE's manual steps; status bar `rootmark 0.3.0 · human:pilot`",
                ),
                (
                    "O06",
                    "edits",
                    "the 22 edits through Obsidian's editor API; each recorded with its blocks' layers after the watcher (expected canonical)",
                ),
                (
                    "O07",
                    "renames",
                    "2 renames with link update (Obsidian's fileManager); layers of the renamed note and of the notes whose links changed",
                ),
                ("O08", "agent_proposals", "4 MCP proposals by yardimci/1.0 (proposal)"),
                (
                    "O09",
                    "layer_ops",
                    "4 promotes and 2 demotes from Obsidian (the plugin's commands); final layers as registered",
                ),
                (
                    "O10",
                    "queries",
                    "the registered lexical queries over HTTP: default hits = expect_default, never proposal/unverified",
                ),
                ("O11", "blame", "blame of every touched document"),
                (
                    "O12",
                    "md_bytes",
                    "every md byte equals what Obsidian (or the copy, or the agent's propose) wrote: rootmark's md writes 0",
                ),
                (
                    "O13",
                    "stop",
                    "SIGTERM: serve exits 0; no panic in its stderr; attest log summarised (refusals with reasons)",
                ),
                ("O14", "observe", "observe.py read-only; its counts = the driver's"),
            ],
        },
        mod,
        dele,
        ren,
    )


def wikilink_targets(root: Path, paths: list[str]) -> list[str]:
    """Notes some other note links to with [[name]] (read locally, never
    recorded): candidates of a rename with link update."""
    by_name = {}
    for p in paths:
        by_name.setdefault(Path(p).stem, []).append(p)
    known = set(paths)
    linked = set()
    rx = re.compile(r"\[\[([^\]|#^]+)")
    md = re.compile(r"\]\(([^)\s]+\.md)(?:#[^)]*)?\)")
    for p in paths:
        text = (root / p).read_text(encoding="utf-8", errors="replace")
        for m in rx.finditer(text):
            name = m.group(1).strip()
            name = name.removesuffix(".md")
            cands = by_name.get(Path(name).name, [])
            if len(cands) == 1 and cands[0] != p:
                linked.add(cands[0])
        for m in md.finditer(text):
            t = unquote(m.group(1))
            for cand in (
                posixpath.normpath(posixpath.join(posixpath.dirname(p), t)),
                posixpath.normpath(t),
            ):
                if cand in known and cand != p:
                    linked.add(cand)
                    break
    return sorted(linked, key=lambda s: s.encode())


def build(qorbit: Path, obs: Path) -> dict:
    q_paths = list_md(qorbit)
    d_paths = list_md(DNOTES)
    o_paths = list_md(obs)
    hq = headless(SEED, q_paths, "qorbit")
    hd = headless(SEED, d_paths, "dnotes")
    linked = wikilink_targets(obs, o_paths)
    ob, *_ = obsidian(SEED, o_paths, linked)
    scen = {
        "version": "p2q-scenario/1",
        "seed": SEED,
        "registered": "P2-Q/Q0 (before Q2's first measurement; GOAL Q-1)",
        "corpora": {
            "qorbit": {
                "kind": "personal",
                "recipe": "tr",
                "scene": "headless",
                "source": "~/Works/qorbit: every *.md outside node_modules/ and .git/",
                "snapshot": corpus_summary(qorbit),
            },
            "obsidian": {
                "kind": "personal",
                "recipe": "obsidian",
                "scene": "obsidian",
                "source": "~/Works/Obs: the whole vault except .trash/ (notes: *.md outside .obsidian/)",
                "snapshot": corpus_summary(obs),
                "linked_notes": len(linked),
            },
            "dnotes": {
                "kind": "open",
                "recipe": "default",
                "scene": "headless",
                "source": "research/data/d_notes/notes (NOTICE.md)",
                "snapshot": corpus_summary(DNOTES),
            },
        },
        "recipes": RECIPES,
        "agent": {
            "client": {"name": "toplayici", "version": "1.0"},
            "rule_prefix": "toplayici/",
            "human_id": "pilot",
        },
        "headless": {
            "steps": [{"id": i, "action": a, "expect": e} for i, a, e in HEADLESS_STEPS],
            "checkpoints": ["baslangic", "ajan-yazdi", "insan-onayladi"],
            "trace_queries": TRACE_QUERIES,
            "per_corpus": {},
        },
        "obsidian": {k: v for k, v in ob.items() if k != "steps"},
        "measures": {
            "install": [
                "install.sh wall time (s)",
                "binary blake2b = archive's",
                "conformance passed/total",
            ],
            "ingest": ["ingest-dir wall time (s)", "events", "documents"],
            "index": ["index build wall time (s)", "chunks", "chunks/s", "index bytes"],
            "search": ["latency p50/p95 over the registered queries (ms, HTTP, client side)"],
            "propose": [
                "MCP propose latency p50/p95 (ms)",
                "propose → searchable latency p50/p95 (ms)",
            ],
            "watcher": [
                "direct write → ingested (blame shows it) p50/p95 (ms)",
                "Obsidian save → layer recorded p50/p95 (ms)",
            ],
            "memory": ["peak RSS of rootmark serve (KiB; ps sampling every 0.2 s)"],
            "disk": [".rootmark/ bytes at the end"],
            "determinism": [
                "events_hash",
                "index files blake2b (vectors, manifest)",
                "query results blake2b",
                "rankings (doc ids and order)",
                "lexical scores: named last-ULP differences only",
            ],
        },
        "docker": {
            "platform": "linux/amd64",
            "image": DOCKER_IMAGE,
            "image_amd64_manifest": DOCKER_IMAGE_AMD64,
            "note": "emulated: times are information, not speed numbers",
        },
        "ci": {
            "workflow": ".github/workflows/pilot.yml",
            "runner": "ubuntu-latest",
            "corpus": "dnotes only (open)",
        },
        "obsidian_steps": [{"id": i, "action": a, "expect": e} for i, a, e in ob["steps"]],
    }
    for name, h in (("qorbit", hq), ("dnotes", hd)):
        h = dict(h)
        if name == "dnotes":
            h["append_paths"] = h["_appends"]  # open corpus: paths in the clear
        del h["_appends"]
        h["queries"] = headless_queries(h, SEED)
        scen["headless"]["per_corpus"][name] = h
    return scen


def render(scen: dict) -> str:
    return json.dumps(scen, ensure_ascii=False, indent=1, sort_keys=True) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--qorbit", required=True, type=Path)
    ap.add_argument("--obsidian", required=True, type=Path)
    ap.add_argument("--out", type=Path, default=Path(__file__).with_name("scenario.json"))
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    text = render(build(a.qorbit, a.obsidian))
    if a.check:
        same = a.out.read_text(encoding="utf-8") == text
        print("make_scenario: " + ("same" if same else "DIFFERENT"))
        return 0 if same else 1
    a.out.write_text(text, encoding="utf-8")
    print(f"make_scenario: wrote {a.out.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
