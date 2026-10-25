"""D-NOTES — the alignment set of round P2-B (GOAL P2-B 3, TASKS B2): real
notes of public, openly licensed repositories, and seeded edge cases. No
LLM. Everything below is fixed before any measurement (pre-registration,
ASSUMPTIONS P2-B B2-*): the sources, their revisions and licences, the
selection rules, the seed, the edge-case generator and this script.

**Sources** (`SOURCES`; each cloned into `data/src/d_notes/<name>`,
git-ignored, and checked out at its pinned revision; the notes are read
as git blobs, byte for byte — no line-ending translation):
- `rfcs` — rust-lang/rfcs, `text/*.md`; MIT (dual MIT/Apache-2.0; MIT
  chosen): design notes, footnotes, tables, code;
- `terminal` — microsoft/terminal, every `*.md`; MIT: specs and notes,
  many committed with CRLF line endings;
- `tasks` — obsidian-tasks-group/obsidian-tasks, `docs/**/*.md`; MIT:
  Obsidian notes with callouts, embeds, front matter, HTML;
- `kepano` — kepano/kepano-obsidian, every `*.md`; MIT: an Obsidian vault
  (embeds, front matter);
- `dataview` — blacksmithgu/obsidian-dataview, `docs/**/*.md`; MIT;
- `react_tr` — reactjs/tr.react.dev, `src/content/**/*.md`; CC BY 4.0:
  the Turkish translation of the React documentation (Turkish notes; MDX
  components count as HTML).

**Eligible note:** a `.md` blob of a source's include pattern, valid UTF-8,
`MIN_BYTES`–`MAX_BYTES` bytes. Its **classes** (`CLASSES`, regular
expressions on the bytes as text): `crlf` (a `\\r\\n`), `footnote` (a
footnote definition line `[^x]:`), `callout` (`> [!type`), `embed`
(`![[…]]`), `html` (a line opening an HTML or MDX tag), `table` (a pipe row
followed by a delimiter row), `nested_list` (a list item followed by an
indented list item), `code_fence` (a line opening ``` or ~~~), `frontmatter`
(the file starts with `---` and a line ending). A note is **Turkish** when
it holds at least `TR_MIN` of the letters `ğĞışŞİ` (a `react_tr` page that
is translated).

**Selection** (`random.Random(SEED)`; the pool sorted by `(source, path)`):
for each class in `CLASSES` order, the eligible notes of that class not yet
selected are shuffled and taken until the selected notes hold the class
`CLASS_QUOTA` times (fewer than `CLASS_MIN` available is an error — the
condition is not relaxed); then Turkish notes the same way until
`TR_QUOTA`; then the rest of the pool shuffled until `N_NOTES` notes. Each
selected note is written to `data/d_notes/notes/<source>/<path>` (these
files are committed with the attribution `data/d_notes/NOTICE.md`).

**Edge cases** (`edge()`; `data/d_notes_edge/<class>/<nnn>.md`,
git-ignored, regenerated; their digest in the manifest): `EDGE_PER_CLASS`
documents per edge class (`EDGE_CLASSES`), each from
`random.Random(f"{SEED}:edge:{class}:{n}")`: 3–9 parts drawn from a fixed
list of part makers (headings ATX/setext, paragraphs with lazy
continuation, lists with nesting, code fences, tables, block quotes,
thematic breaks, link reference definitions, …), at least two of them of
the class's own kind; parts joined by one or two blank lines (or none,
where CommonMark allows); `crlf` documents use `\\r\\n` throughout, `bom`
documents start with U+FEFF.

    python -m eval.datasets.d_notes            # clone (if missing), select, write notes, edge cases, manifest
    python -m eval.datasets.d_notes --check    # notes and edge cases against the committed manifest (no network)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import re
import subprocess
from pathlib import Path

from eval.datasets.common import DATA, die, dump_json, git

SEED = 20261025
N_NOTES = 400
CLASS_QUOTA = 25
CLASS_MIN = 20
TR_QUOTA = 40
TR_MIN = 5
MIN_BYTES = 40
MAX_BYTES = 200_000
EDGE_PER_CLASS = 200

SOURCES = [
    {
        "name": "rfcs",
        "url": "https://github.com/rust-lang/rfcs",
        "rev": "f148c94986feb17b3954ee620c848c96a9b41166",
        "license": "MIT (dual MIT/Apache-2.0; MIT chosen)",
        "license_file": "LICENSE-MIT",
        "include": r"text/[^/]+\.md",
    },
    {
        "name": "terminal",
        "url": "https://github.com/microsoft/terminal",
        "rev": "98c8c795ebc2560af55e72beba3fa5fd31ceef64",
        "license": "MIT",
        "license_file": "LICENSE",
        "include": r".+\.md",
    },
    {
        "name": "tasks",
        "url": "https://github.com/obsidian-tasks-group/obsidian-tasks",
        "rev": "5d9902f734d4ac03bd765bbb4df977772f5c2e14",
        "license": "MIT",
        "license_file": "LICENSE",
        "include": r"docs/.+\.md",
    },
    {
        "name": "kepano",
        "url": "https://github.com/kepano/kepano-obsidian",
        "rev": "473697347a0eaac790a6596229c741b3915f3b77",
        "license": "MIT",
        "license_file": "LICENSE",
        "include": r".+\.md",
    },
    {
        "name": "dataview",
        "url": "https://github.com/blacksmithgu/obsidian-dataview",
        "rev": "5ad0994ff384cbb797de382e7edff2388141b73a",
        "license": "MIT",
        "license_file": "LICENSE.txt",
        "include": r"docs/.+\.md",
    },
    {
        "name": "react_tr",
        "url": "https://github.com/reactjs/tr.react.dev",
        "rev": "4e8375d1aaff9ec1aa58e4d641823e266d36e53d",
        "license": "CC BY 4.0",
        "license_file": "LICENSE-DOCS.md",
        "include": r"src/content/.+\.md",
    },
]

CLASSES = {
    "crlf": re.compile(r"\r\n"),
    "footnote": re.compile(r"(?m)^\[\^[^\]\s]+\]:"),
    "callout": re.compile(r"(?m)^>\s*\[!\w+"),
    "embed": re.compile(r"!\[\[[^\]\n]+\]\]"),
    "html": re.compile(r"(?m)^ {0,3}</?([a-zA-Z][a-zA-Z0-9-]*)(\s|>|/>|$)"),
    "table": re.compile(r"(?m)^ {0,3}\|.*\|[ \t]*\r?\n {0,3}\|[ \t:|-]+\|[ \t]*$"),
    "nested_list": re.compile(
        r"(?m)^ {0,3}([-*+]|\d+[.)])[ \t].*\r?\n( {2,}|\t)+([-*+]|\d+[.)])[ \t]"
    ),
    "code_fence": re.compile(r"(?m)^ {0,3}(```|~~~)"),
    "frontmatter": re.compile(r"\A---\r?\n"),
}
TR_LETTERS = set("ğĞışŞİ")

SRC = DATA / "src" / "d_notes"
NOTES = DATA / "d_notes" / "notes"
EDGE = DATA / "d_notes_edge"
MANIFEST = DATA / "MANIFEST.d_notes.json"


def sha256(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def classes_of(text: str) -> list[str]:
    return [c for c, r in CLASSES.items() if r.search(text)]


def turkish(text: str) -> bool:
    return sum(ch in TR_LETTERS for ch in text) >= TR_MIN


def blob(repo: Path, rev: str, path: str) -> bytes:
    r = subprocess.run(
        ["git", "cat-file", "blob", f"{rev}:{path}"], cwd=repo, capture_output=True, check=True
    )
    return r.stdout


def fetch_at(url: str, rev: str, dest: Path, retries: int = 3) -> None:
    """`dest` (a bare repository, created if absent) holds the commit `rev`
    of `url` with its whole tree (`git fetch --depth 1 origin <rev>`);
    nothing is checked out, blobs are read with `git cat-file`."""
    if not (dest / "HEAD").exists():
        dest.mkdir(parents=True, exist_ok=True)
        git("init", "--quiet", "--bare", cwd=dest)
    if git("cat-file", "-t", rev, cwd=dest, ok_codes=(0, 128)).strip() == "commit":
        return
    for _ in range(retries):
        try:
            git("fetch", "--quiet", "--depth", "1", url, rev, cwd=dest)
            break
        except RuntimeError as e:
            last = e
    else:
        die(f"{url}: fetch of {rev} failed {retries} times: {last}")
    if git("rev-parse", f"{rev}^{{commit}}", cwd=dest).strip() != rev:
        die(f"{url}: revision {rev} not found")


def pool() -> list[dict]:
    out = []
    for s in SOURCES:
        repo = SRC / s["name"]
        fetch_at(s["url"], s["rev"], repo)
        try:
            blob(repo, s["rev"], s["license_file"])
        except subprocess.CalledProcessError:
            die(f"{s['name']}: licence file {s['license_file']} missing at {s['rev']}")
        names = git("ls-tree", "-r", "--name-only", s["rev"], cwd=repo).splitlines()
        inc = re.compile(s["include"])
        for p in sorted(names):
            if not inc.fullmatch(p):
                continue
            b = blob(repo, s["rev"], p)
            if not (MIN_BYTES <= len(b) <= MAX_BYTES):
                continue
            try:
                t = b.decode("utf-8")
            except UnicodeDecodeError:
                continue
            out.append(
                {
                    "source": s["name"],
                    "source_path": p,
                    "bytes": b,
                    "classes": classes_of(t),
                    "turkish": turkish(t),
                }
            )
    out.sort(key=lambda n: (n["source"], n["source_path"]))
    return out


def select(notes: list[dict]) -> list[dict]:
    rng = random.Random(SEED)
    chosen: list[int] = []
    taken: set[int] = set()

    def count(pred) -> int:
        return sum(1 for i in chosen if pred(notes[i]))

    def take(pred, quota: int, minimum: int, what: str) -> None:
        cands = [i for i, n in enumerate(notes) if pred(n) and i not in taken]
        rng.shuffle(cands)
        have = count(pred)
        if have + len(cands) < minimum:
            die(f"d_notes: only {have + len(cands)} eligible notes of {what} (at least {minimum})")
        for i in cands:
            if have >= quota:
                break
            chosen.append(i)
            taken.add(i)
            have += 1

    for c in CLASSES:
        take(lambda n, c=c: c in n["classes"], CLASS_QUOTA, CLASS_MIN, f"class {c}")
    take(lambda n: n["turkish"], TR_QUOTA, 30, "Turkish")
    rest = [i for i in range(len(notes)) if i not in taken]
    rng.shuffle(rest)
    for i in rest:
        if len(chosen) >= N_NOTES:
            break
        chosen.append(i)
        taken.add(i)
    if len(chosen) < 300:
        die(f"d_notes: {len(chosen)} notes (at least 300)")
    return [
        notes[i]
        for i in sorted(chosen, key=lambda i: (notes[i]["source"], notes[i]["source_path"]))
    ]


# ---- edge cases -------------------------------------------------------------------------

WORDS = [
    "alpha",
    "beta",
    "gamma",
    "delta",
    "çay",
    "şeker",
    "ılık",
    "öğle",
    "İstanbul",
    "ünlü",
    "note",
    "block",
    "line",
    "kernel",
    "lineage",
    "proposal",
    "canonical",
    "🎉",
    "✓",
    "naïve",
    "café",
    "straße",
]


def words(rng: random.Random, lo: int = 3, hi: int = 12) -> str:
    return " ".join(rng.choice(WORDS) for _ in range(rng.randint(lo, hi)))


def p_heading(rng):
    if rng.random() < 0.25:
        return f"{words(rng, 1, 4)}\n{rng.choice(['===', '---', '======'])}"
    lvl = rng.randint(1, 6)
    closing = f" {'#' * rng.randint(1, 3)}" if rng.random() < 0.2 else ""
    return f"{' ' * rng.randint(0, 3)}{'#' * lvl} {words(rng, 1, 5)}{closing}"


def p_paragraph(rng):
    lines = [words(rng) for _ in range(rng.randint(1, 4))]
    if rng.random() < 0.3:
        lines[0] = " " * rng.randint(1, 3) + lines[0]
    if rng.random() < 0.2:
        lines = [ln + "  " for ln in lines]
    return "\n".join(lines)


def p_list(rng, nested: bool = False):
    out = []
    ordered = rng.random() < 0.4
    for i in range(rng.randint(1, 4)):
        marker = f"{i + 1}." if ordered else rng.choice("-*+")
        out.append(f"{marker} {words(rng, 1, 6)}")
        if nested or rng.random() < 0.3:
            ind = " " * (len(marker) + 1)
            for _ in range(rng.randint(1, 3)):
                out.append(f"{ind}{rng.choice('-*+')} {words(rng, 1, 5)}")
                if rng.random() < 0.3:
                    out.append(f"{ind}  - {words(rng, 1, 3)}")
        if rng.random() < 0.2:
            out.append(f"{' ' * (len(marker) + 1)}{words(rng)}")
    return "\n".join(out)


def p_code(rng):
    fence = rng.choice(["```", "~~~", "````"])
    lang = rng.choice(["", "rust", "python", "md"])
    body = "\n".join(
        rng.choice(
            ["fn main() {}", "    indented", "", "# not a heading", "> not a quote", "| a |"]
        )
        for _ in range(rng.randint(1, 5))
    )
    return f"{fence}{lang}\n{body}\n{fence}"


def p_table(rng):
    cols = rng.randint(1, 4)
    head = "| " + " | ".join(words(rng, 1, 2) for _ in range(cols)) + " |"
    delim = "|" + "|".join(rng.choice(["---", ":--", "--:", ":-:"]) for _ in range(cols)) + "|"
    rows = [
        "| " + " | ".join(words(rng, 1, 3) for _ in range(cols)) + " |"
        for _ in range(rng.randint(1, 4))
    ]
    return "\n".join([head, delim, *rows])


def p_quote(rng):
    lines = [f"> {words(rng)}" for _ in range(rng.randint(1, 3))]
    if rng.random() < 0.3:
        lines.append(f"> > {words(rng)}")
    return "\n".join(lines)


def p_callout(rng):
    kind = rng.choice(["note", "tip", "warning", "info", "NOTE", "quote"])
    fold = rng.choice(["", "-", "+"])
    lines = [f"> [!{kind}]{fold} {words(rng, 0, 4)}".rstrip()]
    lines += [f"> {words(rng)}" for _ in range(rng.randint(0, 3))]
    if rng.random() < 0.4:  # nested callout
        lines += ["> > [!tip] iç", f"> > {words(rng)}"]
    if rng.random() < 0.2:
        lines.append(">")
        lines.append(f"> {words(rng)}")
    return "\n".join(lines)


def p_html(rng):
    kind = rng.randint(0, 4)
    if kind == 0:
        return f"<div>\n{words(rng)}\n</div>"
    if kind == 1:  # a blank line inside the HTML (CommonMark ends the block there)
        return f"<div>\n\n{words(rng)}</div>"
    if kind == 2:
        return f"<details>\n<summary>{words(rng, 1, 3)}</summary>\n\n{words(rng)}\n\n</details>"
    if kind == 3:
        return f"<!-- {words(rng)} -->"
    return f'<img src="a.png" alt="{words(rng, 1, 2)}">'


def p_footnote_ref(rng):
    return f"{words(rng)}[^{rng.randint(1, 3)}] {words(rng, 1, 4)}"


def p_footnote_def(rng):
    d = f"[^{rng.randint(1, 3)}]: {words(rng)}"
    if rng.random() < 0.3:
        d += f"\n    {words(rng)}"
    return d


def p_embed(rng):
    k = rng.randint(0, 3)
    a, b = f"{words(rng, 1, 2)}.png", f"{words(rng, 1, 2)}"
    if k == 0:
        return f"![[{a}]]"
    if k == 1:  # two embeds on one line
        return f"![[{a}]] ve ![[{b}#{words(rng, 1, 1)}]]"
    if k == 2:
        return f"{words(rng, 1, 3)} ![[{b}|{words(rng, 1, 2)}]] {words(rng, 1, 3)}"
    return f"![[{b}]]\n![[{a}]]"


def p_rule(rng):
    return rng.choice(["---", "***", "___", "- - -"])


def p_linkref(rng):
    return f"[{words(rng, 1, 1)}]: https://example.org/{rng.randint(1, 99)}"


def p_turkish(rng):
    return f"## Başlık {rng.choice(['🎉', '☕', 'İçerik', 'çğıöşü'])} {words(rng, 1, 3)}"


BASE = [p_heading, p_paragraph, p_list, p_quote, p_code, p_rule, p_linkref, p_table]
OWN = {
    "html": [p_html],
    "footnote": [p_footnote_ref, p_footnote_def],
    "callout": [p_callout],
    "embed": [p_embed],
    "table": [p_table],
    "nested_list": [lambda r: p_list(r, nested=True)],
    "code_fence": [p_code],
    "frontmatter": [p_paragraph, p_heading],
    "crlf": BASE,
    "bom": BASE,
    "turkish_emoji": [p_turkish, p_paragraph],
}
EDGE_CLASSES = list(OWN)


def edge_doc(cls: str, n: int) -> bytes:
    rng = random.Random(f"{SEED}:edge:{cls}:{n}")
    parts = [rng.choice(BASE)(rng) for _ in range(rng.randint(1, 7))]
    for _ in range(rng.randint(2, 3)):
        parts.insert(rng.randint(0, len(parts)), rng.choice(OWN[cls])(rng))
    if cls == "footnote" and rng.random() < 0.6:
        parts.append(p_footnote_def(rng))  # a definition at the end of the document
    text = ""
    for i, p in enumerate(parts):
        if i:
            text += rng.choice(["\n\n", "\n\n", "\n\n\n", "\n"]) if rng.random() < 0.9 else "\n\n"
        text += p
    text += rng.choice(["\n", "", "\n\n"])
    if cls == "frontmatter":
        fm = f"---\ntitle: {words(rng, 1, 3)}\ntags: [{words(rng, 1, 1)}]\n---\n"
        text = fm + ("" if rng.random() < 0.15 else rng.choice(["", "\n"]) + text)
    if cls == "crlf":
        text = text.replace("\n", "\r\n")
    if cls == "bom":
        text = "﻿" + text
    return text.encode("utf-8")


def edge() -> dict[str, list[bytes]]:
    return {c: [edge_doc(c, n) for n in range(EDGE_PER_CLASS)] for c in EDGE_CLASSES}


def edge_digest(docs: dict[str, list[bytes]]) -> dict[str, str]:
    return {c: sha256(b"".join(sha256(d).encode() + b"\n" for d in ds)) for c, ds in docs.items()}


# ---- writing and checking -----------------------------------------------------------------


def manifest(sel: list[dict], docs: dict[str, list[bytes]]) -> dict:
    counts = {c: sum(c in n["classes"] for n in sel) for c in CLASSES}
    return {
        "name": "d_notes",
        "round": "P2-B",
        "seed": SEED,
        "rules": {
            "n_notes": N_NOTES,
            "class_quota": CLASS_QUOTA,
            "class_min": CLASS_MIN,
            "tr_quota": TR_QUOTA,
            "tr_min_letters": TR_MIN,
            "bytes": [MIN_BYTES, MAX_BYTES],
            "edge_per_class": EDGE_PER_CLASS,
        },
        "sources": [
            {k: s[k] for k in ("name", "url", "rev", "license", "license_file", "include")}
            for s in SOURCES
        ],
        "notes": [
            {
                "path": f"{n['source']}/{n['source_path']}",
                "source": n["source"],
                "source_path": n["source_path"],
                "sha256": sha256(n["bytes"]),
                "classes": n["classes"],
                "turkish": n["turkish"],
            }
            for n in sel
        ],
        "class_counts": counts,
        "turkish": sum(n["turkish"] for n in sel),
        "edge_classes": EDGE_CLASSES,
        "edge_sha256": edge_digest(docs),
    }


NOTICE = """# D-NOTES — attribution

