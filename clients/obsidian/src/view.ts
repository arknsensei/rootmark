// View models of the panels (no Obsidian import; component tests run on
// these): what the blame gutter shows per line, what a history line says.
import type { BlameBlock, LogEvent } from "./api";
import { actorOf } from "./commands";
import type { Alignment } from "./align";

export interface GutterMark {
  /** 0-based editor line where the block starts */
  line: number;
  lineage_id: string;
  layer: string;
  /** one letter in the gutter: C canonical, P proposal, D derived */
  label: string;
  /** CSS class: `mdk-layer-<layer>` */
  cls: string;
  /** hover text: layer, author kind, source, last event */
  title: string;
}

export function gutterMarks(al: Alignment<BlameBlock>): GutterMark[] {
  if (!al.aligned) return [];
  return al.pairs.map(({ block: b, range }) => ({
    line: range.start,
    lineage_id: b.lineage_id,
    layer: b.layer,
    label: b.layer === "canonical" ? "C" : b.layer === "proposal" ? "P" : "D",
    cls: `mdk-layer-${b.layer}${b.inactive ? " mdk-inactive" : ""}`,
    title: [
      b.layer + (b.layer_source === "derived" ? " (derived from the log)" : ""),
      b.author_kind,
      b.source === "" ? "source: —" : `source: ${b.source}`,
      `created at seq ${String(b.seq_created)}, last event seq ${String(b.seq_last)}`,
    ].join(" · "),
  }));
}

/** One line of the history pop-up. */
export function historyLine(e: LogEvent): string {
  const m = e.meta;
  const extra: string[] = [];
  if (typeof m["reason"] === "string") extra.push(`reason: ${m["reason"]}`);
  if (typeof m["from"] === "string" && typeof m["to"] === "string") extra.push(`${m["from"]} → ${m["to"]}`);
  if (typeof m["source"] === "string") extra.push(`source: ${m["source"]}`);
  const prov = m["provenance"];
  if (typeof prov === "object" && prov !== null) {
    const p = prov as { author_kind?: unknown; source?: unknown };
    if (typeof p.author_kind === "string") extra.push(`written by ${p.author_kind}${typeof p.source === "string" && p.source !== "" ? ` (${p.source})` : ""}`);
  }
  const actor = actorOf(m);
  return `#${String(e.seq)} ${e.kind} · ${actor}${extra.length ? " · " + extra.join(" · ") : ""}`;
}
