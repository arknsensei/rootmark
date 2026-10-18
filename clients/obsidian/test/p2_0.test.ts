// P2-0/Z3: the plugin's attested writes and `unverified` content (spec
// 2.0.0 §7.1; contract §2.6, §2.2, §2.5).
// - own-writes.ts against a fake adapter: a save, an append and a rename of
//   this Obsidian are reported with the bytes on disk; a rewrite of the bytes
//   already on disk (another process's, reloaded) is not; non-notes are not;
//   the adapter is restored as it was;
// - the gutter's `U` mark, the `unverified` tab of the proposal list, the
//   one-click promotion's request, and `unverified` search hits with the
//   proposals.
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { RootmarkClient, type BlameBlock, type Hit } from "../src/api";
import { listProposals, promoteAll, runSearch } from "../src/commands";
import { observeOwnWrites, type OwnWritesAdapter } from "../src/own-writes";
import { UNVERIFIED_REASON } from "../src/ui/reasons";
import { gutterMarks } from "../src/view";
import { fetchTransport, startFake, type Fake } from "./fake-server";

class FakeAdapter implements OwnWritesAdapter {
  files = new Map<string, string>();
  calls: string[] = [];
  write(path: string, data: string): Promise<void> {
    this.calls.push(`write ${path}`);
    this.files.set(path, data);
    return Promise.resolve();
  }
  append(path: string, data: string): Promise<void> {
    this.calls.push(`append ${path}`);
    this.files.set(path, (this.files.get(path) ?? "") + data);
    return Promise.resolve();
  }
  rename(from: string, to: string): Promise<void> {
    this.calls.push(`rename ${from} ${to}`);
    const d = this.files.get(from);
    if (d === undefined) return Promise.reject(new Error("no file"));
    this.files.set(to, d);
    this.files.set(from, "");
    return Promise.resolve();
  }
  process(path: string, fn: (data: string) => string): Promise<string> {
    this.calls.push(`process ${path}`);
    const out = fn(this.files.get(path) ?? "");
    this.files.set(path, out);
    return Promise.resolve(out);
  }
  exists(path: string): Promise<boolean> {
    return Promise.resolve(this.files.has(path));
  }
  read(path: string): Promise<string> {
    return Promise.resolve(this.files.get(path) ?? "");
  }
  readBinary(path: string): Promise<ArrayBuffer> {
    return Promise.resolve(new TextEncoder().encode(this.files.get(path) ?? "").buffer);
  }
}

const text = (b: Uint8Array): string => new TextDecoder().decode(b);

