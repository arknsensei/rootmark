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
      reason: `mdk has ${String(blame.length)} blocks, the editor ${String(ranges.length)}: not aligned (is the note saved and ingested?)`,
    };
  }
  return { aligned: true, pairs: blame.map((block, i) => ({ block, range: ranges[i] as BlockRange })) };
}
