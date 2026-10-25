"""P2-B/B2 — alignment of block locations on D-NOTES (GOAL P2-B 3; B-3,
B-4). Pre-registered: written and committed with the corpus before any
measurement (`eval.datasets.d_notes`, ASSUMPTIONS P2-B B2-*).

Each set (the real notes; each edge class) is copied into its own vault,
ingested with `rootmark ingest-dir`, and every document's blocks are read
with `rootmark replay --loc` (the server's location code: `loc`,
`loc_stale`, the block's version and normalized text). Per block:

- **(i) slice:** `file[byte_start..byte_end]`, parsed again on its own
  (`rootmark_core.parse_document`), is exactly one block with the block's
  type and `normalized_text`; `line_start` is the line of `byte_start` and
  `line_end` the line of `byte_end − 1` (1-based, counted on the file's
  bytes); `content_hash` is the hash of the file's bytes; `loc_stale` is
  false (the file was just ingested);
- **(ii) cover:** the blocks are in order and do not overlap
  (`byte_start ≥` the previous `byte_end`); every gap between blocks (and
  before the first, after the last) parsed on its own gives no block —
  blank lines, front matter, link reference definitions or other content
  spec §2.2 does not count (an empty paragraph); a gap that parses into a
  block is a cover failure of the block after it (of the last block for
  the trailing gap). The gaps are classified and counted: `blank`,
  `frontmatter`, `link_ref`, `no_block_other`.

**Alignment error** of a set = blocks failing (i) or (ii) / all blocks.
B-3: real notes ≤ 0.005; B-4: each edge class ≤ 0.02 (reported per class).

**Old method** (sequence pairing of `clients/obsidian/src/align.ts`; no
threshold, reported for comparison): the share of documents the editor's
block map could pair by order. Obsidian's section map is not available
outside Obsidian; the proxy counts a document as pairable when every block
is a heading, paragraph, top-level list item, code, table or block quote
that is not a callout (`> [!`) and no gap holds content other than blank
lines and front matter (an HTML block, thematic break, footnote, callout,
link reference definition or comment has no sure counterpart there).

Writes `p2b/align.json` into the RUN (no timing, no path: two runs are
byte-equal). Exit 0 when B-3 and B-4 hold, 3 otherwise (the record is
still written).

    python -m eval.p2b_align --run <RUN> --rootmark <binary>
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import blake3
import rootmark_core

from eval import runinfo
from eval.datasets import d_notes
from eval.datasets.common import dump_json

B3_MAX = 0.005
B4_MAX = 0.02
PAIRABLE = {"Heading", "Paragraph", "ListItem", "Code", "Table", "Blockquote"}
LINK_REF = re.compile(r"^ {0,3}\[[^\]]+\]:\s*\S")


def content_hash(b: bytes) -> str:
    return blake3.blake3(b).hexdigest()[:32]


def line_of(b: bytes, off: int) -> int:
    return 1 + b[:off].count(b"\n")


def run(cmd: list[str]) -> str:
    r = subprocess.run(cmd, capture_output=True, check=False)
    if r.returncode != 0:
        raise SystemExit(
            f"p2b_align: {' '.join(cmd[:3])} … failed: {r.stderr.decode('utf-8', 'replace')}"
        )
    return r.stdout.decode("utf-8")


def gap_class(text: str, head: bool) -> str:
    if text.strip() == "":
        return "blank"
    if head and re.match(r"\A---\r?\n", text):
        return "frontmatter"
    lines = [ln for ln in text.splitlines() if ln.strip()]
    if lines and all(LINK_REF.match(ln) for ln in lines):
        return "link_ref"
    return "no_block_other"


def measure_doc(raw: bytes, rep: dict) -> dict:
    text = raw.decode("utf-8")
    blocks = rep["blocks_detail"]
    bad: set[int] = set()
    slice_fail = cover_fail = 0
    gaps = {"blank": 0, "frontmatter": 0, "link_ref": 0, "no_block_other": 0}
    pairable = True
    if rep.get("loc_stale") is not False:
        bad.update(range(len(blocks)))
    prev_end = 0
    for i, b in enumerate(blocks):
        loc = b.get("loc")
        if loc is None or not (0 <= loc["byte_start"] < loc["byte_end"] <= len(raw)):
            # no usable range: fails (i) and (ii)
            slice_fail += 1
            cover_fail += 1
            bad.add(i)
            pairable = False
            continue
        bs, be = loc["byte_start"], loc["byte_end"]
        # (i) the slice is the block, the lines are the bytes' lines
        try:
            piece = raw[bs:be].decode("utf-8")
            drafts = rootmark_core.parse_document(piece, "slice.md")[0]
        except UnicodeDecodeError:
            drafts = []
        if not (
            len(drafts) == 1
            and drafts[0]["normalized_text"] == b["normalized_text"]
            and drafts[0]["block_type"] == b["block_type"]
            and loc["line_start"] == line_of(raw, bs)
            and loc["line_end"] == line_of(raw, be - 1)
            and loc["content_hash"] == content_hash(raw)
        ):
            slice_fail += 1
            bad.add(i)
        # (ii) order, and the gap before the block holds no block
        if bs < prev_end:
            cover_fail += 1
            bad.add(i)
        else:
            gap = raw[prev_end:bs].decode("utf-8", "replace")
            if rootmark_core.parse_document(gap, "gap.md")[0]:
                cover_fail += 1
                bad.add(i)
                pairable = False
            else:
                g = gap_class(gap, prev_end == 0)
                gaps[g] += 1
                if g not in ("blank", "frontmatter"):
                    pairable = False
        prev_end = max(prev_end, be)
        if b["block_type"] not in PAIRABLE or (
            b["block_type"] == "Blockquote" and text[bs:be].lstrip().startswith("> [!")
        ):
            pairable = False
    if blocks and prev_end <= len(raw):
        tail = raw[prev_end:].decode("utf-8", "replace")
        if rootmark_core.parse_document(tail, "gap.md")[0]:
            cover_fail += 1
            bad.add(len(blocks) - 1)
        else:
            g = gap_class(tail, False)
            gaps[g] += 1
            if g not in ("blank", "frontmatter"):
                pairable = False
    return {
        "blocks": len(blocks),
        "bad": len(bad),
        "slice_fail": slice_fail,
        "cover_fail": cover_fail,
        "gaps": gaps,
        "pairable": pairable,
    }


def measure_set(rootmark: str, docs: dict[str, bytes], work: Path) -> dict:
    v = work / "vault"
    for rel, b in docs.items():
        f = v / rel
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_bytes(b)
    run([rootmark, "ingest-dir", str(v), str(v)])
    tot = {"docs": 0, "blocks": 0, "bad": 0, "slice_fail": 0, "cover_fail": 0, "pairable_docs": 0}
    gaps = {"blank": 0, "frontmatter": 0, "link_ref": 0, "no_block_other": 0}
    failing: list[dict] = []
    empty = 0
    for rel in sorted(docs):
        out = subprocess.run(
            [rootmark, "replay", "--loc", str(v), rel], capture_output=True, check=False
        )
        if out.returncode != 0:
            # a document with no block has no snapshot to replay only when it
            # has no content block at all
            if rootmark_core.parse_document(docs[rel].decode("utf-8"), rel)[0]:
                raise SystemExit(f"p2b_align: replay of {rel} failed: {out.stderr.decode()}")
            empty += 1
            continue
        m = measure_doc(docs[rel], json.loads(out.stdout))
        tot["docs"] += 1
        for k in ("blocks", "bad", "slice_fail", "cover_fail"):
            tot[k] += m[k]
        tot["pairable_docs"] += int(m["pairable"])
        for k, n in m["gaps"].items():
            gaps[k] += n
        if m["bad"]:
            failing.append(
                {
                    "doc": rel,
                    "bad": m["bad"],
                    "slice_fail": m["slice_fail"],
                    "cover_fail": m["cover_fail"],
                }
            )
    tot["empty_docs"] = empty
    tot["gaps"] = gaps
    tot["error"] = tot["bad"] / tot["blocks"] if tot["blocks"] else 0.0
    tot["old_method_pairable"] = tot["pairable_docs"] / tot["docs"] if tot["docs"] else 0.0
    tot["failing_docs"] = failing
    return tot


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--run", required=True)
    ap.add_argument("--rootmark", required=True)
    a = ap.parse_args(argv)
    rootmark = str(Path(a.rootmark).resolve())
    m = d_notes.check()
    out_dir = runinfo.run_dir(a.run, create=False) / "p2b"
    out_dir.mkdir(parents=True, exist_ok=True)
    res: dict = {
        "manifest_seed": m["seed"],
        "notes_digest": content_hash("".join(n["sha256"] for n in m["notes"]).encode()),
        "edge_sha256": m["edge_sha256"],
    }
    with tempfile.TemporaryDirectory(prefix="rootmark-p2b-align-") as tmp:
        notes = {n["path"]: (d_notes.NOTES / n["path"]).read_bytes() for n in m["notes"]}
        res["notes"] = measure_set(rootmark, notes, Path(tmp) / "notes")
        tr = {n["path"]: notes[n["path"]] for n in m["notes"] if n["turkish"]}
        res["notes_turkish"] = measure_set(rootmark, tr, Path(tmp) / "notes_tr")
        res["edge"] = {}
        for c in m["edge_classes"]:
            docs = {
                f"{c}/{p.name}": p.read_bytes() for p in sorted((d_notes.EDGE / c).glob("*.md"))
            }
            res["edge"][c] = measure_set(rootmark, docs, Path(tmp) / f"edge_{c}")
            shutil.rmtree(Path(tmp) / f"edge_{c}")
    b3 = res["notes"]["error"] <= B3_MAX
    b4 = {c: r["error"] <= B4_MAX for c, r in res["edge"].items()}
    res["criteria"] = {
        "B-3": {"error": res["notes"]["error"], "max": B3_MAX, "pass": b3},
        "B-4": {
            "per_class": {c: res["edge"][c]["error"] for c in res["edge"]},
            "max": B4_MAX,
            "pass_per_class": b4,
            "pass": all(b4.values()),
        },
    }
    res["pass"] = b3 and all(b4.values())
    (out_dir / "align.json").write_text(dump_json(res), encoding="utf-8")
    print(
        f"p2b-align: B-3 notes error {res['notes']['error']:.5f} ({'pass' if b3 else 'FAIL'}); "
        f"B-4 max class error {max(res['criteria']['B-4']['per_class'].values()):.5f} "
        f"({'pass' if all(b4.values()) else 'FAIL'}); old method pairable "
        f"{res['notes']['old_method_pairable']:.3f}"
    )
    return 0 if res["pass"] else 3


if __name__ == "__main__":
    sys.exit(main())