The notes under `notes/<source>/` are copied unchanged (byte for byte, at the
pinned revision) from the repositories below, under their licences. They are
used only as a measurement set (round P2-B, alignment of block locations).

{rows}
"""


def write_edge(docs: dict[str, list[bytes]]) -> None:
    for c, ds in docs.items():
        d = EDGE / c
        d.mkdir(parents=True, exist_ok=True)
        for i, b in enumerate(ds):
            (d / f"{i:03d}.md").write_bytes(b)


def build() -> None:
    sel = select(pool())
    docs = edge()
    if NOTES.exists():
        for p in sorted(NOTES.rglob("*"), reverse=True):
            p.unlink() if p.is_file() else p.rmdir()
    for n in sel:
        f = NOTES / n["source"] / n["source_path"]
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_bytes(n["bytes"])
    rows = "\n".join(
        f"- `{s['name']}`: {s['url']} at `{s['rev']}` — {s['license']} (`{s['license_file']}`)"
        for s in SOURCES
    )
    (DATA / "d_notes" / "NOTICE.md").write_text(NOTICE.format(rows=rows), encoding="utf-8")
    write_edge(docs)
    m = manifest(sel, docs)
    MANIFEST.write_text(dump_json(m), encoding="utf-8")
    print(f"d_notes: {len(sel)} notes, Turkish {m['turkish']}, classes {m['class_counts']}")


def check() -> dict:
    """The committed notes and the regenerated edge cases against the
    manifest; returns it. Any difference is an error."""
    m = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if m["seed"] != SEED or m["edge_classes"] != EDGE_CLASSES:
        die("d_notes: the manifest names another seed or edge classes")
    on_disk = sorted(str(p.relative_to(NOTES)) for p in NOTES.rglob("*") if p.is_file())
    if on_disk != sorted(n["path"] for n in m["notes"]):
        die("d_notes: the notes on disk are not the manifest's")
    for n in m["notes"]:
        b = (NOTES / n["path"]).read_bytes()
        if sha256(b) != n["sha256"]:
            die(f"d_notes: {n['path']} does not match its sha256")
        if classes_of(b.decode("utf-8")) != n["classes"]:
            die(f"d_notes: {n['path']}: classes differ")
    docs = edge()
    if edge_digest(docs) != m["edge_sha256"]:
        die("d_notes: the edge cases differ from the manifest's digest")
    write_edge(docs)
    return m


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    if a.check:
        m = check()
        print(f"d_notes: verified ({len(m['notes'])} notes, {len(m['edge_classes'])} edge classes)")
    else:
        build()


if __name__ == "__main__":
    main()
