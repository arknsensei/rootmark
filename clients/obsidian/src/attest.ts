// Attested writes, the client's side (spec §7.1; docs/CLIENT_CONTRACT.md
// §2.6; P2-0/Z3). After every save the plugin tells `rootmark serve` the
// BLAKE3 of the bytes it saved, signed with a session key (stage 1: the
// key comes from `POST /v1/attest_session` over the token). Only the two
// contract endpoints below are used; nothing here writes a vault file.

import { decodeHttp, RootmarkError, type Connection, type HttpRequest, type Transport } from "./api";
import { contentHash } from "./blake3";
import type { SavedHow } from "./own-writes";

/** `POST /v1/attest_session` (§2.6). */
export interface AttestSession {
  session_id: string;
  /** 64 hex: the HMAC-SHA256 key of this session */
  key: string;
  ttl_s: number;
  /** the key source: `session` (stage 1) or `keychain` (stage 2) */
  stage: number;
}

/** `POST /v1/attest` (§2.6). */
export interface AttestResult {
  attestation_id: string;
  doc_path: string;
  content_hash: string;
  at_seq: number;
  /** false: the same attestation (doc, hash, at_seq) was already recorded */
  recorded: boolean;
  /** lineages a late attestation raised to `canonical` (spec §7.1) */
  raised: string[];
}

export const ATTEST_ENDPOINTS = ["attest_session", "attest"] as const;
export type AttestEndpoint = (typeof ATTEST_ENDPOINTS)[number];

/** The MAC message of one attestation (§2.6). */
export function macMessage(docPath: string, hash: string, savedAtSeq: number | null): string {
  return `rootmark-attest/1\n${docPath}\n${hash}\n${savedAtSeq === null ? "" : String(savedAtSeq)}`;
}

function fromHex(h: string): Uint8Array<ArrayBuffer> {
  const out = new Uint8Array(new ArrayBuffer(h.length / 2));
  for (let i = 0; i < out.length; i++) out[i] = parseInt(h.slice(2 * i, 2 * i + 2), 16);
  return out;
}

function toHex(b: ArrayBuffer): string {
  return Array.from(new Uint8Array(b), (x) => x.toString(16).padStart(2, "0")).join("");
}

/** `HMAC-SHA256(key, message)` as 64 lowercase hex (WebCrypto). */
export async function hmacHex(subtle: SubtleCrypto, keyHex: string, message: string): Promise<string> {
  const key = await subtle.importKey("raw", fromHex(keyHex), { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
  return toHex(await subtle.sign("HMAC", key, new TextEncoder().encode(message)));
}

const isStr = (v: unknown): v is string => typeof v === "string";
const isNum = (v: unknown): v is number => typeof v === "number" && Number.isInteger(v);

/** A session answer checked against §2.6 (`bad_response` otherwise). */
function checkSession(v: Record<string, unknown>): AttestSession {
  if (!isStr(v["session_id"]) || !isStr(v["key"]) || !/^[0-9a-f]{64}$/.test(v["key"]) || !isNum(v["ttl_s"]) || !isNum(v["stage"])) {
    throw new RootmarkError("bad_response", "attest_session: answer outside the contract (§2.6)", 200);
  }
  return v as unknown as AttestSession;
}

/** An attestation answer checked against §2.6. */
function checkResult(v: Record<string, unknown>): AttestResult {
  const raised = v["raised"];
  if (
    !isStr(v["attestation_id"]) ||
    !isStr(v["doc_path"]) ||
    !isStr(v["content_hash"]) ||
    !isNum(v["at_seq"]) ||
    typeof v["recorded"] !== "boolean" ||
    !Array.isArray(raised) ||
    !raised.every(isStr)
  ) {
    throw new RootmarkError("bad_response", "attest: answer outside the contract (§2.6)", 200);
  }
  return v as unknown as AttestResult;
}

/** Counts shown in the status bar (Z3). */
export interface AttestCounts {
  attested: number;
  failed: number;
  /** the last failure, for the tooltip */
  lastError: string | null;
}

/** Sends the attestations of one connection. */
export class Attester {
  private session: AttestSession | null = null;
  readonly counts: AttestCounts = { attested: 0, failed: 0, lastError: null };

  constructor(
    readonly conn: Connection,
    private readonly transport: Transport,
    private readonly subtle: SubtleCrypto,
  ) {}

  private async post(endpoint: AttestEndpoint, body: unknown): Promise<Record<string, unknown>> {
    const req: HttpRequest = {
      method: "POST",
      url: `${this.conn.baseUrl}/v1/${endpoint}`,
      headers: {
        Authorization: `Bearer ${this.conn.token}`,
        "X-Consumer": this.conn.consumer,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
    };
    let res;
    try {
      res = await this.transport(req);
    } catch (e) {
      throw new RootmarkError("unreachable", `${endpoint}: rootmark serve did not answer (${String(e)})`, null);
    }
    return decodeHttp(endpoint, res);
  }

  private async openSession(): Promise<AttestSession> {
    this.session = checkSession(await this.post("attest_session", {}));
    return this.session;
  }

  /** Attests the saved bytes of `docPath`. An expired or unknown session
   * (the server restarted) is opened again once. */
  async attest(
    docPath: string,
    bytes: Uint8Array,
    savedAtSeq: number | null = null,
    how: SavedHow = {},
  ): Promise<AttestResult> {
    const hash = contentHash(bytes);
    const send = async (s: AttestSession): Promise<AttestResult> => {
      const mac = await hmacHex(this.subtle, s.key, macMessage(docPath, hash, savedAtSeq));
      return checkResult(
        await this.post("attest", {
          doc_path: docPath,
          content_hash: hash,
          ...(savedAtSeq === null ? {} : { saved_at_seq: savedAtSeq }),
          ...(how.renamedFrom === undefined ? {} : { renamed_from: how.renamedFrom }),
          ...(how.linkUpdateOf === undefined ? {} : { link_update_of: how.linkUpdateOf }),
          session_id: s.session_id,
          mac,
        }),
      );
    };
    try {
      let r: AttestResult;
      try {
        r = await send(this.session ?? (await this.openSession()));
      } catch (e) {
        if (!(e instanceof RootmarkError) || e.code !== "denied" || this.session === null) throw e;
        r = await send(await this.openSession());
      }
      this.counts.attested += 1;
      return r;
    } catch (e) {
      this.counts.failed += 1;
      this.counts.lastError = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }
}
