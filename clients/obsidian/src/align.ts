// Where the blocks of `blame` are in the editor (blame gutter, TASKS E2).
// `blame` lists a document's blocks in document order (contract §2.5) but
// carries no positions; the editor's own block map — Obsidian's
// `metadataCache` sections and top-level list items — gives positions in
// the same block model (spec §2.2: heading, paragraph, each top-level list
// item, code, table, blockquote; front matter no block). The two are paired
// by order only when the counts agree; otherwise there is no alignment and
// the panel says so (it never guesses a line).

/** A section of Obsidian's `CachedMetadata.sections` (lines 0-based). */
export interface EditorSection {
  type: string;
  start: number;
  end: number;
}

/** A list item of `CachedMetadata.listItems` (`parent < 0`: top level). */
export interface EditorListItem {
  start: number;
  end: number;
  parent: number;
}

export interface BlockRange {
  start: number;
  end: number;
  /** the heading path's last heading text, if the editor knows it */
  heading: string | null;
}

/** Obsidian section types that are exactly one spec block (spec §2.2);
 * `list` is one block per top-level item; `yaml` (front matter) is none.
 * Any other type (thematic break, HTML, comment, footnote, a link
 * reference definition, or a type this client does not know) has no sure
 * counterpart, so the note is not aligned: an allow-list, never a guess. */
const ONE_BLOCK = new Set(["heading", "paragraph", "code", "table", "blockquote"]);

/** Obsidian marks a top-level item with a negative `parent` (minus the
 * list's first line); a list on line 0 gives -0, which is still top level
 * (a nested item's parent is a line number ≥ 0, never -0). */
const isTopLevel = (parent: number): boolean => parent < 0 || Object.is(parent, -0);

/** The editor's blocks in document order, in the spec's block model; `[]`
 * when a section has no sure counterpart (then nothing is aligned). */
export function editorBlocks(
  sections: readonly EditorSection[],
  listItems: readonly EditorListItem[],
  headings: readonly { line: number; heading: string }[] = [],
): BlockRange[] {
  const top = listItems.filter((l) => isTopLevel(l.parent));
  const out: { start: number; end: number }[] = [];
  for (const s of sections) {
    if (s.type === "yaml") continue;
    if (s.type === "list") {
      const items = top.filter((l) => l.start >= s.start && l.end <= s.end);
      if (items.length === 0) return [];
      for (const it of items) out.push({ start: it.start, end: it.end });
      continue;
    }
    if (!ONE_BLOCK.has(s.type)) return [];
    out.push({ start: s.start, end: s.end });
  }
  const sorted = [...headings].sort((a, b) => a.line - b.line);
  return out.map((b) => {
    let h: string | null = null;
    for (const x of sorted) if (x.line <= b.start) h = x.heading;
    return { ...b, heading: h };
  });
}

export type Alignment<T> =
  | { aligned: true; pairs: { block: T; range: BlockRange }[] }
  | { aligned: false; reason: string };

/** `blame` blocks ↔ editor blocks by order; only when the counts agree. */
export function align<T>(blame: readonly T[], ranges: readonly BlockRange[]): Alignment<T> {
  if (ranges.length === 0 && blame.length > 0) {
    return {
      aligned: false,
      reason: "the editor's block map is not available or holds a section without a sure block (thematic break, HTML, footnote, …): not aligned",
    };
  }
  if (blame.length !== ranges.length) {
    return {
      aligned: false,
      reason: `rootmark has ${String(blame.length)} blocks, the editor ${String(ranges.length)}: not aligned (is the note saved and ingested?)`,
    };
  }
  return { aligned: true, pairs: blame.map((block, i) => ({ block, range: ranges[i] as BlockRange })) };
}

/** Where to jump for a block of a note (trace viewer): its start line when
 * the note's blocks align and it is among them, else the reason. */
export function blockLine(
  blame: readonly { lineage_id: string }[],
  ranges: readonly BlockRange[],
  lineage: string,
): { line: number } | { reason: string } {
  const al = align(blame, ranges);
  if (!al.aligned) return { reason: al.reason };
  const p = al.pairs.find((x) => x.block.lineage_id === lineage);
  return p === undefined
    ? { reason: `block ${lineage.slice(0, 8)} is no longer in this note (changed since the trace)` }
    : { line: p.range.start };
}