describe("own writes (src/own-writes.ts)", () => {
  it("reports this Obsidian's saves, appends and renames with the bytes on disk", async () => {
    const a = new FakeAdapter();
    const seen: [string, string][] = [];
    const restore = observeOwnWrites(a, (p, b) => seen.push([p, text(b)]));
    await a.write("n.md", "# N\n\nOne.\n");
    await a.append("n.md", "\nTwo.\n");
    await a.rename("n.md", "m.md");
    await a.write(".obsidian/workspace.json", "{}");
    expect(seen).toEqual([
      ["n.md", "# N\n\nOne.\n"],
      ["n.md", "# N\n\nOne.\n\nTwo.\n"],
      ["m.md", "# N\n\nOne.\n\nTwo.\n"],
    ]);
    // every call reached the adapter unchanged
    expect(a.calls).toEqual(["write n.md", "append n.md", "rename n.md m.md", "write .obsidian/workspace.json"]);
    restore();
  });

  it("a rewrite of the bytes already on disk is no save (Z3 review, finding 1)", async () => {
    const a = new FakeAdapter();
    // another process wrote these bytes; Obsidian reloaded and writes them back
    a.files.set("n.md", "# N\n\nAn agent's text.\n");
    const seen: string[] = [];
    observeOwnWrites(a, (p) => seen.push(p));
    await a.write("n.md", "# N\n\nAn agent's text.\n");
    expect(seen).toEqual([]);
    await a.write("n.md", "# N\n\nAn agent's text.\n\nA human line.\n");
    expect(seen).toEqual(["n.md"]);
  });

  it("a rename is reported only when the caller allows it (Z3 review, 11)", async () => {
    const a = new FakeAdapter();
    const seen: string[] = [];
    const asked: string[] = [];
    observeOwnWrites(
      a,
      (p) => seen.push(p),
      (from) => {
        asked.push(from);
        return Promise.resolve(from === "h.md");
      },
    );
    // both saved by this Obsidian; only h.md is canonical in the log
    await a.write("p.md", "# P\n\nAn agent's proposal, edited here.\n");
    await a.write("h.md", "# H\n\nHuman text.\n");
    await a.rename("p.md", "q.md");
    await a.rename("h.md", "g.md");
    expect(asked).toEqual(["p.md", "h.md"]);
    expect(seen).toEqual(["p.md", "h.md", "g.md"]);
    // a failing check is a refusal
    const b = new FakeAdapter();
    const seenB: string[] = [];
    observeOwnWrites(b, (p) => seenB.push(p), () => Promise.reject(new Error("offline")));
    await b.write("g.md", "# G\n");
    await b.rename("g.md", "f.md");
    expect(seenB).toEqual(["g.md"]);
  });

  it("an append of bytes this Obsidian did not report is not reported; a rename carries its old path for the server's check (security reviews)", async () => {
    const a = new FakeAdapter();
    const seen: string[] = [];
    const renames: (string | undefined)[] = [];
    observeOwnWrites(
      a,
      (p, _b, how) => {
        seen.push(p);
        renames.push(how.renamedFrom);
      },
      () => Promise.resolve(true),
    );
    // another process wrote this file; this Obsidian never saved it
    a.files.set("agent.md", "# A\n\nAn agent's unseen text.\n");
    await a.append("agent.md", "\nAn appended line.\n");
    expect(seen).toEqual([]);
    // a rename is reported with renamed_from: the server accepts it only if
    // the bytes are the canonical head of the old document (contract §2.6)
    await a.rename("agent.md", "renamed.md");
    expect(seen).toEqual(["renamed.md"]);
    expect(renames).toEqual(["agent.md"]);
    seen.length = 0;
    // saved here, then changed on disk by someone else: an append is not ours
    await a.write("n.md", "# N\n");
    a.files.set("n.md", "# N\n\nForeign bytes.\n");
    await a.append("n.md", "\nMore.\n");
    expect(seen).toEqual(["n.md"]);
  });

  it("a process write right after a rename is reported as its link update; any other process write is not (P2-0.1/Y1)", async () => {
    const a = new FakeAdapter();
    let t = 1_000;
    const seen: [string, string, string | undefined][] = [];
    observeOwnWrites(
      a,
      (p, b, how) => seen.push([p, text(b), how.linkUpdateOf]),
      () => Promise.resolve(false),
      () => t,
    );
    a.files.set("a.md", "See [[b]].\n");
    a.files.set("b.md", "# B\n");
    // a process write with no rename before it: not a save of this editor
    await a.process("a.md", (d) => d.replace("See", "Saw"));
    expect(seen).toEqual([]);
    // Obsidian renames b.md, then rewrites the links to it
    await a.rename("b.md", "sub/bee.md");
    t += 50;
    await a.process("a.md", (d) => d.replace("[[b]]", "[[bee]]"));
    await a.process("c.json", (d) => d + "x");
    await a.process("a.md", (d) => d);
    expect(seen).toEqual([["a.md", "Saw [[bee]].\n", "sub/bee.md"]]);
    // two renames in a row: the rewrite is attributed to the rename whose
    // note it names; a process write naming neither is not reported
    a.files.set("c.md", "# C\n");
    a.files.set("d.md", "# D\n");
    a.files.set("x.md", "See [[c]] and [[d]].\n");
    await a.rename("c.md", "c2.md");
    await a.rename("d.md", "d2.md");
    await a.process("x.md", (d) => d.replace("[[c]]", "[[c2]]"));
    await a.process("x.md", (d) => d.replace("See", "Seen"));
    expect(seen.slice(1)).toEqual([["x.md", "See [[c2]] and [[d]].\n", "c2.md"]]);
    seen.splice(1);
    // a new name inside the old one (bee.md → b.md): counted on link targets
    a.files.set("bee.md", "# Bee\n");
    a.files.set("y.md", "See [[bee]].\n");
    await a.rename("bee.md", "b.md");
    await a.process("y.md", (d) => d.replace("[[bee]]", "[[b]]"));
    expect(seen.slice(1)).toEqual([["y.md", "See [[b]].\n", "b.md"]]);
    seen.splice(1);
    // long after the rename: not reported
    t += 20_000;
    await a.process("a.md", (d) => d + "more\n");
    expect(seen.length).toBe(1);
    expect(a.calls.filter((c) => c.startsWith("process"))).toEqual(["process a.md", "process a.md", "process c.json", "process a.md", "process x.md", "process x.md", "process y.md", "process a.md"]);
  });

  it("restores the adapter's own methods (and removes the wrappers of inherited ones)", async () => {
    const a = new FakeAdapter();
    const seen: string[] = [];
    const restore = observeOwnWrites(a, (p) => seen.push(p));
    expect(Object.prototype.hasOwnProperty.call(a, "write")).toBe(true);
    restore();
    for (const m of ["write", "append", "rename", "process"]) expect(Object.prototype.hasOwnProperty.call(a, m)).toBe(false);
    await a.write("n.md", "x");
    expect(seen).toEqual([]);
  });
});

