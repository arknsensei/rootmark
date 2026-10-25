// What the VS Code commands do, as plain functions over the shared contract
// client (clients/obsidian/src/api.ts, the module both clients use): no `vscode` import, so the tests and the
// e2e harness drive exactly this. Nothing here writes a vault file: the
// only writes are the contract's HTTP tools (`promote`, `demote`). A save
// in VS Code is an unattested external change (README): this client never
// calls `attest`.

import { sliceMatches } from "../../obsidian/src/align";
import { contentHash, hash32 } from "../../obsidian/src/blake3";
import {
  describeError,
  RootmarkError,
  type BlameBlock,
  type HistoryResult,
  type Hit,
  type Loc,
  type RootmarkClient,
  type SearchArgs,
  type StatusResult,
} from "../../obsidian/src/api";

/** `X-Consumer` of this client (contract §1: a trace label, never the actor). */
export const CONSUMER = "vscode";

/** The status bar line and its tooltip. */
export function statusLine(s: StatusResult): { text: string; tooltip: string } {
  const human = s.server_policy.identity.human_id;
  return {
    text: `rootmark ${s.mdk_version} · ${s.index.state}`,
    tooltip: [
      `vault ${s.vault}`,
      `spec ${s.spec_version}, level ${String(s.level_claimed)}`,
      `index ${s.index.state} (${s.index.format ?? "none"})`,
      `promotion ${s.policy.promotion}, edit ${s.policy.edit}`,
      `unattested changes: ${s.attestation.unattested} (VS Code saves are not attested)`,
      human === null ? "no [identity] human_id: promote/demote are refused" : `human ${human}`,
    ].join("\n"),
  };
}

/** One search result line. */
export interface SearchItem {
  doc_path: string;
  layer: string;
  score: number;
  /** 0-based first line of the hit's first located block, when its
   * location is current; null otherwise (never guessed) */
  line: number | null;
  label: string;
}

/** The search options the command asks for (contract §2.2). */
export interface SearchOptions {
  mode?: "hybrid" | "lexical" | "flat";
  k?: number;
  include_proposal?: boolean;
  path_prefix?: string[];
  date_from?: string;
  date_to?: string;
}

/** `search` with filters, locations asked for; each hit with the line of
 * its first block when the hit's locations are current. */
export async function search(client: RootmarkClient, query: string, o: SearchOptions = {}): Promise<{ items: SearchItem[]; undated_excluded: number | null }> {
  const args: SearchArgs = { query, loc: true };
  if (o.mode !== undefined) args.mode = o.mode;
  if (o.k !== undefined) args.k = o.k;
  if (o.include_proposal !== undefined) args.include_proposal = o.include_proposal;
  if (o.path_prefix !== undefined) args.path_prefix = o.path_prefix;
  if (o.date_from !== undefined) args.date_from = o.date_from;
  if (o.date_to !== undefined) args.date_to = o.date_to;
  const r = await client.call("search", args);
  const items = r.hits.map((h: Hit) => {
    const first = h.loc_stale === false ? (h.loc ?? []).find((l): l is Loc => l !== null) : undefined;
    const line = first === undefined ? null : first.line_start - 1;
    return {
      doc_path: h.doc_path,
      layer: h.layer,
      score: h.score,
      line,
      label: `${h.doc_path}${line === null ? "" : `:${String(line + 1)}`} · ${h.layer} · ${h.score.toFixed(4)}`,
    };
  });
  return { items, undated_excluded: typeof r.undated_excluded === "number" ? r.undated_excluded : null };
}

/** A decorated block range of the editor (0-based lines, inclusive). */
export interface BlockDecoration {
  lineage_id: string;
  layer: string;
  start: number;
  end: number;
  hover: string;
}

export type BlameView =
  | { kind: "located"; blocks: BlockDecoration[] }
  /** the server says its locations are not the file's bytes */
  | { kind: "stale"; reason: string }
  /** the editor's text is not the bytes the locations belong to (unsaved,
   * or saved and not yet ingested) */
  | { kind: "changed"; reason: string }
  /** a v1 server (no `loc`) or a version with no recorded locations */
  | { kind: "unlocated"; reason: string }
  | { kind: "untracked"; reason: string };

