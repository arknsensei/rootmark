// The client's only way to the kernel: the HTTP contract of `mdk serve`
// (docs/CLIENT_CONTRACT.md §1–§3). Argument types and error codes are
// generated from the contract (contract.gen.ts); the result types below
// name the main fields of §2 and the inner fields of §2.5 — the ones the
// client relies on, each checked on every answer (`shape.ts`). No Obsidian
// import here: the same client runs in the plugin (transport: Obsidian
// `requestUrl`) and in Node (e2e harness: `fetch`).

import {
  ERROR_STATUS,
  TOOLS,
  type ErrorCode,
  type ToolArgs,
  type ToolName,
} from "./contract.gen";
import { lastCheckpointViolation, resultViolation } from "./shape";

export { TOOLS, ERROR_STATUS };
export type { ErrorCode, ToolArgs, ToolName };

export type Layer = "canonical" | "proposal" | "derived";
/** §2.5: a string (`Human`, `Agent`, `Transcript`, `Synthetic`). */
export type AuthorKind = string;
export interface Actor {
  /** `human`, `agent` or `policy` (§4.2) */
  kind: string;
  id: string;
}

export interface Checkpoint {
  checkpoint_id: string;
  docs: Record<string, string>;
  event_seq: number;
  name: string | null;
}

export interface StatusResult {
  vault: string;
  spec_version: string;
  mdk_version: string;
  level_claimed: number;
  embedder: string | null;
  index: string;
  last_checkpoint: Checkpoint | null;
  policy: { promotion: string; edit: string } & Record<string, unknown>;
  server_policy: {
    server: Record<string, unknown>;
    index: Record<string, unknown>;
    identity: { human_id: string | null };
  };
}

export interface Hit {
  doc_path: string;
  chunk_id: string;
  lineage_ids: string[];
  layer: Layer;
  score: number;
}

export interface SearchResult {
  checkpoint_id: string;
  mode: "hybrid" | "lexical" | "flat";
  include_proposal: boolean;
  query_hash: string;
  trace_id: string;
  hits: Hit[];
}

export interface Returned {
  doc_path: string;
  lineage_id: string;
  version_id: string;
}

export interface Trace {
  trace_id: string;
  checkpoint_id: string;
  consumer: string;
  query_hash: string;
  returned: Returned[];
  at_seq: number;
}

export interface Block {
  version_id: string;
  lineage_id: string;
  block_type: string;
  text: string;
  heading_path: string[];
  parents: string[];
  provenance: { author_kind: AuthorKind; source: string };
  layer: Layer;
}

export interface ReadBlockResult {
  checkpoint_id: string;
  block: { doc_path: string; block: Block; inactive: boolean };
  trace: Trace;
}

export interface LogEvent {
  seq: number;
  kind: string;
  doc_path: string;
  lineage_ids: string[];
  /** `actor {kind, id}` and `source` on change/delete events written
   * through the server (spec 1.0.3); absent → "actor not recorded" (§4.2) */
  meta: { actor?: Actor; source?: string; reason?: string } & Record<string, unknown>;
}

export interface HistoryResult {
  lineage_id: string;
  layer: Layer;
  events: LogEvent[];
}

export interface BlameBlock {
  lineage_id: string;
  version_id: string;
  layer: Layer;
  layer_source: string;
  author_kind: AuthorKind;
  source: string;
  seq_created: number;
  seq_last: number;
  inactive: boolean;
}

export interface BlameResult {
  doc_path: string;
  at: string | null;
  blocks: BlameBlock[];
}

export interface ProposeResult {
  doc_path: string;
  mode: string;
  bytes: number;
  synced_external_events: number;
  edit_policy: string;
  actor: Actor;
  snapshot_id: string;
  events: number;
  chunks_embedded: number;
  blocks: BlameBlock[];
  events_hash: string;
}

/** A policy refusal is an answer, not an error (§2.4). */
export type PromoteResult =
  | {
      decision: "promoted";
      lineage_id: string;
      layer: Layer;
      event: LogEvent;
      policy: string;
      policy_version: string;
    }
  | {
      decision: "denied";
      lineage_id: string;
      layer: Layer;
      reason: string;
      actor: Actor;
      policy: string;
      policy_version: string;
    };

export interface CheckpointsResult {
  checkpoints: Checkpoint[];
}

export interface TraceResult {
  traces: Trace[];
}

/** Results by tool. */
export interface ToolResult {
  status: StatusResult;
  search: SearchResult;
  read_block: ReadBlockResult;
  history: HistoryResult;
  blame: BlameResult;
  propose: ProposeResult;
  promote: PromoteResult;
  checkpoint: Checkpoint;
  checkpoints: CheckpointsResult;
  trace: TraceResult;
}

