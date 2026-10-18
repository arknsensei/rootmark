// P2-0/Z3 (contract §2.6): the plugin's attestation client against a
// scripted transport — the two HTTP-only endpoints with the token and
// X-Consumer, the MAC the server checks (HMAC-SHA256 of the §2.6 message,
// cross-checked with Node's crypto), one new session after a `denied`,
// answers outside the contract refused, counts for the status bar.
import { createHmac, webcrypto } from "node:crypto";
import { describe, expect, it } from "vitest";
import { RootmarkError, type HttpRequest, type HttpResponse } from "../src/api";
import { Attester, macMessage } from "../src/attest";
import { contentHash } from "../src/blake3";

const KEY = "11".repeat(32);
const conn = { baseUrl: "http://127.0.0.1:7357", token: "ab".repeat(32), consumer: "rootmark-obsidian/test" };
const subtle = webcrypto.subtle as unknown as SubtleCrypto;

function server(script: ((req: HttpRequest) => HttpResponse)[]): { transport: (r: HttpRequest) => Promise<HttpResponse>; seen: HttpRequest[] } {
  const seen: HttpRequest[] = [];
  return {
    seen,
    transport: (req) => {
      seen.push(req);
      const f = script.shift();
      if (f === undefined) throw new Error("no more answers scripted");
      return Promise.resolve(f(req));
    },
  };
}

const ok = (body: unknown): HttpResponse => ({ status: 200, text: JSON.stringify(body) });
const session = (id: string): HttpResponse => ok({ session_id: id, key: KEY, ttl_s: 43200, stage: 1 });
const recorded = (req: HttpRequest): HttpResponse => {
  const b = JSON.parse(req.body ?? "{}") as { doc_path: string; content_hash: string };
  return ok({ attestation_id: "0".repeat(32), doc_path: b.doc_path, content_hash: b.content_hash, at_seq: 7, recorded: true, raised: [] });
};

describe("attest (contract §2.6)", () => {
  it("opens a session, then sends the MAC the server checks", async () => {
    const s = server([() => session("s1"), recorded]);
    const a = new Attester(conn, s.transport, subtle);
    const bytes = new TextEncoder().encode("# Note\n\nA human line.\n");
    const r = await a.attest("notes/a.md", bytes);
    expect(r.recorded).toBe(true);
    expect(s.seen.map((q) => q.url)).toEqual([`${conn.baseUrl}/v1/attest_session`, `${conn.baseUrl}/v1/attest`]);
    for (const q of s.seen) {
      expect(q.method).toBe("POST");
      expect(q.headers["Authorization"]).toBe(`Bearer ${conn.token}`);
      expect(q.headers["X-Consumer"]).toBe(conn.consumer);
    }
    const body = JSON.parse(s.seen[1]?.body ?? "") as Record<string, unknown>;
    const hash = contentHash(bytes);
    expect(body).toEqual({
      doc_path: "notes/a.md",
      content_hash: hash,
      session_id: "s1",
      mac: createHmac("sha256", Buffer.from(KEY, "hex")).update(`rootmark-attest/1\nnotes/a.md\n${hash}\n`).digest("hex"),
      // P2-0.1 T2: the saved bytes (UTF-8) for a partial match
      content: "# Note\n\nA human line.\n",
    });
    expect(a.counts).toEqual({ attested: 1, failed: 0, lastError: null });
  });

  it("bytes that are not UTF-8 are attested without content; a link update carries its old path (P2-0.1)", async () => {
    const s = server([() => session("s1"), recorded, recorded]);
    const a = new Attester(conn, s.transport, subtle);
    await a.attest("a.md", new Uint8Array([0xff, 0xfe, 0x41]));
    await a.attest("x.md", new TextEncoder().encode("See [[bee]].\n"), null, { linkUpdateOf: "sub/bee.md", linkUpdateFrom: "b.md" });
    const b1 = JSON.parse(s.seen[1]?.body ?? "") as Record<string, unknown>;
    const b2 = JSON.parse(s.seen[2]?.body ?? "") as Record<string, unknown>;
    expect("content" in b1).toBe(false);
    expect(b2["link_update_of"]).toBe("sub/bee.md");
    expect(b2["link_update_from"]).toBe("b.md");
    expect(b2["content"]).toBe("See [[bee]].\n");
  });

  it("the MAC message carries saved_at_seq in decimal", () => {
    expect(macMessage("a.md", "f".repeat(32), 12)).toBe(`rootmark-attest/1\na.md\n${"f".repeat(32)}\n12`);
    expect(macMessage("a.md", "f".repeat(32), null)).toBe(`rootmark-attest/1\na.md\n${"f".repeat(32)}\n`);
  });

  it("reuses the session; a denied attestation opens one new session once", async () => {
    const denied: HttpResponse = { status: 403, text: JSON.stringify({ error: { code: "denied", message: "unknown attestation session" } }) };
    const s = server([() => session("s1"), recorded, () => denied, () => session("s2"), recorded]);
    const a = new Attester(conn, s.transport, subtle);
    await a.attest("a.md", new Uint8Array([1]));
    await a.attest("a.md", new Uint8Array([2]));
    expect(s.seen.map((q) => q.url.split("/v1/")[1])).toEqual(["attest_session", "attest", "attest", "attest_session", "attest"]);
    expect((JSON.parse(s.seen[4]?.body ?? "") as { session_id: string }).session_id).toBe("s2");
    expect(a.counts.attested).toBe(2);
  });

  it("a second denial is an error, counted, never retried forever", async () => {
    const denied: HttpResponse = { status: 403, text: JSON.stringify({ error: { code: "denied", message: "wrong MAC" } }) };
    const s = server([() => session("s1"), () => denied, () => session("s2"), () => denied]);
    const a = new Attester(conn, s.transport, subtle);
    await expect(a.attest("a.md", new Uint8Array([1]))).rejects.toMatchObject({ code: "denied" });
    expect(a.counts).toMatchObject({ attested: 0, failed: 1 });
    expect(a.counts.lastError).toContain("wrong MAC");
  });

  it("an answer outside the contract is bad_response", async () => {
    const s = server([() => ok({ session_id: "s1", key: "short", ttl_s: 1, stage: 1 })]);
    const a = new Attester(conn, s.transport, subtle);
    const e = await a.attest("a.md", new Uint8Array([1])).catch((x: unknown) => x);
    expect(e).toBeInstanceOf(RootmarkError);
    expect((e as RootmarkError).code).toBe("bad_response");
    const s2 = server([() => session("s1"), () => ok({ attestation_id: "x" })]);
    const e2 = await new Attester(conn, s2.transport, subtle).attest("a.md", new Uint8Array([1])).catch((x: unknown) => x);
    expect((e2 as RootmarkError).code).toBe("bad_response");
  });

  it("no server: unreachable", async () => {
    const a = new Attester(conn, () => Promise.reject(new Error("ECONNREFUSED")), subtle);
    await expect(a.attest("a.md", new Uint8Array([1]))).rejects.toMatchObject({ code: "unreachable" });
  });
});