function hover(b: BlameBlock): string {
  return [
    b.layer + (b.layer_source === "derived" ? " (derived from the log)" : ""),
    b.author_kind,
    b.source === "" ? "source: —" : `source: ${b.source}`,
    `created at seq ${String(b.seq_created)}, last event seq ${String(b.seq_last)}`,
    b.lineage_id.slice(0, 8),
  ].join(" · ");
}

/** The blame of the open note as editor decorations, only from the
 * server's locations and only when they are the editor's bytes
 * (`content_hash` of `text`); otherwise why not — never a guessed line. */
export async function blameView(client: RootmarkClient, doc: string, text: string): Promise<BlameView> {
  let r;
  try {
    r = await client.call("blame", { doc_path: doc });
  } catch (e) {
    if (e instanceof RootmarkError && e.code === "not_found") return { kind: "untracked", reason: `${doc} is not in the rootmark log` };
    throw e;
  }
  if (r.loc_stale === true) {
    return { kind: "stale", reason: `stale: ${doc} on disk is not the version rootmark ingested; no line is guessed` };
  }
  const blocks = r.blocks;
  if (blocks.length > 0 && blocks.some((b) => b.loc === undefined)) {
    return { kind: "unlocated", reason: `rootmark gave no block locations for ${doc} (a v1 server, or a version ingested before locations)` };
  }
  const bytes = new TextEncoder().encode(text);
  const hash = contentHash(bytes);
  if (blocks.some((b) => b.loc?.content_hash !== hash)) {
    return { kind: "changed", reason: `the editor's text of ${doc} is not the version rootmark located (unsaved, or not yet ingested)` };
  }
  // the decision rule of K-P2B-T1 (contract §2.5 "Deneysel"): a location is
  // used only when its bytes give the block's version
  if (!blocks.every((b) => b.loc !== undefined && sliceMatches(bytes, b.loc, b.version_id, hash32))) {
    return { kind: "unlocated", reason: `a block location of ${doc} fails the slice check; no line is guessed` };
  }
  return {
    kind: "located",
    blocks: blocks.map((b) => ({
      lineage_id: b.lineage_id,
      layer: b.layer,
      start: (b.loc as Loc).line_start - 1,
      end: (b.loc as Loc).line_end - 1,
      hover: hover(b),
    })),
  };
}

/** The block whose lines hold `line` (0-based), if located. */
export function blockAt(v: BlameView, line: number): BlockDecoration | null {
  if (v.kind !== "located") return null;
  return v.blocks.find((b) => b.start <= line && line <= b.end) ?? null;
}

/** The history of a lineage, one line per event. */
export async function historyLines(client: RootmarkClient, lineage: string): Promise<string[]> {
  const h: HistoryResult = await client.call("history", { lineage_id: lineage });
  return [
    `${lineage.slice(0, 8)} · now ${h.layer}`,
    ...h.events.map((e) => {
      const m = e.meta;
      const actor = m.actor === undefined ? "actor not recorded" : `${m.actor.kind}:${m.actor.id}`;
      const move = typeof m["from"] === "string" && typeof m["to"] === "string" ? ` · ${m["from"]} → ${m["to"]}` : "";
      return `#${String(e.seq)} ${e.kind} · ${actor}${move}`;
    }),
  ];
}

/** A human promotion (contract §2.4): the decision as a line. */
export async function promote(client: RootmarkClient, lineage: string, reason: string): Promise<string> {
  const r = await client.call("promote", { lineage_id: lineage, reason });
  return r.decision === "promoted" ? `promoted ${lineage.slice(0, 8)} → ${r.layer}` : `denied: ${r.reason}`;
}

/** A human demotion (contract §2.7, HTTP only). */
export async function demote(client: RootmarkClient, lineage: string, reason: string, to?: "proposal" | "unverified" | "derived"): Promise<string> {
  const r = await client.httpOnly("demote", to === undefined ? { lineage_id: lineage, reason } : { lineage_id: lineage, reason, to });
  return r.decision === "demoted" ? `demoted ${lineage.slice(0, 8)} → ${String(r.layer)}` : `denied: ${String(r.reason)}`;
}

export { describeError };
