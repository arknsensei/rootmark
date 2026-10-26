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

// P2-R/R4 (R-9): the checker knows the contract's §2.10 — `POST /v1/prune`
// (an HTTP-only endpoint), its answer, the archived checkpoint and trace
// fields and the code `archived` (410). The clients make no such call; the
// samples are real answers of `rootmark prune` / `checkpoint list`
// (P2-R's determinism record, shortened lists).
describe("P2-R: prune and the archive (§2.10)", () => {
  const PRUNE = {"dry_run":false,"events_hash":"ce956a6b7b5cd46aa64ae4c0b29fd106","events_hash_before":"ac4d5e816bee32f061422ebca7339b7f","journal":{"blobs_removed":0,"now_ms":1800000000000,"records_kept":0,"records_removed":0},"retention":{"bound":{"checkpoint_id":"9d6798673c2db5cd9b193f36f264b1f7","event_seq":54},"checkpoints":["ebeeb7a63dcef4b55e5d56d62b1b398a"],"keep_checkpoints":2,"loc":{"kept":18,"removed":[{"doc_path":"n00.md","snapshot_id":"5f3e446029db38087e4931ce9daf0580"}]},"traces":{"archive":".rootmark/traces/archive/0-3.jsonl","base":4,"base_before":0,"filter_lines":0,"lines":4}},"rules":{"events":[61,62,63,64,65,66],"pruned":[{"doc_path":"agents/p0.md","idle_checkpoints":4,"last_seq":39,"layer":"proposal","lineage_id":"f2f6a40305aa7d8d63cd601eee5495d9","rule":"stale"}],"rules":["stale"],"version":"det"}};
  const ARCHIVED_CHECKPOINT = {"archived":true,"checkpoint_id":"ebeeb7a63dcef4b55e5d56d62b1b398a","docs":{"agents/p0.md":"7bdaceb211d5a09152d19afa6718d2e4","n00.md":"da37ee5c506b5598c5daf70f811fd0c5","n01.md":"65d9e2bdbb5d22bc23eb8460d958b682","n02.md":"ba093d240009574ac0b8c79aae6c5acf","n03.md":"92e3d84cc3214d9f3ecca90d494d3351","n04.md":"93b632aa56afd0fef89d0067a3473434","n05.md":"551ed61f01792c2b775068ba9b182bc0","n06.md":"c25c42bab8b5f79e886e244bdf6c5c14","n07.md":"1fd5ef10f9bdcdf819c77aeecc8e8ab2","n08.md":"7adabd016377f8d782a4f70c7fa39b35","n09.md":"9c475f7c49df41288ded9575e41c34ea","n10.md":"b18143c418f7c4c19d74663f96c96261","n11.md":"004616eb94eefa24e60727f99795a296"},"event_seq":41,"name":null};
  it("the endpoint and its body", () => {
    expect(req(call("/v1/prune", { dry_run: true, now_ms: 0 }))).toEqual([]);
    expect(req(call("/v1/prune", {}))).toEqual([]);
    expect(req(call("/v1/prune", { dry: true }))).toEqual(["POST /v1/prune arguments: unknown field dry"]);
  });
  it("the answers", () => {
    expect(res(call("/v1/prune", {}, { status: 200, response: JSON.stringify(PRUNE) }))).toEqual([]);
    expect(res(call("/v1/prune", {}, { status: 200, response: JSON.stringify({ ...PRUNE, journal: {} }) }))[0]).toContain("missing field");
    expect(res(call("/v1/checkpoints", {}, { status: 200, response: JSON.stringify({ checkpoints: [ARCHIVED_CHECKPOINT] }) }))).toEqual([]);
    expect(res(call("/v1/trace", {}, { status: 200, response: JSON.stringify({ traces: [], archived_before: 4 }) }))).toEqual([]);
    const archived = { error: { code: "archived", message: "checkpoint x is archived (.rootmark/checkpoints/archive/x.json)" } };
    expect(res(call("/v1/search", { query: "x", at: "x" }, { status: 410, response: JSON.stringify(archived) }))).toEqual([]);
    expect(res(call("/v1/search", { query: "x", at: "x" }, { status: 404, response: JSON.stringify(archived) }))[0]).toContain("§3: 410");
  });
});
