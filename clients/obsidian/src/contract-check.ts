// The contract checker (P2-B/B4 (c); docs/CLIENT_CONTRACT.md §1, §2.9, §3):
// every HTTP call a client makes — in its tests and in its e2e runs — is
// recorded (`setRecorder`, called by `exchange`) and checked against the
// schema generated from the contract (`contract.gen.ts`): an unknown
// endpoint or tool, an unknown or missing argument, a wrong type, a missing
// header, a response without a field the contract promises or with a field
// of the wrong type, an error outside §3. No Obsidian or VS Code import:
// both clients use this file (one source, never a copy).

import {
  ARGUMENT_SCHEMAS,
  ERROR_STATUS,
  HTTP_HEADERS,
  HTTP_ONLY,
  REQUEST_SCHEMA,
  RESPONSE_SCHEMA,
  TOOLS,
} from "./contract.gen";

/** One recorded HTTP exchange (what went out, what came back). */
export interface CallRecord {
  method: string;
  url: string;
  headers: Record<string, string>;
  body?: string;
  status: number | null;
  response: string | null;
  /** a test's deliberate call outside the contract (`allowNextViolation`):
   * recorded and reported apart, never hidden */
  deliberate?: string;
}

export interface HttpRequestLike {
  method: string;
  url: string;
  headers: Record<string, string>;
  body?: string;
}

export interface HttpResponseLike {
  status: number;
  text: string;
}

type Recorder = (r: CallRecord) => void;
let recorder: Recorder | null = null;
let deliberate: string | null = null;

/** Marks the next recorded exchange as a test's deliberate call outside
 * the contract (a negative test of the client's error handling), with why. */
export function allowNextViolation(why: string): void {
  deliberate = why;
}

/** Sets (or clears) the process-wide recorder of HTTP exchanges. */
export function setRecorder(r: Recorder | null): void {
  recorder = r;
}

/** Sends a request through `transport`, recording the exchange when a
 * recorder is set (tests, e2e); a transport failure is recorded with no
 * status and rethrown. */
export async function exchange<Req extends HttpRequestLike, Res extends HttpResponseLike>(
  transport: (r: Req) => Promise<Res>,
  req: Req,
): Promise<Res> {
  const why = deliberate;
  deliberate = null;
  const mark = why === null ? {} : { deliberate: why };
  let res: Res;
  try {
    res = await transport(req);
  } catch (e) {
    recorder?.({ ...pick(req), status: null, response: null, ...mark });
    throw e;
  }
  recorder?.({ ...pick(req), status: res.status, response: res.text, ...mark });
  return res;
}

function pick(req: HttpRequestLike): Omit<CallRecord, "status" | "response"> {
  return {
    method: req.method,
    url: req.url,
    headers: { ...req.headers },
    ...(req.body === undefined ? {} : { body: req.body }),
  };
}

// ---- a small JSON Schema subset (the one §2.9 and `tools/list` use) ------------------

type Schema = Record<string, unknown>;
const isObj = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);

/** JSON Schema string length: code points (not UTF-16 units). */
function codePoints(v: string): number {
  let n = 0;
  for (let i = 0; i < v.length; i++) {
    const c = v.charCodeAt(i);
    // a high surrogate followed by a low one is one code point
    if (c >= 0xd800 && c <= 0xdbff && i + 1 < v.length) {
      const d = v.charCodeAt(i + 1);
      if (d >= 0xdc00 && d <= 0xdfff) i++;
    }
    n++;
  }
  return n;
}

function typeOk(t: string, v: unknown): boolean {
  switch (t) {
    case "string":
      return typeof v === "string";
    case "integer":
      return typeof v === "number" && Number.isInteger(v);
    case "number":
      return typeof v === "number";
    case "boolean":
      return typeof v === "boolean";
    case "null":
      return v === null;
    case "array":
      return Array.isArray(v);
    case "object":
      return isObj(v);
    default:
      return false;
  }
}

/** The violations of `v` against `s` (`defs` resolves `$ref`); `closed`:
 * an object property outside `properties` is a violation (request
 * arguments: `additionalProperties: false`), otherwise allowed (responses,
 * §2.5: other fields may come). */
