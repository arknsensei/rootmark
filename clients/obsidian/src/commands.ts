// The plugin's command layer (TASKS E2–E4): what every panel does, as plain
// functions over the contract client. No Obsidian import: the panels call
// these, and the e2e harness (clients/obsidian/e2e/) calls the same
// functions against a real `mdk serve` when Obsidian cannot run headless.
// Nothing here writes a vault file: the only writes are the contract's
// HTTP tools (`promote`, `checkpoint`; the plugin never proposes).

import { MdkError, type BlameBlock, type HistoryResult, type MdkClient, type PromoteResult } from "./api";

/** A proposal block of the vault (proposal view). */
export interface ProposalRow {
  doc_path: string;
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
export async function blameOf(client: MdkClient, doc: string, at?: string): Promise<BlameBlock[] | null> {
  try {
    const r = await client.call("blame", at === undefined ? { doc_path: doc } : { doc_path: doc, at });
    return r.blocks;
  } catch (e) {
    if (e instanceof MdkError && e.code === "not_found") return null;
    throw e;
  }
}

export async function historyOf(client: MdkClient, lineage: string): Promise<HistoryResult> {
  return client.call("history", { lineage_id: lineage });
}

/** The proposal blocks of `docs` (vault order, then block order). A
 * document the log does not know is skipped; `section(doc, i)` names the
 * section of its i-th block when known. */
export async function listProposals(
  client: MdkClient,
  docs: readonly string[],
  section: (doc: string, index: number) => string | null = () => null,
): Promise<ProposalRow[]> {
  const rows: ProposalRow[] = [];
  for (const doc of [...docs].sort()) {
    const blocks = await blameOf(client, doc);
    if (blocks === null) continue;
    blocks.forEach((b, i) => {
      if (b.layer !== "proposal" || b.inactive) return;
      rows.push({
        doc_path: doc,
        lineage_id: b.lineage_id,
        version_id: b.version_id,
        source: b.source,
        author_kind: b.author_kind,
        seq_created: b.seq_created,
        seq_last: b.seq_last,
        section: section(doc, i),
      });
    });
  }
  return rows;
}

export interface PromoteOutcome {
  lineage_id: string;
  result: PromoteResult | null;
  error: MdkError | null;
}

/** Promote over HTTP — the human channel (K-D1.3; contract §2.4). The
 * reason is the human's; the UI does not let an empty one through, but
 * the server is the authority (an empty reason is its 400
 * `invalid_arguments`, reported, never retried). A policy refusal is an
 * answer (`decision: "denied"`). One outcome per lineage, in order. */
export async function promoteAll(
  client: MdkClient,
  lineages: readonly string[],
  reason: string,
): Promise<PromoteOutcome[]> {
  const out: PromoteOutcome[] = [];
  for (const l of lineages) {
    try {
      out.push({ lineage_id: l, result: await client.call("promote", { lineage_id: l, reason }), error: null });
    } catch (e) {
      if (!(e instanceof MdkError)) throw e;
      out.push({ lineage_id: l, result: null, error: e });
    }
  }
  return out;
}

/** Demote is in the GOAL's proposal view but not in the contract
 * (docs/CLIENT_CONTRACT.md §2 has no demote tool); the client relies on
 * nothing the contract does not say, so it is refused here, visibly
 * (decision request K-E2-T; ASSUMPTIONS E2-1). */
export const DEMOTE_UNAVAILABLE =
  "Demote is not offered by mdk serve's contract (docs/CLIENT_CONTRACT.md §2); decision request K-E2-T";

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
