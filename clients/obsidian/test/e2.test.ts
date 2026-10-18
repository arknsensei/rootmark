// E2 component tests (TASKS E2): blame alignment and gutter marks, history
// lines, the proposal list, Promote over the human channel (E-5 against a
// fake server), Demote refused, and a static scan that the plugin never
// calls a file-writing API.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { RootmarkClient, RootmarkError, type BlameBlock, type LogEvent } from "../src/api";
import { align, editorBlocks } from "../src/align";
import { actorOf, blameOf, demote, DEMOTE_UNAVAILABLE, listProposals, promoteAll, promoteSummary } from "../src/commands";
import { gutterMarks, historyLine } from "../src/view";
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

const block = (lineage: string, layer: "canonical" | "proposal", extra: Partial<BlameBlock> = {}): BlameBlock => ({
  lineage_id: lineage,
  version_id: `v${lineage}`,
  layer,
  layer_source: "event",
  author_kind: layer === "canonical" ? "Human" : "Agent",
  source: layer === "canonical" ? "" : "scene-agent/1.0",
  seq_created: 1,
  seq_last: 1,
  inactive: false,
  ...extra,
});

describe("blame alignment (gutter)", () => {
  // # A / para / - one / - two (nested - two.a) / ```code``` with front matter
  const sections = [
    { type: "yaml", start: 0, end: 2 },
    { type: "heading", start: 3, end: 3 },
    { type: "paragraph", start: 5, end: 6 },
    { type: "list", start: 8, end: 10 },
    { type: "code", start: 12, end: 14 },
  ];
  const items = [
    { start: 8, end: 8, parent: -8 },
    { start: 9, end: 10, parent: -8 },
    { start: 10, end: 10, parent: 9 },
  ];
  const headings = [{ line: 3, heading: "A" }];

  it("editor blocks follow the spec's block model: no front matter, one block per top-level list item", () => {
    expect(editorBlocks(sections, items, headings)).toEqual([
      { start: 3, end: 3, heading: "A" },
      { start: 5, end: 6, heading: "A" },
      { start: 8, end: 8, heading: "A" },
      { start: 9, end: 10, heading: "A" },
      { start: 12, end: 14, heading: "A" },
    ]);
  });

  it("aligned by order when the counts agree; marks carry layer, author, source, last event", () => {
    const blame = [block("h", "canonical"), block("p", "canonical"), block("l1", "canonical"), block("l2", "proposal", { seq_created: 7, seq_last: 9 }), block("c", "canonical")];
    const al = align(blame, editorBlocks(sections, items, headings));
    const marks = gutterMarks(al);
    expect(marks.map((m) => [m.line, m.label])).toEqual([[3, "C"], [5, "C"], [8, "C"], [9, "P"], [12, "C"]]);
    expect(marks[3]?.cls).toBe("rootmark-layer-proposal");
    expect(marks[3]?.title).toBe("proposal · Agent · source: scene-agent/1.0 · created at seq 7, last event seq 9");
    expect(marks[0]?.title).toContain("source: —");
  });

  it("counts that disagree: no marks, a reason, never a guessed line", () => {
    const al = align([block("h", "canonical")], editorBlocks(sections, items, headings));
    expect(al.aligned).toBe(false);
    expect(al.aligned ? "" : al.reason).toContain("rootmark has 1 blocks, the editor 5");
    expect(gutterMarks(al)).toEqual([]);
    expect(editorBlocks([{ type: "thematicBreak", start: 0, end: 0 }], [])).toEqual([]);
  });

  it("an allow-list: any section type without a sure block breaks the alignment", () => {
    for (const t of ["html", "comment", "footnoteDefinition", "definition", "math", "callout", "somethingNew"]) {
      expect(editorBlocks([{ type: "paragraph", start: 0, end: 0 }, { type: t, start: 2, end: 2 }], []), t).toEqual([]);
    }
    for (const t of ["heading", "paragraph", "code", "table", "blockquote"]) {
      expect(editorBlocks([{ type: t, start: 0, end: 1 }], []), t).toHaveLength(1);
    }
  });

  it("a list on line 0 (parent -0) is top level; a child of line 0 (parent 0) is not", () => {
    const r = editorBlocks(
      [{ type: "list", start: 0, end: 2 }],
      [{ start: 0, end: 1, parent: -0 }, { start: 1, end: 1, parent: 0 }, { start: 2, end: 2, parent: -0 }],
    );
    expect(r.map((x) => x.start)).toEqual([0, 2]);
  });
});