// ---- contract v2: the server's block locations (P2-B/B4 (a)) ---------------------------

import type { Loc } from "./api";


/** Unicode `White_Space` (what the kernel's normalization trims and
 * collapses; not JavaScript's `\s`, which also holds U+FEFF). */
const WS = "\\t\\n\\v\\f\\r \\u0085\\u00a0\\u1680\\u2000-\\u200a\\u2028\\u2029\\u202f\\u205f\\u3000";
const WS_RUN = new RegExp(`[${WS}]+`, "g");
const WS_EDGE = new RegExp(`^[${WS}]+|[${WS}]+$`, "g");

/** spec §2.1: NFC, line endings to `\n`; for every block but code also
 * trimmed and each white space run collapsed to one space. */
export function normalizeBlock(text: string, isCode: boolean): string {
  const lf = text.normalize("NFC").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  return isCode ? lf : lf.replace(WS_EDGE, "").replace(WS_RUN, " ");
}

/** The kernel's block types as hashed into `version_id` (spec §1). */
const BLOCK_TYPES = ["Heading", "Paragraph", "ListItem", "Code", "Table", "Blockquote", "Other"] as const;

/** The slice check of the decision rule (K-P2B-T1; contract §2.5
 * "Deneysel"): the bytes of a location, normalized, give the block's
 * `version_id` for one of the block types. */
export function sliceMatches(bytes: Uint8Array, loc: Loc, versionId: string, hash32: (s: string) => string): boolean {
  if (!(loc.byte_start >= 0 && loc.byte_start < loc.byte_end && loc.byte_end <= bytes.length)) return false;
  let text: string;
  try {
    text = new TextDecoder("utf-8", { fatal: true }).decode(bytes.subarray(loc.byte_start, loc.byte_end));
  } catch {
    return false;
  }
  return BLOCK_TYPES.some((t) => hash32(`${t}\n${normalizeBlock(text, t === "Code")}`) === versionId);
}

export type LocAlignment<T> =
  | Alignment<T>
  /** the server's locations are not the file's bytes: the panel says so
   * and guesses no line */
  | { aligned: false; stale: true; reason: string };

/** `blame` blocks ↔ editor lines from the server's locations (contract v2):
 * - `loc_stale` → not aligned, "stale" (no line guessed);
 * - the note's bytes are not the located version (`content_hash`) → not
 *   aligned (unsaved, or saved and not yet ingested);
 * - every block passes the slice check → aligned by location;
 * - otherwise (or with no `loc`: a v1 server) → `null`: the caller falls
 *   back to the order pairing (`align`).
 * `contentHash` and `hash32` are the kernel's BLAKE3 hashes of bytes and
 * of a string (first 32 hex characters). */
export function alignByLoc<T extends { version_id: string; loc?: Loc }>(
  blocks: readonly T[],
  locStale: boolean | undefined,
  bytes: Uint8Array,
  contentHash: (b: Uint8Array) => string,
  hash32: (s: string) => string,
  headings: readonly { line: number; heading: string }[] = [],
): LocAlignment<T> | null {
  if (locStale === true) {
    return { aligned: false, stale: true, reason: "stale: the note on disk is not the version rootmark ingested; no line is guessed" };
  }
  if (blocks.length === 0 || blocks.some((b) => b.loc === undefined)) return null;
  const h = contentHash(bytes);
  if (blocks.some((b) => b.loc?.content_hash !== h)) {
    return { aligned: false, reason: "the note is not the version rootmark located (unsaved, or saved and not yet ingested): not aligned" };
  }
  if (!blocks.every((b) => sliceMatches(bytes, b.loc as Loc, b.version_id, hash32))) return null;
  const sorted = [...headings].sort((a, b) => a.line - b.line);
  return {
    aligned: true,
    pairs: blocks.map((block) => {
      const l = block.loc as Loc;
      const start = l.line_start - 1;
      let heading: string | null = null;
      for (const x of sorted) if (x.line <= start) heading = x.heading;
      return { block, range: { start, end: l.line_end - 1, heading } };
    }),
  };
}