export function violations(s: Schema, v: unknown, where: string, defs: Record<string, unknown>, closed: boolean): string[] {
  if (typeof s["$ref"] === "string") {
    const d = defs[s["$ref"]];
    if (!isObj(d)) return [`${where}: schema names an unknown $ref ${s["$ref"]}`];
    return violations(d, v, where, defs, closed);
  }
  if (Array.isArray(s["oneOf"])) {
    const ok = (s["oneOf"] as Schema[]).some((x) => violations(x, v, where, defs, closed).length === 0);
    return ok ? [] : [`${where}: matches none of its alternatives`];
  }
  if (Array.isArray(s["enum"])) {
    return (s["enum"] as unknown[]).includes(v) ? [] : [`${where}: ${JSON.stringify(v)} is not one of ${JSON.stringify(s["enum"])}`];
  }
  const out: string[] = [];
  if (s["type"] !== undefined) {
    const ts = Array.isArray(s["type"]) ? (s["type"] as string[]) : [s["type"] as string];
    if (!ts.some((t) => typeOk(t, v))) return [`${where}: ${(JSON.stringify(v) as string | undefined ?? "undefined").slice(0, 60)} is not ${ts.join(" | ")}`];
  }
  if (typeof v === "string") {
    const n = codePoints(v);
    if (typeof s["minLength"] === "number" && n < s["minLength"]) out.push(`${where}: shorter than ${String(s["minLength"])}`);
    if (typeof s["maxLength"] === "number" && n > s["maxLength"]) out.push(`${where}: longer than ${String(s["maxLength"])}`);
  }
  if (typeof v === "number") {
    if (typeof s["minimum"] === "number" && v < s["minimum"]) out.push(`${where}: below ${String(s["minimum"])}`);
    if (typeof s["maximum"] === "number" && v > s["maximum"]) out.push(`${where}: above ${String(s["maximum"])}`);
  }
  if (Array.isArray(v)) {
    if (typeof s["minItems"] === "number" && v.length < s["minItems"]) out.push(`${where}: fewer than ${String(s["minItems"])} items`);
    if (typeof s["maxItems"] === "number" && v.length > s["maxItems"]) out.push(`${where}: more than ${String(s["maxItems"])} items`);
    if (isObj(s["items"])) v.forEach((x, i) => out.push(...violations(s["items"] as Schema, x, `${where}[${String(i)}]`, defs, closed)));
  }
  if (isObj(v) && (isObj(s["properties"]) || Array.isArray(s["required"]))) {
    const props = isObj(s["properties"]) ? s["properties"] : {};
    for (const r of (s["required"] as string[] | undefined) ?? []) {
      if (!(r in v)) out.push(`${where}: missing field ${r}`);
    }
    for (const [k, x] of Object.entries(v)) {
      const p = props[k];
      if (isObj(p)) out.push(...violations(p, x, `${where}.${k}`, defs, closed));
      else if (closed || s["additionalProperties"] === false) out.push(`${where}: unknown field ${k}`);
    }
  }
  return out;
}

// ---- one exchange --------------------------------------------------------------------

const TOOL_SET = new Set<string>(TOOLS);
const HTTP_ONLY_SET = new Set<string>(HTTP_ONLY);
const RESPONSES = RESPONSE_SCHEMA.responses as Record<string, Schema>;
const DEFS = RESPONSE_SCHEMA.defs as Record<string, unknown>;
const ARGS = ARGUMENT_SCHEMAS as Record<string, Schema>;
const REQUESTS = REQUEST_SCHEMA as Record<string, Schema>;

function header(h: Record<string, string>, name: string): string | undefined {
  const k = Object.keys(h).find((x) => x.toLowerCase() === name.toLowerCase());
  return k === undefined ? undefined : h[k];
}

/** The contract violations of one recorded exchange: the client's request
 * (endpoint, method, headers, arguments) and the answer it got. A client's
 * call outside the contract is a request violation; a response violation
 * is the server's (or a test's fake server answering outside the contract
 * on purpose, to test the client's refusal). */
export interface Violations {
  request: string[];
  response: string[];
}

