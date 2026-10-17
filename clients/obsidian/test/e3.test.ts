// E3 component tests (TASKS E3): search panel model (mode, k,
// include_proposal, at; canonical order unchanged with proposals on),
// checkpoints (list, naming, read-only past view), traces (newest first,
// jump targets).
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { MdkClient, type Hit } from "../src/api";
import { blameAt, listCheckpoints, nameCheckpoint, recentTraces, runSearch } from "../src/commands";
import { pastView } from "../src/view";
import { fetchTransport, startFake, type Fake } from "./fake-server";
import { SAMPLES } from "./samples";

let fake: Fake;
let client: MdkClient;
beforeEach(async () => {
  fake = await startFake();
  client = new MdkClient({ baseUrl: fake.url, token: "a".repeat(64), consumer: "mdkernel-obsidian/0.1.0" }, fetchTransport);
});
afterEach(async () => {
  await fake.close();
});

const hit = (id: string, layer: "canonical" | "proposal", score: number): Hit => ({
  doc_path: `${id}.md`,
  chunk_id: `chunk-${id}`,
  lineage_ids: [`l-${id}`],
  layer,
  score,
});
const searchAnswer = (hits: Hit[], include: boolean): unknown => ({
  checkpoint_id: "c".repeat(32),
  mode: "lexical",
  include_proposal: include,
  query_hash: "q".repeat(32),
  trace_id: "t".repeat(32),
  hits,
});

describe("search panel (contract §2.2)", () => {
  const canonical = [hit("c1", "canonical", 3.1), hit("c2", "canonical", 2.0), hit("c3", "canonical", 2.0)];

  it("sends query, mode, k, include_proposal and at as given", async () => {
    fake.reply(200, searchAnswer(canonical, false));
    await runSearch(client, "kalimba", { mode: "lexical", k: 3, include_proposal: false, at: "yesterday" });
    expect(JSON.parse(fake.requests[0]?.body ?? "")).toEqual({ query: "kalimba", mode: "lexical", k: 3, include_proposal: false, at: "yesterday" });
  });

  it("include_proposal on: the canonical list is the flag-off list, same order; proposals apart", async () => {
    // a contract-following server: the canonical prefix is byte-equal to the
    // flag-off answer, then the proposals (separate pass, ×0.5)
    fake.reply(200, searchAnswer(canonical, false));
    fake.reply(200, searchAnswer([...canonical, hit("p1", "proposal", 1.5), hit("p2", "proposal", 0.2)], true));
    const off = await runSearch(client, "kalimba", { mode: "lexical", k: 3 });
    const on = await runSearch(client, "kalimba", { mode: "lexical", k: 3, include_proposal: true });
    expect(on.canonical).toEqual(off.canonical);
    expect(on.canonical.map((h) => h.doc_path)).toEqual(["c1.md", "c2.md", "c3.md"]);
    expect(on.proposal.map((h) => h.doc_path)).toEqual(["p1.md", "p2.md"]);
    expect(off.proposal).toEqual([]);
  });

  it("ties keep the server's order (the client never re-ranks)", async () => {
    const tied = [hit("z", "canonical", 1), hit("a", "canonical", 1)];
    fake.reply(200, searchAnswer(tied, false));
    expect((await runSearch(client, "x")).canonical.map((h) => h.doc_path)).toEqual(["z.md", "a.md"]);
  });

  it("an answer outside §2.2 is bad_response: a derived hit, a canonical hit after a proposal", async () => {
    fake.reply(200, searchAnswer([hit("d", "canonical", 1), { ...hit("x", "canonical", 1), layer: "derived" }], false));
    await expect(runSearch(client, "x")).rejects.toMatchObject({ code: "bad_response" });
    fake.reply(200, searchAnswer([hit("p", "proposal", 1), hit("c", "canonical", 1)], true));
    await expect(runSearch(client, "x", { include_proposal: true })).rejects.toMatchObject({ code: "bad_response" });
  });

  it("an empty query is the server's empty_query, shown", async () => {
    fake.reply(400, { error: { code: "empty_query", message: "search: empty query" } });
    await expect(runSearch(client, "  ")).rejects.toMatchObject({ code: "empty_query", status: 400 });
  });
});

describe("checkpoints (contract §2.3)", () => {
  it("lists in creation order and names the current state", async () => {
    fake.reply(200, SAMPLES.checkpoints);
    const cps = await listCheckpoints(client);
    expect(cps.map((c) => c.name)).toEqual([null, "y"]);
    fake.reply(200, SAMPLES.checkpoint);
    const cp = await nameCheckpoint(client, "yesterday");
    expect(JSON.parse(fake.requests[1]?.body ?? "")).toEqual({ name: "yesterday" });
    expect(cp.checkpoint_id).toHaveLength(32);
  });

  it("a taken name is the server's exists (409), not swallowed", async () => {
    fake.reply(409, { error: { code: "exists", message: 'name "y" is taken' } });
    await expect(nameCheckpoint(client, "y")).rejects.toMatchObject({ code: "exists", status: 409 });
  });

  it("the past view is blame at the checkpoint, read only; a note without a snapshot then says so", async () => {
    fake.reply(200, { ...(SAMPLES.blame as object), at: "y" });
    const blocks = await blameAt(client, "a.md", "y");
    expect(JSON.parse(fake.requests[0]?.body ?? "")).toEqual({ doc_path: "a.md", at: "y" });
    expect(pastView("a.md", "y", blocks).split("\n")).toEqual([
      "a.md at y (read only)",
      "  1 canonical Human  — · f4495f5e",
      "  2 canonical Human  — · 3a9c0a74",
      "  3 canonical Human  — · 4f6b5cb6",
    ]);
    fake.reply(404, { error: { code: "not_found", message: "blame: a.md has no snapshot at checkpoint y" } });
    expect(pastView("a.md", "y", await blameAt(client, "a.md", "y"))).toBe("a.md had no snapshot at checkpoint y");
  });
});

describe("traces (contract §2)", () => {
  it("newest first, limited, with the returned blocks as jump targets; consumer filter sent", async () => {
    fake.reply(200, SAMPLES.trace);
    const rows = await recentTraces(client, { consumer: "probe", limit: 1 });
    expect(JSON.parse(fake.requests[0]?.body ?? "")).toEqual({ consumer: "probe" });
    expect(rows).toHaveLength(1);
    expect(rows[0]?.at_seq).toBe(1);
    expect(rows[0]?.returned).toEqual([
      { doc_path: "a.md", lineage_id: "f4495f5ee61ea960be41bfa9de348401", version_id: "ba2f8a67da73b8981e9d95879a3df51d" },
    ]);
    fake.reply(200, SAMPLES.trace);
    expect((await recentTraces(client)).map((r) => r.at_seq)).toEqual([1, 0]);
    expect(JSON.parse(fake.requests[1]?.body ?? "")).toEqual({});
  });
});
