// View models of the panels (no Obsidian import; component tests run on
// these): what the blame gutter shows per line, what a history line says.
import type { BlameBlock, IndexStatus, LogEvent } from "./api";
import { actorOf } from "./commands";
import type { Alignment } from "./align";

export interface GutterMark {
  /** 0-based editor line where the block starts */
  line: number;
  lineage_id: string;
  layer: string;
  /** one letter in the gutter: C canonical, P proposal, D derived */
  label: string;
  /** CSS class: `rootmark-layer-<layer>` */
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
    cls: `rootmark-layer-${b.layer}${b.inactive ? " rootmark-inactive" : ""}`,
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

/** The read-only past view of a note (checkpoint panel): its blocks as of
 * `at`, one line each. */
export function pastView(doc: string, at: string, blocks: readonly BlameBlock[] | null): string {
  if (blocks === null) return `${doc} had no snapshot at checkpoint ${at}`;
  return [
    `${doc} at ${at} (read only)`,
    ...blocks.map((b, i) => `${String(i + 1).padStart(3)} ${b.layer.padEnd(9)} ${b.author_kind.padEnd(6)} ${b.source === "" ? "—" : b.source} · ${b.lineage_id.slice(0, 8)}`),
  ].join("\n");
}

/** What the connection status says about the vault's index (contract §2.5
 * `status.index`, K-E1.2): null when it is `ok`; otherwise a short label for
 * the status bar and the full line (state, formats, what is unavailable, how
 * to rebuild) for its tooltip and the notice. */
export function indexNotice(ix: IndexStatus): { label: string; detail: string } | null {
  if (ix.state === "ok") return null;
  const rebuild = "rebuild: run `rootmark index build <vault>`";
  const restart = ix.auto_rebuild
    ? "or restart `rootmark serve` ([index] auto_rebuild = true rebuilds it at start)"
    : "or set [index] auto_rebuild = true in .rootmark/policy.toml and restart `rootmark serve`";
  const unavailable = "search is unavailable (index_stale) until then; blame, history, checkpoints and traces work";
  switch (ix.state) {
    case "stale":
      return {
        label: "index stale",
        detail: `index stale: format ${ix.format ?? "?"}, this rootmark expects ${ix.expected_format}; ${unavailable}; ${rebuild}, ${restart}`,
      };
    case "missing":
      return { label: "index missing", detail: `index missing: no index in the vault; ${unavailable}; ${rebuild}, ${restart}` };
    case "damaged":
      return {
        label: "index damaged",
        detail: `index damaged: its manifest cannot be read; ${unavailable}; ${rebuild} (a damaged index is never rebuilt automatically)`,
      };
    default:
      return { label: `index ${ix.state}`, detail: `index state ${ix.state} (unknown to this plugin); ${rebuild}` };
  }
}