export interface HttpRequest {
  method: "GET" | "POST";
  url: string;
  headers: Record<string, string>;
  body?: string;
}

export interface HttpResponse {
  status: number;
  text: string;
}

/** Sends one request; rejects only when no HTTP answer arrived. */
export type Transport = (req: HttpRequest) => Promise<HttpResponse>;

/** Where `mdk serve` listens and who the client is (§1). */
export interface Connection {
  /** `http://127.0.0.1:<port>` */
  baseUrl: string;
  token: string;
  /** `X-Consumer`: the trace label (never the actor; §1) */
  consumer: string;
}

/** Codes of the contract (§3), plus three of the client's own: the vault
 * lacks the connection settings (`not_configured`), the server did not
 * answer (`unreachable`) or answered outside the contract (`bad_response`).
 * None is ever swallowed. */
export type ClientErrorCode = ErrorCode | "not_configured" | "unreachable" | "bad_response";

export class MdkError extends Error {
  constructor(
    readonly code: ClientErrorCode,
    message: string,
    readonly status: number | null,
    /** what the caller can do (`{"error": {…, "hint"}}`, §3) */
    readonly hint: string | null = null,
  ) {
    super(message);
    this.name = "MdkError";
  }
}

const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

const isCode = (c: unknown): c is ErrorCode =>
  typeof c === "string" && Object.prototype.hasOwnProperty.call(ERROR_STATUS, c);

/** The contract's answer → result or `MdkError` (§3: decide on `code`);
 * a result is checked against §2/§2.5 (`shape.ts`). */
export function decode(tool: ToolName, res: HttpResponse): unknown {
  let body: unknown;
  try {
    body = JSON.parse(res.text);
  } catch {
    throw new MdkError("bad_response", `${tool}: HTTP ${String(res.status)}, body is not JSON`, res.status);
  }
  if (res.status === 200) {
    if (!isRecord(body) || "error" in body) {
      throw new MdkError("bad_response", `${tool}: HTTP 200 without a result object`, 200);
    }
    const bad = resultViolation(tool, body) ?? (tool === "status" ? lastCheckpointViolation(body) : null);
    if (bad !== null) {
      throw new MdkError("bad_response", `answer outside the contract (§2.5): ${bad}`, 200);
    }
    return body;
  }
  const err = isRecord(body) && isRecord(body["error"]) ? body["error"] : null;
  const code = err?.["code"];
  if (err === null || !isCode(code)) {
    throw new MdkError("bad_response", `${tool}: HTTP ${String(res.status)} without a contract error code`, res.status);
  }
  const message = typeof err["message"] === "string" ? err["message"] : code;
  const hint = typeof err["hint"] === "string" ? err["hint"] : null;
  throw new MdkError(code, message, res.status, hint);
}

type IsUnion<T, U = T> = T extends unknown ? ([U] extends [T] ? false : true) : never;
/** The arguments of exactly one tool: for a union of tools (a tool name not
 * known at compile time, or an explicit union) nothing is assignable. */
export type ArgsOf<T extends ToolName> = true extends IsUnion<T> ? never : NoInfer<ToolArgs[T]>;

export class MdkClient {
  constructor(
    readonly conn: Connection,
    private readonly transport: Transport,
  ) {}

  private async send(tool: ToolName, method: "GET" | "POST", body?: string): Promise<unknown> {
    const req: HttpRequest = {
      method,
      url: `${this.conn.baseUrl}/v1/${tool}`,
      headers: {
        Authorization: `Bearer ${this.conn.token}`,
        "X-Consumer": this.conn.consumer,
        ...(body === undefined ? {} : { "Content-Type": "application/json" }),
      },
      ...(body === undefined ? {} : { body }),
    };
    let res: HttpResponse;
    try {
      res = await this.transport(req);
    } catch (e) {
      throw new MdkError("unreachable", `${tool}: mdk serve did not answer (${String(e)})`, null);
    }
    return decode(tool, res);
  }

  /** `POST /v1/<tool>` with the tool's arguments (§1). A tool outside the
   * contract, or arguments that are not that tool's, do not compile; a
   * tool name that is a union (not one known tool) takes no arguments at
   * all (`ArgsOf`). */
  async call<T extends ToolName>(tool: T, args: ArgsOf<T>): Promise<ToolResult[T]> {
    return (await this.send(tool, "POST", JSON.stringify(args))) as ToolResult[T];
  }

  /** `GET /v1/status` (§1). */
  async status(): Promise<StatusResult> {
    return (await this.send("status", "GET")) as StatusResult;
  }
}

