// P2-B/B4 (c): the contract checker catches every kind of call outside the
// contract (unknown endpoint or tool, unknown or missing field, wrong type,
// missing header, wrong method) and accepts conforming exchanges.
import { describe, expect, it } from "vitest";
import { checkCall, report, type CallRecord } from "../src/contract-check";

const H = { Authorization: `Bearer ${"a".repeat(64)}`, "X-Consumer": "test" };
const call = (path: string, body: unknown, over: Partial<CallRecord> = {}): CallRecord => ({
  method: "POST",
  url: `http://127.0.0.1:7357${path}`,
  headers: { ...H },
  body: JSON.stringify(body),
  status: null,
  response: null,
  ...over,
});
const req = (r: CallRecord) => checkCall(r).request;
const res = (r: CallRecord) => checkCall(r).response;

describe("requests", () => {
  it("conforming calls pass (tools, HTTP-only endpoints, GET status)", () => {
    expect(req(call("/v1/search", { query: "çay", k: 5, path_prefix: ["notes"], date_from: "2026-10-01", loc: true }))).toEqual([]);
    expect(req(call("/v1/events", { since_seq: 0, kinds: ["Promoted"], limit: 10 }))).toEqual([]);
    expect(req(call("/v1/demote", { lineage_id: "l", reason: "r", to: "proposal" }))).toEqual([]);
    expect(req(call("/v1/attest_session", {}))).toEqual([]);
    expect(req({ ...call("/v1/status", {}), method: "GET", body: "" })).toEqual([]);
  });

  it("an unknown endpoint or tool", () => {
    // built, not written: scripts/check_deps.sh refuses a client source naming an off-contract path
    expect(req(call(["", "v1", "delete"].join("/"), {}))[0]).toContain("not an endpoint of the contract");
    expect(req(call("/v2/search", { query: "x" }))[0]).toContain("not an endpoint");
  });

  it("an unknown field, a missing field, a wrong type, a bound", () => {
    expect(req(call("/v1/search", { query: "x", fuzzy: true }))).toEqual(["POST /v1/search arguments: unknown field fuzzy"]);
    expect(req(call("/v1/blame", {}))).toEqual(["POST /v1/blame arguments: missing field doc_path"]);
    expect(req(call("/v1/search", { query: "x", k: "5" }))[0]).toContain("arguments.k");
    expect(req(call("/v1/events", { limit: 1001 }))[0]).toContain("above 1000");
    expect(req(call("/v1/demote", { lineage_id: "l", reason: "r", to: "canonical" }))[0]).toContain("is not one of");
  });

  it("a missing header, a malformed token, the wrong method", () => {
    expect(req(call("/v1/blame", { doc_path: "a.md" }, { headers: { Authorization: H.Authorization } }))).toEqual(["POST /v1/blame: missing header X-Consumer (§1)"]);
    expect(req(call("/v1/blame", { doc_path: "a.md" }, { headers: { ...H, Authorization: "Bearer xyz" } }))[0]).toContain("64 hex");
    expect(req({ ...call("/v1/blame", { doc_path: "a.md" }), method: "GET" })[0]).toContain("method");
    expect(req(call("/v1/blame", { doc_path: "a.md" }, { url: "http://example.org/v1/blame" }))[0]).toContain("not 127.0.0.1");
  });
});

describe("responses", () => {
  it("a conforming answer and error pass; a missing or mistyped field does not", () => {
    const ok = { doc_path: "a.md", at: null, loc_stale: false, blocks: [] };
    expect(res(call("/v1/blame", { doc_path: "a.md" }, { status: 200, response: JSON.stringify(ok) }))).toEqual([]);
    expect(res(call("/v1/blame", { doc_path: "a.md" }, { status: 200, response: JSON.stringify({ ...ok, extra: 1 }) }))).toEqual([]);
    expect(res(call("/v1/blame", { doc_path: "a.md" }, { status: 200, response: JSON.stringify({ at: null, blocks: [] }) }))[0]).toContain("missing field doc_path");
    expect(res(call("/v1/blame", { doc_path: "a.md" }, { status: 200, response: JSON.stringify({ ...ok, loc_stale: "no" }) }))[0]).toContain("loc_stale");
    expect(res(call("/v1/blame", { doc_path: "a.md" }, { status: 404, response: JSON.stringify({ error: { code: "not_found", message: "m" } }) }))).toEqual([]);
    expect(res(call("/v1/blame", { doc_path: "a.md" }, { status: 500, response: JSON.stringify({ error: { code: "not_found", message: "m" } }) }))[0]).toContain("§3");
    expect(res(call("/v1/blame", { doc_path: "a.md" }, { status: 400, response: JSON.stringify({ error: { code: "nope", message: "m" } }) }))[0]).toContain("not in §3");
  });
});

describe("report", () => {
  it("counts calls by endpoint and keeps deliberate violations apart", () => {
    const r = report([
      call("/v1/blame", { doc_path: "a.md" }),
      { ...call("/v1/promote", { lineage_id: "l", reason: "" }), deliberate: "negative test" },
      call("/v1/blame", {}),
    ]);
    expect(r.calls).toBe(3);
    expect(r.endpoints).toEqual({ "POST /v1/blame": 2, "POST /v1/promote": 1 });
    expect(r.request_violations).toEqual(["#2 POST /v1/blame arguments: missing field doc_path"]);
    expect(r.deliberate_violations[0]).toContain("deliberate: negative test");
  });
});
