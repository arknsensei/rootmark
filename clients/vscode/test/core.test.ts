// The VS Code command layer against a fake `rootmark serve` (contract
// tests): what goes out (endpoint, headers, arguments) and what the
// commands make of the answers. Every exchange is checked by the contract
// checker (test/setup.ts).
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { RootmarkClient } from "../../obsidian/src/api";
import { contentHash } from "../../obsidian/src/blake3";
import { fetchTransport } from "../../obsidian/src/node-transport";
import { startFake, type Fake } from "../../obsidian/test/fake-server";
import { blameView, blockAt, CONSUMER, demote, historyLines, promote, search, statusLine } from "../src/core";

const TOKEN = "b".repeat(64);
let fake: Fake;
let client: RootmarkClient;

beforeEach(async () => {
  fake = await startFake();
  client = new RootmarkClient({ baseUrl: fake.url, token: TOKEN, consumer: CONSUMER }, fetchTransport);
});
afterEach(async () => {
  await fake.close();
});

const TEXT = "---\ntitle: x\n---\n# Başlık\n\nBir paragraf.\n\nÖneri metni.\n";
const HASH = contentHash(new TextEncoder().encode(TEXT));
const loc = (ls: number, le: number, bs: number, be: number, h = HASH) => ({ line_start: ls, line_end: le, byte_start: bs, byte_end: be, content_hash: h });
const block = (l: string, layer: string, lc?: ReturnType<typeof loc>, version = "f".repeat(32)) => ({
  lineage_id: l.repeat(32),
  version_id: version,
  layer,
  layer_source: "event",
  author_kind: layer === "proposal" ? "Agent" : "Human",
  source: layer === "proposal" ? "agent-x/1" : "",
  seq_created: 1,
  seq_last: 2,
  inactive: false,
  ...(lc === undefined ? {} : { loc: lc }),
});

describe("requests", () => {
  it("every request is POST /v1/<tool> with the bearer token and X-Consumer: vscode", async () => {
    fake.reply(200, { doc_path: "a.md", at: null, loc_stale: false, blocks: [] });
    await blameView(client, "a.md", TEXT);
    const r = fake.requests[0];
    expect(r?.method).toBe("POST");
    expect(r?.path).toBe("/v1/blame");
    expect(r?.headers["authorization"]).toBe(`Bearer ${TOKEN}`);
    expect(r?.headers["x-consumer"]).toBe("vscode");
    expect(JSON.parse(r?.body ?? "")).toEqual({ doc_path: "a.md" });
  });

  it("search sends the filters and asks for locations", async () => {
    fake.reply(200, { checkpoint_id: "c", mode: "lexical", include_proposal: true, query_hash: "q", trace_id: "t", hits: [], undated_excluded: 2 });
    const r = await search(client, "çay", { mode: "lexical", k: 5, include_proposal: true, path_prefix: ["notes"], date_from: "2026-10-01", date_to: "2026-10-31" });
    expect(JSON.parse(fake.requests[0]?.body ?? "")).toEqual({
      query: "çay",
      loc: true,
      mode: "lexical",
      k: 5,
      include_proposal: true,
      path_prefix: ["notes"],
      date_from: "2026-10-01",
      date_to: "2026-10-31",
    });
    expect(r.undated_excluded).toBe(2);
  });
});

describe("blame decorations from locations", () => {
  it("located: one range per block, per layer, only when the editor's bytes are the located version", async () => {
    fake.reply(200, {
      doc_path: "a.md",
      at: null,
      loc_stale: false,
      // a real answer's versions and locations for TEXT (`rootmark blame --loc`)
      blocks: [
        block("a", "canonical", loc(4, 4, 17, 27), "dfe6bf736a0bf236376948c1c3c64a6b"),
        block("b", "canonical", loc(6, 6, 29, 42), "3ab11f2c3c8ba870fc3d9acd4686b531"),
        block("c", "proposal", loc(8, 8, 44, 57), "2dcefe862764461e58e001dd182e7cc2"),
      ],
    });
    const v = await blameView(client, "a.md", TEXT);
    expect(v.kind).toBe("located");
    if (v.kind !== "located") return;
    expect(v.blocks.map((b) => [b.layer, b.start, b.end])).toEqual([
      ["canonical", 3, 3],
      ["canonical", 5, 5],
      ["proposal", 7, 7],
    ]);
    expect(blockAt(v, 7)?.lineage_id).toBe("c".repeat(32));
    expect(blockAt(v, 6)).toBeNull();
  });

  it("stale: the server says the file is not the version; no line", async () => {
    fake.reply(200, { doc_path: "a.md", at: null, loc_stale: true, blocks: [block("a", "canonical", loc(4, 4, 17, 28, "0".repeat(32)))] });
    const v = await blameView(client, "a.md", TEXT);
    expect(v.kind).toBe("stale");
    expect(blockAt(v, 3)).toBeNull();
  });

  it("a location whose bytes do not give the block's version: unlocated (K-P2B-T1); no line", async () => {
    fake.reply(200, { doc_path: "a.md", at: null, loc_stale: false, blocks: [block("a", "canonical", loc(4, 4, 18, 27), "dfe6bf736a0bf236376948c1c3c64a6b")] });
    const v = await blameView(client, "a.md", TEXT);
    expect(v.kind).toBe("unlocated");
  });

  it("changed: the editor holds other bytes than the located ones; no line", async () => {
    fake.reply(200, { doc_path: "a.md", at: null, loc_stale: false, blocks: [block("a", "canonical", loc(4, 4, 17, 28))] });
    const v = await blameView(client, "a.md", TEXT + "unsaved\n");
    expect(v.kind).toBe("changed");
  });

  it("unlocated: a v1 server's blame (no loc); no line", async () => {
    fake.reply(200, { doc_path: "a.md", at: null, blocks: [block("a", "canonical")] });
    const v = await blameView(client, "a.md", TEXT);
    expect(v.kind).toBe("unlocated");
  });

  it("untracked: not_found is not an error for the panel", async () => {
    fake.reply(404, { error: { code: "not_found", message: "blame: no snapshot for x.md" } });
    const v = await blameView(client, "x.md", TEXT);
    expect(v.kind).toBe("untracked");
  });
});

