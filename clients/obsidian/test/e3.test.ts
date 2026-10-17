// E3 component tests (TASKS E3): search panel model (mode, k,
// include_proposal, at; canonical order unchanged with proposals on),
// checkpoints (list, naming, read-only past view), traces (newest first,
// jump targets).
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { RootmarkClient, type Hit } from "../src/api";
import { blameAt, listCheckpoints, nameCheckpoint, recentTraces, runSearch } from "../src/commands";
import { blockLine } from "../src/align";
import { indexNotice, pastView } from "../src/view";
import { fetchTransport, startFake, type Fake } from "./fake-server";
import { SAMPLES } from "./samples";

let fake: Fake;
let client: RootmarkClient;
beforeEach(async () => {
  fake = await startFake();
  client = new RootmarkClient({ baseUrl: fake.url, token: "a".repeat(64), consumer: "rootmark-obsidian/0.1.0" }, fetchTransport);
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

  it("the server's order is kept even when it is not score order; a proposal scoring above a canonical hit stays apart", async () => {
    // the contract promises the canonical prefix equals the flag-off answer,
    // not monotone scores: any re-sort (stable by score included) fails here
    const odd = [hit("low", "canonical", 0.5), hit("high", "canonical", 9), hit("mid", "canonical", 2)];
    fake.reply(200, searchAnswer(odd, false));
    fake.reply(200, searchAnswer([...odd, hit("top", "proposal", 50), hit("p2", "proposal", 0.1)], true));
    const off = await runSearch(client, "x", { k: 3 });
    const on = await runSearch(client, "x", { k: 3, include_proposal: true });
    expect(off.canonical.map((h) => h.doc_path)).toEqual(["low.md", "high.md", "mid.md"]);
    expect(on.canonical).toEqual(off.canonical);
    expect(on.proposal.map((h) => h.doc_path)).toEqual(["top.md", "p2.md"]);
  });

  it("proposal hits that were not asked for are bad_response, never hidden", async () => {
    fake.reply(200, searchAnswer([hit("c", "canonical", 1), hit("p", "proposal", 1)], false));
    await expect(runSearch(client, "x")).rejects.toMatchObject({ code: "bad_response", message: expect.stringContaining("without include_proposal") as unknown });
    // the server claims the flag was off although it was sent on
    fake.reply(200, searchAnswer([hit("c", "canonical", 1), hit("p", "proposal", 1)], false));
    await expect(runSearch(client, "x", { include_proposal: true })).rejects.toMatchObject({ code: "bad_response" });
  });

  it("more than k hits in a layer (or 2k in all) is bad_response (§2.2 normative length)", async () => {
    fake.reply(200, searchAnswer([hit("a", "canonical", 3), hit("b", "canonical", 2), hit("c", "canonical", 1)], false));
    await expect(runSearch(client, "x", { k: 2 })).rejects.toMatchObject({ code: "bad_response", message: expect.stringContaining("at most k per layer") as unknown });
    fake.reply(200, searchAnswer([hit("a", "canonical", 3), hit("p", "proposal", 2), hit("q", "proposal", 1)], true));
    await expect(runSearch(client, "x", { k: 1, include_proposal: true })).rejects.toMatchObject({ code: "bad_response" });
    fake.reply(200, searchAnswer([hit("a", "canonical", 3), hit("p", "proposal", 2)], true));
    await expect(runSearch(client, "x", { k: 1, include_proposal: true })).resolves.toMatchObject({ canonical: [{ doc_path: "a.md" }] });
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

describe("trace jump to a block's line", () => {
  const ranges = [
    { start: 0, end: 0, heading: "A" },
    { start: 2, end: 3, heading: "A" },
    { start: 5, end: 5, heading: "A" },
  ];
  const blame = [{ lineage_id: "l0" }, { lineage_id: "l1" }, { lineage_id: "l2" }];
  it("the returned block's start line when the note aligns", () => {
    expect(blockLine(blame, ranges, "l1")).toEqual({ line: 2 });
  });
  it("otherwise the reason (never a guessed line)", () => {
    expect(blockLine(blame.slice(0, 2), ranges, "l1")).toMatchObject({ reason: expect.stringContaining("not aligned") as unknown });
    expect(blockLine(blame, ranges, "gone")).toMatchObject({ reason: expect.stringContaining("no longer in this note") as unknown });
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

describe("index state in the connection status (K-E1.2)", () => {
  const ix = (state: string, auto_rebuild = false) => ({ state, format: state === "stale" ? "mdkernel-index/2" : null, expected_format: "mdkernel-index/3", auto_rebuild });
  it("index_notice_ok_is_silent", () => {
    expect(indexNotice({ ...ix("ok"), format: "mdkernel-index/3" })).toBeNull();
  });
  it("index_notice_stale_names_formats_and_rebuild", () => {
    const n = indexNotice(ix("stale"));
    expect(n?.label).toBe("index stale");
    expect(n?.detail).toContain("mdkernel-index/2");
    expect(n?.detail).toContain("mdkernel-index/3");
    expect(n?.detail).toContain("rootmark index build");
    expect(n?.detail).toContain("auto_rebuild = true");
  });
  it("index_notice_missing_and_damaged", () => {
    expect(indexNotice(ix("missing", true))?.detail).toContain("restart `rootmark serve`");
    const d = indexNotice(ix("damaged"));
    expect(d?.label).toBe("index damaged");
    expect(d?.detail).toContain("never rebuilt automatically");
    expect(d?.detail).toContain("rootmark index build");
  });
});
