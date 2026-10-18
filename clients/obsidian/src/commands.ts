// The plugin's command layer (TASKS E2–E4): what every panel does, as plain
// functions over the contract client. No Obsidian import: the panels call
// these, and the e2e harness (clients/obsidian/e2e/) calls the same
// functions against a real `rootmark serve` when Obsidian cannot run headless.
// Nothing here writes a vault file: the only writes are the contract's
// HTTP tools (`promote`, `checkpoint`; the plugin never proposes).

import {
  RootmarkError,
  type BlameBlock,
  type Checkpoint,
  type HistoryResult,
  type Hit,
  type RootmarkClient,
  type PromoteResult,
  type Returned,
} from "./api";

/** A block awaiting promotion (proposal view): a `proposal`, or (spec
 * 2.0.0) an `unverified` external edit. */
export interface ProposalRow {
  doc_path: string;
  /** `proposal` or `unverified` */
  layer: string;
  lineage_id: string;
  version_id: string;
  /** who wrote it: the consumer label of the agent (`source`) */
  source: string;
  author_kind: string;
  /** log offsets (the kernel keeps no clock: spec §4, no time stamps) */
  seq_created: number;
  seq_last: number;
  /** heading path of the block when the editor's block map aligns */
  section: string | null;
}

/** `blame` of one document; `null` when the log does not know it (an
 * untracked file is not an error for the panel; every other error is). */
export async function blameOf(client: RootmarkClient, doc: string, at?: string): Promise<BlameBlock[] | null> {
  try {
    const r = await client.call("blame", at === undefined ? { doc_path: doc } : { doc_path: doc, at });
    return r.blocks;
  } catch (e) {
    if (e instanceof RootmarkError && e.code === "not_found") return null;
    throw e;
  }
}

export async function historyOf(client: RootmarkClient, lineage: string): Promise<HistoryResult> {
  return client.call("history", { lineage_id: lineage });
}

/** The proposal blocks of `docs` (vault order, then block order). A
 * document the log does not know is skipped. `sections(doc, blocks)` gives
 * the section of each block only when the editor's block map aligns with
 * them (`align`), else null: then every section is unknown ("—"), never a
 * guess by position. */
export async function listProposals(
  client: RootmarkClient,
  docs: readonly string[],
  sections: (doc: string, blocks: readonly BlameBlock[]) => readonly (string | null)[] | null = () => null,
  layers: readonly string[] = ["proposal"],
): Promise<ProposalRow[]> {
  const rows: ProposalRow[] = [];
  for (const doc of [...docs].sort()) {
    const blocks = await blameOf(client, doc);
    if (blocks === null) continue;
    const names = sections(doc, blocks);
    blocks.forEach((b, i) => {
      if (!layers.includes(b.layer) || b.inactive) return;
      rows.push({
        doc_path: doc,
        layer: b.layer,
        lineage_id: b.lineage_id,
        version_id: b.version_id,
        source: b.source,
        author_kind: b.author_kind,
        seq_created: b.seq_created,
        seq_last: b.seq_last,
        section: names?.[i] ?? null,
      });
    });
  }
  return rows;
}

export interface PromoteOutcome {
  lineage_id: string;
  result: PromoteResult | null;
  error: RootmarkError | null;
}

/** Promote over HTTP — the human channel (K-D1.3; contract §2.4). The
 * reason is the human's; the UI does not let an empty one through, but
 * the server is the authority (an empty reason is its 400
 * `invalid_arguments`, reported, never retried). A policy refusal is an
 * answer (`decision: "denied"`). One outcome per lineage, in order. */
export async function promoteAll(
  client: RootmarkClient,
  lineages: readonly string[],
  reason: string,
): Promise<PromoteOutcome[]> {
  const out: PromoteOutcome[] = [];
  for (const l of lineages) {
    try {
      out.push({ lineage_id: l, result: await client.call("promote", { lineage_id: l, reason }), error: null });
    } catch (e) {
      if (!(e instanceof RootmarkError)) throw e;
      out.push({ lineage_id: l, result: null, error: e });
    }
  }
  return out;
}

/** Demote is in the GOAL's proposal view but not in the HTTP/MCP contract
 * (docs/CLIENT_CONTRACT.md §2 has no demote tool; only the CLI has `rootmark
 * demote`). The client relies on nothing the contract does not say, so it
 * is refused here, visibly, and the user is pointed to the CLI (decision
 * request K-E2-T; ASSUMPTIONS E2-1). */
export const DEMOTE_UNAVAILABLE =
  "Demote is not in rootmark serve's contract (docs/CLIENT_CONTRACT.md §2; decision request K-E2-T). " +
  "From a terminal: rootmark demote <vault> <lineage> --actor human:<id> --reason <why>";

export function demote(): never {
  throw new Error(DEMOTE_UNAVAILABLE);
}