describe("history lines (contract §4.2)", () => {
  const ev = (meta: Record<string, unknown>, kind = "BlockDeleted"): LogEvent => ({ seq: 5, kind, doc_path: "a.md", lineage_ids: ["l"], meta });
  it("shows the actor, or says it was not recorded (never invents one)", () => {
    expect(historyLine(ev({ actor: { kind: "agent", id: "a/1" }, source: "a/1" }))).toBe("#5 BlockDeleted · agent:a/1 · source: a/1");
    expect(historyLine(ev({ matcher_version: "v3", provenance: { author_kind: "Human", source: "" } }, "BlockCreated"))).toBe(
      "#5 BlockCreated · actor not recorded · written by Human",
    );
    expect(historyLine(ev({ actor: { kind: "human", id: "ada" }, from: "proposal", to: "canonical", reason: "ok" }, "Promoted"))).toBe(
      "#5 Promoted · human:ada · reason: ok · proposal → canonical",
    );
    expect(actorOf({ actor: "bob" })).toBe("actor not recorded");
  });
});

describe("proposal list", () => {
  it("vault-wide proposal blocks; untracked documents skipped; other errors raised", async () => {
    const blame = structuredClone(SAMPLES.blame) as { blocks: BlameBlock[] };
    blame.blocks.push(block("prop1", "proposal"), block("gone", "proposal", { inactive: true }));
    fake.reply(200, blame); // a.md
    fake.reply(404, { error: { code: "not_found", message: "blame: no snapshot for untracked.md" } });
    const rows = await listProposals(client, ["untracked.md", "a.md"], (doc, blocks) => blocks.map((_, i) => `${doc}#${String(i)}`));
    expect(fake.requests.map((r) => JSON.parse(r.body) as unknown)).toEqual([{ doc_path: "a.md" }, { doc_path: "untracked.md" }]);
    expect(rows).toEqual([
      { doc_path: "a.md", layer: "proposal", lineage_id: "prop1", version_id: "vprop1", source: "scene-agent/1.0", author_kind: "Agent", seq_created: 1, seq_last: 1, section: "a.md#3" },
    ]);
    // not aligned → every section unknown ("—"), never a positional guess
    fake.reply(200, blame);
    const unaligned = await listProposals(client, ["a.md"], () => null);
    expect(unaligned.map((r) => r.section)).toEqual([null]);
    fake.reply(503, { error: { code: "index_stale", message: "x" } });
    await expect(blameOf(client, "a.md")).rejects.toMatchObject({ code: "index_stale" });
  });
});

describe("Promote over HTTP — E-5 against a fake server", () => {
  it("promote sends the reason; the answer carries Promoted by human:<human_id>", async () => {
    fake.reply(200, SAMPLES.promote);
    const out = await promoteAll(client, ["f6aed2eee498ffff1d3679f32bb62e36"], "approved in the scene");
    const r = fake.requests[0];
    expect(r?.path).toBe("/v1/promote");
    expect(JSON.parse(r?.body ?? "")).toEqual({ lineage_id: "f6aed2eee498ffff1d3679f32bb62e36", reason: "approved in the scene" });
    const res = out[0]?.result;
    expect(res?.decision).toBe("promoted");
    expect(res?.decision === "promoted" ? res.event.meta.actor : null).toEqual({ id: "ada", kind: "human" });
    expect(promoteSummary(out)).toBe("1 promoted");
  });

  it("without a reason the server's 400 invalid_arguments is reported, not retried", async () => {
    fake.reply(400, { error: { code: "invalid_arguments", message: "promote: reason must not be empty" } });
    const out = await promoteAll(client, ["l"], "");
    expect(fake.requests).toHaveLength(1);
    expect(out[0]?.error).toBeInstanceOf(RootmarkError);
    expect(out[0]?.error?.code).toBe("invalid_arguments");
    expect(out[0]?.error?.status).toBe(400);
    expect(promoteSummary(out)).toBe("0 promoted, 1 failed (invalid_arguments: promote: reason must not be empty)");
  });

  it("a policy refusal is shown as such", async () => {
    fake.reply(200, { decision: "denied", lineage_id: "l", layer: "proposal", reason: "policy deny", actor: { kind: "human", id: "ada" }, policy: "deny", policy_version: "1" });
    expect(promoteSummary(await promoteAll(client, ["l"], "why"))).toBe("0 promoted, 1 denied (policy deny)");
  });

  it("Demote is refused visibly (not in the contract)", () => {
    expect(() => demote()).toThrow(DEMOTE_UNAVAILABLE);
    expect(fake.requests).toHaveLength(0);
  });
});