export function checkCall(r: CallRecord): Violations {
  const request: string[] = [];
  const response: string[] = [];
  let path: string;
  try {
    const u = new URL(r.url);
    path = u.pathname;
    if (u.hostname !== "127.0.0.1") request.push(`${r.url}: not 127.0.0.1 (§1)`);
    if (u.search !== "") request.push(`${r.url}: a query string (§1: the body carries the arguments)`);
  } catch {
    return { request: [`${r.url}: not a URL`], response };
  }
  const m = /^\/v1\/([a-z_]+)$/.exec(path);
  const name = m?.[1];
  if (name === undefined || (!TOOL_SET.has(name) && !HTTP_ONLY_SET.has(name))) {
    return { request: [`${r.method} ${path}: not an endpoint of the contract (§1, §2)`], response };
  }
  const where = `${r.method} /v1/${name}`;
  if (!(r.method === "POST" || (r.method === "GET" && name === "status"))) request.push(`${where}: method (POST; GET only for status)`);
  for (const h of HTTP_HEADERS) {
    const v = header(r.headers, h);
    if (v === undefined || v.trim() === "") request.push(`${where}: missing header ${h} (§1)`);
  }
  const auth = header(r.headers, "Authorization");
  if (auth !== undefined && !/^Bearer [0-9a-f]{64}$/.test(auth)) request.push(`${where}: Authorization is not "Bearer <64 hex>" (§1)`);
  // the arguments: a tool's from `tools/list`, an HTTP-only endpoint's from §2.9
  let args: unknown = {};
  if (r.body !== undefined && r.body !== "") {
    try {
      args = JSON.parse(r.body);
    } catch {
      request.push(`${where}: the body is not JSON`);
    }
  }
  if (r.method === "POST") {
    const schema = TOOL_SET.has(name) ? ARGS[name] : REQUESTS[name];
    if (schema === undefined) request.push(`${where}: no argument schema in the contract`);
    else request.push(...violations(schema, args, `${where} arguments`, DEFS, true));
  }
  // the answer (none: the client reports `unreachable`)
  if (r.status === null || r.response === null) return { request, response };
  let body: unknown;
  try {
    body = JSON.parse(r.response);
  } catch {
    response.push(`${where}: HTTP ${String(r.status)} body is not JSON`);
    return { request, response };
  }
  if (r.status === 200) {
    const s = RESPONSES[name];
    if (s === undefined) response.push(`${where}: no response schema in the contract`);
    else response.push(...violations(s, body, `${where} response`, DEFS, false));
  } else {
    const e = RESPONSES["error"];
    if (e !== undefined) response.push(...violations(e, body, `${where} error`, DEFS, false));
    const code = isObj(body) && isObj(body["error"]) ? body["error"]["code"] : undefined;
    const want = typeof code === "string" ? (ERROR_STATUS as Record<string, number>)[code] : undefined;
    // §3: 405 (wrong method) is `invalid_arguments`; 401 is `denied`
    const statusOk = want === r.status || (code === "invalid_arguments" && r.status === 405) || (code === "denied" && r.status === 401);
    if (want === undefined) response.push(`${where}: error code ${JSON.stringify(code)} is not in §3`);
    else if (!statusOk) response.push(`${where}: error ${String(code)} with HTTP ${String(r.status)} (§3: ${String(want)})`);
  }
  return { request, response };
}

/** A run's record: every exchange and its violations. */
export interface ContractReport {
  calls: number;
  /** calls by endpoint (`POST /v1/<name>`) */
  endpoints: Record<string, number>;
  request_violations: string[];
  response_violations: string[];
  /** violations of the calls a test marked deliberate, with why */
  deliberate_violations: string[];
}

export function report(records: readonly CallRecord[]): ContractReport {
  const endpoints: Record<string, number> = {};
  const req: string[] = [];
  const res: string[] = [];
  const del: string[] = [];
  records.forEach((r, i) => {
    let p = r.url;
    try {
      p = new URL(r.url).pathname;
    } catch {
      // kept as written; checkCall reports it
    }
    const k = `${r.method} ${p}`;
    endpoints[k] = (endpoints[k] ?? 0) + 1;
    const v = checkCall(r);
    if (r.deliberate !== undefined) {
      del.push(...[...v.request, ...v.response].map((x) => `#${String(i)} ${x} (deliberate: ${r.deliberate ?? ""})`));
      return;
    }
    req.push(...v.request.map((x) => `#${String(i)} ${x}`));
    res.push(...v.response.map((x) => `#${String(i)} ${x}`));
  });
  return { calls: records.length, endpoints, request_violations: req, response_violations: res, deliberate_violations: del };
}