/** The one-line notice of promote outcomes (UI and harness). */
export function promoteSummary(outcomes: readonly PromoteOutcome[]): string {
  const promoted = outcomes.filter((o) => o.result?.decision === "promoted").length;
  const denied = outcomes.filter((o) => o.result?.decision === "denied");
  const failed = outcomes.filter((o) => o.error !== null);
  const parts = [`${String(promoted)} promoted`];
  if (denied.length > 0) {
    const d = denied[0]?.result;
    parts.push(`${String(denied.length)} denied${d?.decision === "denied" ? ` (${d.reason})` : ""}`);
  }
  if (failed.length > 0) {
    const e = failed[0]?.error;
    parts.push(
      `${String(failed.length)} failed${e ? ` (${e.code}: ${e.message}${e.hint === null ? "" : ` — ${e.hint}`})` : ""}`,
    );
  }
  return parts.join(", ");
}

/** The actor line of a log event (contract §4.2): an event without
 * `meta.actor` says so; the client never invents one. */
export function actorOf(meta: Record<string, unknown>): string {
  const a = meta["actor"];
  if (typeof a === "object" && a !== null && "kind" in a && "id" in a) {
    const { kind, id } = a;
    if (typeof kind === "string" && typeof id === "string") return `${kind}:${id}`;
  }
  return "actor not recorded";
}

// ---- E3: search, checkpoints, traces ---------------------------------------

export interface SearchView {
  checkpoint_id: string;
  trace_id: string;
  /** canonical hits, in the server's order (never re-ranked here) */
  canonical: Hit[];
  /** proposal hits (only with include_proposal), after the canonical ones
   * on the server and shown apart (contract §2.2) */
  proposal: Hit[];
}

/** `search` as the panel shows it: the server's order kept, the layers
 * split for display. A hit of another layer is `bad_response` (derived is
 * never returned, §2.2). */
export async function runSearch(
  client: RootmarkClient,
  query: string,
  opts: { mode?: "hybrid" | "lexical" | "flat"; k?: number; include_proposal?: boolean; at?: string } = {},
): Promise<SearchView> {
  const r = await client.call("search", { query, ...opts });
  const k = opts.k ?? 10;
  const asked = opts.include_proposal === true;
  // spec 2.0.0 §9: `unverified` hits come with the proposals (same pass)
  const bad = r.hits.find((h) => h.layer !== "canonical" && h.layer !== "proposal" && h.layer !== "unverified");
  if (bad !== undefined) throw new RootmarkError("bad_response", `search returned a ${bad.layer} hit (contract §2.2)`, 200);
  const firstProposal = r.hits.findIndex((h) => h.layer !== "canonical");
  if (firstProposal >= 0 && r.hits.slice(firstProposal).some((h) => h.layer === "canonical")) {
    throw new RootmarkError("bad_response", "search: a canonical hit after a proposal hit (contract §2.2)", 200);
  }
  const canonical = r.hits.filter((h) => h.layer === "canonical");
  const proposal = r.hits.filter((h) => h.layer !== "canonical");
  // a proposal is never shown unasked, and never hidden: an answer that
  // holds one without the flag is outside the contract (§2.2)
  if ((!asked || !r.include_proposal) && proposal.length > 0) {
    throw new RootmarkError("bad_response", "search: proposal hits without include_proposal (contract §2.2)", 200);
  }
  if (canonical.length > k || proposal.length > k) {
    throw new RootmarkError(
      "bad_response",
      `search: ${String(canonical.length)} canonical / ${String(proposal.length)} proposal hits for k = ${String(k)} (contract §2.2: at most k per layer)`,
      200,
    );
  }
  return { checkpoint_id: r.checkpoint_id, trace_id: r.trace_id, canonical, proposal };
}

/** Checkpoints in creation order (§2.3). */
export async function listCheckpoints(client: RootmarkClient): Promise<Checkpoint[]> {
  return (await client.call("checkpoints", {})).checkpoints;
}

/** Names the current state's checkpoint (or creates it named; §2.3). */
export async function nameCheckpoint(client: RootmarkClient, name: string): Promise<Checkpoint> {
  return client.call("checkpoint", { name });
}

/** The read-only past view of a note: its blame at a checkpoint; `null`
 * when the note had no snapshot then. */
export async function blameAt(client: RootmarkClient, doc: string, at: string): Promise<BlameBlock[] | null> {
  return blameOf(client, doc, at);
}

export interface TraceRow {
  trace_id: string;
  at_seq: number;
  consumer: string;
  checkpoint_id: string;
  /** what the answer rested on: one jump target per returned block */
  returned: Returned[];
}

/** The latest `limit` traces (newest first), optionally of one consumer. */
export async function recentTraces(client: RootmarkClient, opts: { consumer?: string; limit?: number } = {}): Promise<TraceRow[]> {
  const r = await client.call("trace", opts.consumer === undefined ? {} : { consumer: opts.consumer });
  const rows = r.traces.map((t) => ({
    trace_id: t.trace_id,
    at_seq: t.at_seq,
    consumer: t.consumer,
    checkpoint_id: t.checkpoint_id,
    returned: t.returned,
  }));
  rows.sort((a, b) => b.at_seq - a.at_seq);
  return rows.slice(0, opts.limit ?? 50);
}