/** Read-only access to the vault's files (never a write: the plugin
 * writes nothing to the vault; GOAL P1-E). */
export interface VaultReader {
  read(path: string): Promise<string>;
  exists(path: string): Promise<boolean>;
}

export const TOKEN_PATH = ".mdkernel/server.token";
export const POLICY_PATH = ".mdkernel/policy.toml";

/** A TOML value as written: a basic ("…", with \\ and \" escapes) or
 * literal ('…') string, or a bare value up to a comment. */
function tomlScalar(v: string): string | null {
  if (v.startsWith('"')) {
    const m = /^"((?:[^"\\]|\\.)*)"\s*(#.*)?$/.exec(v);
    return m?.[1] === undefined ? null : m[1].replace(/\\(.)/g, "$1");
  }
  if (v.startsWith("'")) {
    const m = /^'([^']*)'\s*(#.*)?$/.exec(v);
    return m?.[1] ?? null;
  }
  return v.replace(/\s*#.*$/, "").trim();
}

/** `[table] key = <value>` of a TOML text, or null. Only the two keys the
 * client reads (`[server] http_port`, `[identity] human_id`); array tables
 * (`[[…]]`) are never one of them. A value it cannot read is null, and the
 * caller says what is missing. */
export function tomlValue(text: string, table: string, key: string): string | null {
  let current: string | null = "";
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line.startsWith("#") || line === "") continue;
    if (line.startsWith("[[")) {
      current = null;
      continue;
    }
    const t = /^\[([^\]]+)\]\s*(#.*)?$/.exec(line);
    if (t?.[1] !== undefined) {
      current = t[1].trim();
      continue;
    }
    const kv = /^([A-Za-z0-9_]+)\s*=\s*(.+)$/.exec(line);
    if (current === table && kv?.[1] === key && kv[2] !== undefined) {
      return tomlScalar(kv[2].trim());
    }
  }
  return null;
}

export interface Settings {
  /** `[server] http_port` (the client never picks or starts a server) */
  port: number;
  token: string;
  /** `[identity] human_id`: shown only (edited in policy.toml) */
  humanId: string | null;
}

/** The connection settings from the vault (§1): token file and
 * `[server] http_port`. Missing → an error saying what is missing. */
export async function readSettings(vault: VaultReader): Promise<Settings> {
  if (!(await vault.exists(POLICY_PATH))) {
    throw new MdkError("not_configured", `${POLICY_PATH} not found: this vault is not served by mdk`, null);
  }
  const policy = await vault.read(POLICY_PATH);
  const portText = tomlValue(policy, "server", "http_port");
  const port = portText === null ? NaN : Number(portText);
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new MdkError(
      "not_configured",
      `${POLICY_PATH} has no [server] http_port: set it and start \`mdk serve --vault <vault> --http 127.0.0.1\``,
      null,
    );
  }
  if (!(await vault.exists(TOKEN_PATH))) {
    throw new MdkError("not_configured", `${TOKEN_PATH} not found: start \`mdk serve --http\` once`, null);
  }
  const token = (await vault.read(TOKEN_PATH)).trim();
  if (!/^[0-9a-f]{64}$/.test(token)) {
    throw new MdkError("not_configured", `${TOKEN_PATH} is not a 64-hex token`, null);
  }
  return { port, token, humanId: tomlValue(policy, "identity", "human_id") };
}

export type ConnectionState =
  | { kind: "online"; client: MdkClient; status: StatusResult; settings: Settings }
  /** passive: no server (or no settings); the plugin reports why and never
   * starts one (GOAL P1-E "mdk serve yoksa plugin pasif") */
  | { kind: "offline"; reason: string };

/** Connects if `mdk serve` answers `GET /v1/status`; otherwise passive. */
export async function connect(
  vault: VaultReader,
  transport: Transport,
  consumer: string,
): Promise<ConnectionState> {
  let settings: Settings;
  try {
    settings = await readSettings(vault);
  } catch (e) {
    return { kind: "offline", reason: describeError(e) };
  }
  const client = new MdkClient(
    { baseUrl: `http://127.0.0.1:${String(settings.port)}`, token: settings.token, consumer },
    transport,
  );
  try {
    const status = await client.status();
    return { kind: "online", client, status, settings };
  } catch (e) {
    return { kind: "offline", reason: describeError(e) };
  }
}

/** A human-readable line for an error, with its hint (§3: the client shows
 * the hint to the user). */
export function describeError(e: unknown): string {
  if (e instanceof MdkError) {
    return e.hint === null ? `${e.code}: ${e.message}` : `${e.code}: ${e.message} — ${e.hint}`;
  }
  return e instanceof Error ? e.message : String(e);
}