describe("search, history, promote, demote", () => {
  it("search: a hit's line is its first current location, never a stale one", async () => {
    fake.reply(200, {
      checkpoint_id: "c",
      mode: "hybrid",
      include_proposal: false,
      query_hash: "q",
      trace_id: "t",
      hits: [
        { doc_path: "a.md", chunk_id: "x", lineage_ids: ["a".repeat(32), "b".repeat(32)], layer: "canonical", score: 0.5, loc: [null, loc(6, 6, 30, 44)], loc_stale: false },
        { doc_path: "b.md", chunk_id: "y", lineage_ids: ["c".repeat(32)], layer: "canonical", score: 0.25, loc: [loc(2, 2, 3, 9)], loc_stale: true },
      ],
    });
    const r = await search(client, "çay");
    expect(r.items.map((i) => [i.doc_path, i.line])).toEqual([
      ["a.md", 5],
      ["b.md", null],
    ]);
    expect(r.undated_excluded).toBeNull();
  });

  it("history lines name the actor and the move", async () => {
    fake.reply(200, {
      lineage_id: "a".repeat(32),
      layer: "canonical",
      events: [
        { seq: 3, kind: "BlockCreated", doc_path: "a.md", lineage_ids: ["a".repeat(32)], meta: {} },
        { seq: 9, kind: "Promoted", doc_path: "a.md", lineage_ids: ["a".repeat(32)], meta: { actor: { kind: "human", id: "ada" }, from: "proposal", to: "canonical", reason: "ok" } },
      ],
    });
    const l = await historyLines(client, "a".repeat(32));
    expect(l[1]).toBe("#3 BlockCreated · actor not recorded");
    expect(l[2]).toBe("#9 Promoted · human:ada · proposal → canonical");
  });

  it("promote and demote: the decisions, demote over its HTTP-only endpoint", async () => {
    fake.reply(200, { decision: "promoted", lineage_id: "c".repeat(32), layer: "canonical", policy: "human_only", policy_version: "1", event: { seq: 10, kind: "Promoted", doc_path: "a.md", lineage_ids: ["c".repeat(32)], meta: {} } });
    expect(await promote(client, "c".repeat(32), "ok")).toBe(`promoted ${"c".repeat(8)} → canonical`);
    fake.reply(200, { decision: "denied", lineage_id: "c".repeat(32), layer: "canonical", reason: "deny", actor: { kind: "human", id: "ada" }, policy: "deny", policy_version: "1" });
    expect(await demote(client, "c".repeat(32), "undo")).toBe("denied: deny");
    expect(fake.requests.map((r) => r.path)).toEqual(["/v1/promote", "/v1/demote"]);
    expect(JSON.parse(fake.requests[1]?.body ?? "")).toEqual({ lineage_id: "c".repeat(32), reason: "undo" });
  });

  it("status line", () => {
    const s = statusLine({
      vault: "/v",
      spec_version: "2.0.3",
      mdk_version: "0.2.0",
      level_claimed: 3,
      embedder: null,
      index: { state: "ok", format: "mdkernel-index/3", expected_format: "mdkernel-index/3", auto_rebuild: true },
      last_checkpoint: null,
      policy: { promotion: "human_only", edit: "append_only" },
      server_policy: { server: {}, index: {}, identity: { human_id: null } },
      attestation: { unattested: "canonical", window_s: 2, stage: 1, pending: 0 },
    });
    expect(s.text).toBe("rootmark 0.2.0 · ok");
    expect(s.tooltip).toContain("VS Code saves are not attested");
    expect(s.tooltip).toContain("promote/demote are refused");
  });
});
