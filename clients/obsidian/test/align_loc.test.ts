// P2-B/B4 (a): the blame gutter aligns by the server's block locations
// (contract v2 §2.5) when they are the note's bytes and pass the slice
// check; `loc_stale` says "stale" and guesses no line; without `loc` (a v1
// server) the order pairing stays. The fixtures are real answers of
// `rootmark blame --loc` for the texts below.
import { describe, expect, it } from "vitest";
import { align, alignByLoc, normalizeBlock, sliceMatches, type BlockRange } from "../src/align";
import type { BlameBlock, BlameResult } from "../src/api";
import { contentHash, hash32 } from "../src/blake3";
import { alignNote } from "../src/commands";

const TEXT = "---\ntitle: örnek\n---\n# Başlık 🎉\n\nBir  paragraf\niki satır.\n\n- madde\n  - iç\n\n    girintili kod\n    ikinci\n\n> [!note] Not\n> callout\n";
const H = "0fdc9c885b8f20e21e84db2ccb5ef362";
const loc = (bs: number, be: number, ls: number, le: number) => ({ byte_start: bs, byte_end: be, line_start: ls, line_end: le, content_hash: H });
const B = (lineage: string, version: string, l: ReturnType<typeof loc> | undefined, layer = "canonical"): BlameBlock => ({
  lineage_id: lineage,
  version_id: version,
  layer: layer as BlameBlock["layer"],
  layer_source: "event",
  author_kind: "Human",
  source: "",
  seq_created: 0,
  seq_last: 0,
  inactive: false,
  ...(l === undefined ? {} : { loc: l }),
});
const BLOCKS: BlameBlock[] = [
  B("2dc7ccff12b8a5b507415587ce0db072", "19115f797b47e9007d8379cc83f388ac", loc(22, 37, 4, 4)),
  B("05f4026b24e8269b4d1b682fabb5fda7", "61a7aeac60e2638b6af06d34b6181099", loc(39, 64, 6, 7)),
  B("46fd06147cb1a46a734c7681d2d38153", "4ca40f0a24489ddfa174579ce81cafb0", loc(66, 111, 9, 13), "proposal"),
  B("449bc413f0593cd9a7437bf4b3e6d8f8", "3b6c70e4743aad81b744571c44587fc1", loc(113, 136, 15, 16), "unverified"),
];
const bytes = (s: string) => new TextEncoder().encode(s);
/** A v1 server's block: no `loc`. */
function withoutLoc(b: BlameBlock): BlameBlock {
  const c = { ...b };
  delete c.loc;
  return c;
}

describe("the kernel's hashes and normalization in the client", () => {
  it("contentHash of the note is the server's content_hash", () => {
    expect(contentHash(bytes(TEXT))).toBe(H);
  });

  it("normalization collapses white space (not U+FEFF) and keeps code", () => {
    expect(normalizeBlock("  a \t b\r\nc  ", false)).toBe("a b c");
    expect(normalizeBlock("﻿a", false)).toBe("﻿a");
    expect(normalizeBlock("  x\r\n  y", true)).toBe("  x\n  y");
  });

  it("the slice check holds for every block of the fixture, and for an indented code block", () => {
    for (const b of BLOCKS) expect(b.loc !== undefined && sliceMatches(bytes(TEXT), b.loc, b.version_id, hash32)).toBe(true);
    // `Para.\n\n    code line\n    second\n`: the location starts after the
    // first line's indentation (K-P2B-T1), the slice is still the block
    const code = "Para.\n\n    code line\n    second\n";
    expect(sliceMatches(bytes(code), { byte_start: 11, byte_end: 31, line_start: 3, line_end: 4, content_hash: "b8d02525c119dfd1bba821b946e99484" }, "143331cd0f7046f65b4f33232b2af927", hash32)).toBe(true);
    // a wrong range fails
    expect(sliceMatches(bytes(TEXT), loc(23, 37, 4, 4), "19115f797b47e9007d8379cc83f388ac", hash32)).toBe(false);
  });
});

describe("alignByLoc", () => {
  it("aligns every block by its lines when the bytes are the located version", () => {
    const al = alignByLoc(BLOCKS, false, bytes(TEXT), contentHash, hash32, [{ line: 3, heading: "Başlık 🎉" }]);
    expect(al?.aligned).toBe(true);
    if (al?.aligned !== true) return;
    expect(al.pairs.map((p) => [p.range.start, p.range.end, p.block.layer])).toEqual([
      [3, 3, "canonical"],
      [5, 6, "canonical"],
      [8, 12, "proposal"],
      [14, 15, "unverified"],
    ]);
    expect(al.pairs[1]?.range.heading).toBe("Başlık 🎉");
  });

  it("stale: the panel says so and guesses no line", () => {
    const al = alignByLoc(BLOCKS, true, bytes(TEXT), contentHash, hash32);
    expect(al).toMatchObject({ aligned: false, stale: true });
  });

  it("other bytes than the located ones (unsaved, not yet ingested): not aligned", () => {
    const al = alignByLoc(BLOCKS, false, bytes(TEXT + "x\n"), contentHash, hash32);
    expect(al?.aligned).toBe(false);
    expect(al).not.toHaveProperty("stale");
  });

  it("no loc (a v1 server) or a failed slice check: null, the caller pairs by order", () => {
    expect(alignByLoc(BLOCKS.map(withoutLoc), undefined, bytes(TEXT), contentHash, hash32)).toBeNull();
    const broken = BLOCKS.map((b, i) => (i === 0 ? { ...b, version_id: "0".repeat(32) } : b));
    expect(alignByLoc(broken, false, bytes(TEXT), contentHash, hash32)).toBeNull();
  });
});

describe("alignNote (the gutter's choice)", () => {
  const ranges: BlockRange[] = [
    { start: 3, end: 3, heading: null },
    { start: 5, end: 6, heading: null },
    { start: 8, end: 12, heading: null },
    { start: 14, end: 15, heading: null },
  ];
  const r: BlameResult = { doc_path: "n.md", at: null, blocks: BLOCKS, loc_stale: false };

  it("by location when it can", () => {
    expect(alignNote(r, bytes(TEXT), []).by).toBe("loc");
  });

  it("by order for a v1 server (as before)", () => {
    const v1: BlameResult = { doc_path: "n.md", at: null, blocks: BLOCKS.map(withoutLoc) };
    const { al, by } = alignNote(v1, null, ranges);
    expect(by).toBe("order");
    expect(al).toEqual(align(v1.blocks, ranges));
  });

  it("stale without bytes read: stale, never the order pairing", () => {
    const { al, by } = alignNote({ ...r, loc_stale: true }, null, ranges);
    expect(by).toBe("loc");
    expect(al).toMatchObject({ aligned: false, stale: true });
  });
});