describe("the plugin writes no file (static scan of src/)", () => {
  const files: string[] = [];
  const walk = (d: string): void => {
    for (const f of readdirSync(d)) {
      const p = join(d, f);
      if (statSync(p).isDirectory()) walk(p);
      else if (p.endsWith(".ts")) files.push(p);
    }
  };
  walk(new URL("../src", import.meta.url).pathname);
  // method names, whatever the receiver is called (an alias such as
  // `const a = vault.adapter; a.write(...)` is caught), dotted or bracketed
  const writeMethods = [
    "modify", "modifyBinary", "create", "createBinary", "createFolder", "append", "process",
    "trash", "trashSystem", "trashLocal", "rename", "copy", "write", "writeBinary", "remove",
    "rmdir", "mkdir", "delete", "processFrontMatter", "saveData", "replaceRange", "replaceSelection", "setValue", "setLine", "changeByRange",
    "openLinkText", "writeFile", "appendFile", "createWriteStream",
  ].join("|");
  const forbidden = [
    new RegExp(`\\.\\s*(${writeMethods})\\s*\\(`),
    new RegExp(`\\[\\s*["'\`](${writeMethods})["'\`]\\s*\\]`),
    /\bfileManager\b/,
    /from\s+["'](node:)?fs(\/promises)?["']/,
    /\brequire\s*\(/,
    // a CodeMirror transaction that edits the document (`changes`, shorthand
    // included), or a change set built by hand
    /\bchanges\b/,
    /\bChangeSet\b/,
  ];  it("scans every source file", () => {
    expect(files.length).toBeGreaterThanOrEqual(10);
  });
  // a write method taken off its object (`x.write.bind(…)`, `const { write } = …`)
  // could be called under another name: only src/own-writes.ts may, to wrap
  // Obsidian's own saves (P2-0/Z3, ASSUMPTIONS Z3-1)
  const bound = new RegExp(`\\.\\s*(${writeMethods})\\s*\\.\\s*(bind|call|apply)\\s*\\(|\\{[^}]*\\b(${writeMethods})\\b[^}]*\\}\\s*=`);
  it.each(files.map((f) => [f.slice(f.indexOf("/src/") + 1)]))("%s calls no write API", (rel) => {
    const text = readFileSync(new URL(`../${rel}`, import.meta.url), "utf8");
    for (const re of forbidden) expect(text, `${rel} ${String(re)}`).not.toMatch(re);
    if (rel !== "src/own-writes.ts") expect(text, `${rel} ${String(bound)}`).not.toMatch(bound);
  });
  it("src/own-writes.ts: the bound copies are called only inside their wrappers, unchanged", () => {
    const text = readFileSync(new URL("../src/own-writes.ts", import.meta.url), "utf8");
    expect([...text.matchAll(/\.(\w+)\.bind\(adapter\)/g)].map((m) => m[1])).toEqual(["write", "append", "rename"]);
    // each bound copy is called once, with the wrapper's own arguments
    expect(text.match(/await write\(/g)?.length).toBe(1);
    expect(text).toContain("await write(path, data, options);");
    expect(text.match(/await append\(/g)?.length).toBe(1);
    expect(text).toContain("await append(path, data, options);");
    expect(text.match(/await rename\(/g)?.length).toBe(1);
    expect(text).toContain("await rename(from, to);");
  });
});