const block = (lineage: string, layer: string): BlameBlock => ({
  lineage_id: lineage,
  version_id: `v-${lineage}`,
  layer: layer as BlameBlock["layer"],
  layer_source: "event",
  author_kind: "Human",
  source: "unattested_fs",
  seq_created: 1,
  seq_last: 2,
  inactive: false,
});

describe("unverified in the views", () => {
  it("the gutter marks an unverified block U", () => {
    const marks = gutterMarks({
      aligned: true,
      pairs: [
        { block: block("a", "canonical"), range: { start: 0, end: 0, heading: null } },
        { block: block("b", "unverified"), range: { start: 2, end: 2, heading: null } },
      ],
    } as never);
    expect(marks.map((m) => [m.label, m.cls])).toEqual([
      ["C", "rootmark-layer-canonical"],
      ["U", "rootmark-layer-unverified"],
    ]);
    expect(marks[1]?.title).toContain("unverified · Human · source: unattested_fs");
  });
});

let fake: Fake;
let client: RootmarkClient;
beforeEach(async () => {
  fake = await startFake();
  client = new RootmarkClient({ baseUrl: fake.url, token: "a".repeat(64), consumer: "rootmark-obsidian/0.1.0" }, fetchTransport);
});
afterEach(async () => {
  await fake.close();
});

describe("unverified over the contract", () => {
  it("the unverified tab lists only unverified blocks; Mine promotes with the fixed reason", async () => {
    fake.reply(200, { doc_path: "a.md", at: null, blocks: [block("c", "canonical"), block("u", "unverified"), block("p", "proposal")] });
    const rows = await listProposals(client, ["a.md"], () => null, ["unverified"]);
    expect(rows.map((r) => [r.lineage_id, r.layer])).toEqual([["u", "unverified"]]);
    fake.reply(200, {
      decision: "promoted",
      lineage_id: "u",
      layer: "canonical",
      policy: "human_only",
      policy_version: "1",
      event: { seq: 9, kind: "Promoted", doc_path: "a.md", lineage_ids: ["u"], meta: { from: "unverified", to: "canonical" } },
    });
    const out = await promoteAll(client, ["u"], UNVERIFIED_REASON);
    expect(out[0]?.result?.decision).toBe("promoted");
    expect(JSON.parse(fake.requests[1]?.body ?? "")).toEqual({ lineage_id: "u", reason: UNVERIFIED_REASON });
  });

  it("unverified search hits come with the proposals, after the canonical ones", async () => {
    const h = (id: string, layer: Hit["layer"], score: number): Hit => ({ doc_path: `${id}.md`, chunk_id: `c-${id}`, lineage_ids: [`l-${id}`], layer, score });
    fake.reply(200, {
      checkpoint_id: "c".repeat(32),
      mode: "lexical",
      include_proposal: true,
      query_hash: "q".repeat(32),
      trace_id: "t".repeat(32),
      hits: [h("c1", "canonical", 2), h("p1", "proposal", 0.9), h("u1", "unverified", 0.8)],
    });
    const r = await runSearch(client, "kalimba", { mode: "lexical", k: 3, include_proposal: true });
    expect(r.canonical.map((x) => x.doc_path)).toEqual(["c1.md"]);
    expect(r.proposal.map((x) => [x.doc_path, x.layer])).toEqual([
      ["p1.md", "proposal"],
      ["u1.md", "unverified"],
    ]);
  });
});
