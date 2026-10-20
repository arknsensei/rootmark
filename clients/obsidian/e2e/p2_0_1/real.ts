// P2-0.1 real-Obsidian sets (GOAL P2-0.1 D-A′, D-C′; TASKS Y3, Y5;
// ASSUMPTIONS Y0-5, Y0-8, Y0-10): a separate Obsidian instance (cdp.ts, own
// --user-data-dir) with the plugin installed, against
// `rootmark serve --http --watch` (W from --window), on a fresh copy of the
// mini vault.
//
// --set d_a_prime: per laundering scenario the human (in Obsidian) may first
// write a paragraph linking to the target (`human_type`, saved, waited until
// canonical), the agent appends a tagged paragraph to its note straight to
// the file (Node `fs`, outside Obsidian), then — after the watcher has
// ingested it, or `delay_ms` later for the "unobserved" kind — the human
// renames the target through Obsidian (`fileManager.renameFile`, links
// updated). After W + 1 s every block of the agent's note whose text holds
// the scenario's `Agent p<nnn>` tag is read: its layer (the measure: how many
// are `canonical`). Then the ten fake link updates: the agent (with the
// token: session, MAC) writes a note with a link redirected to another note
// AND a change beyond the link targets, and asks `attest` with
// `link_update_of`; the measure is the answer (403 `denied`) and that no
// block new to the note is `canonical` after W + 1 s.
//
// --set d_c_prime: per tick the human types a tagged paragraph at the end of
// a note and saves (the save command); once Obsidian's write of that note
// has finished (in the page), the harness waits `agent_delay_ms` and the
// agent appends its tagged paragraph straight to the file; after W + 0.5 s
// the layers of the tick's human and agent blocks are read.
//
//   node e2e/p2_0_1/.build/real.mjs --set d_a_prime|d_c_prime --data <json>
//     --rootmark <bin> --obsidian <bin> --plugin <dir> --vault-src <dir>
//     --window <s> --out <dir>
//
// <out>/<set>.json: one record per scenario/tick and totals; no token, port
// or absolute path. Fail-fast: an HTTP error other than the expected
// `denied`, a typed note Obsidian does not write, a setup paragraph that does
// not become canonical — an exception, never a partial record.
import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { createHash, createHmac } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { contentHash } from "../../src/blake3";
import { freePort } from "../harness";
import { launchObsidian, type Page } from "../real/cdp";

const arg = (name: string, dflt?: string): string => {
  const i = process.argv.indexOf(`--${name}`);
  const v = i >= 0 ? process.argv[i + 1] : dflt;
  if (v === undefined) throw new Error(`--${name} is required`);
  return v;
};
const set = arg("set");
if (set !== "d_a_prime" && set !== "d_c_prime") throw new Error(`--set ${set}: d_a_prime or d_c_prime`);
const rootmark = arg("rootmark");
const obsidian = arg("obsidian");
const plugin = arg("plugin");
const src = arg("vault-src");
const data = JSON.parse(readFileSync(arg("data"), "utf8")) as Record<string, unknown>;
const windowS = Number(arg("window"));
const out = arg("out");
mkdirSync(out, { recursive: true });

const sleep = (ms: number): Promise<void> => new Promise((ok) => setTimeout(ok, ms));
const J = (s: string): string => JSON.stringify(s);
const work = mkdtempSync(join(tmpdir(), "rootmark-p2-0-1-real-"));
const vault = join(work, "vault");
mkdirSync(vault);
for (const f of readdirSync(src).filter((f) => f.endsWith(".md"))) copyFileSync(join(src, f), join(vault, f));

const run = (args: string[]): string => {
  const r = spawnSync(rootmark, args, { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`rootmark ${args.join(" ")}: exit ${String(r.status)}\n${r.stderr}`);
  return r.stdout;
};
run(["ingest-dir", vault, vault]);
const httpPort = await freePort();
run(["serve", "--vault", vault, "--mcp", "stdio"]);
const policyPath = join(vault, ".rootmark/policy.toml");
let policy = readFileSync(policyPath, "utf8");
policy = policy.replace("[server]\n", `[server]\nhttp_port = ${String(httpPort)}\n`);
policy = policy.replace('# [identity]\n# human_id = "<your id>"\n', '[identity]\nhuman_id = "editor"\n');
// P2-A/F1: the default recipe writes `[attestation] unattested = "canonical"` since P2-A/A0; its table is removed before ours
policy = policy.replace('[attestation]\nunattested = "canonical"\n', "");
policy += `\n[attestation]\nunattested = "unverified"\nwindow_s = ${String(windowS)}\n`;
writeFileSync(policyPath, policy);
const pd = join(vault, ".obsidian/plugins/rootmark");
mkdirSync(pd, { recursive: true });
for (const f of ["main.js", "manifest.json", "styles.css"]) copyFileSync(join(plugin, f), join(pd, f));
const server: ChildProcess = spawn(rootmark, ["serve", "--vault", vault, "--http", "127.0.0.1", "--watch"], { stdio: ["ignore", "ignore", "pipe"] });
let serverErr = "";
server.stderr?.on("data", (d: Buffer) => (serverErr += d.toString("utf8")));
for (let i = 0; i < 600 && !serverErr.includes("watching"); i++) await sleep(100);
if (!serverErr.includes("watching")) throw new Error(`rootmark serve did not start:\n${serverErr}`);
const token = readFileSync(join(vault, ".rootmark/server.token"), "utf8").trim();

async function post(tool: string, body: unknown): Promise<{ status: number; body: Record<string, unknown> }> {
  const r = await fetch(`http://127.0.0.1:${String(httpPort)}/v1/${tool}`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}`, "X-Consumer": "p2-0-1-real", "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return { status: r.status, body: (await r.json()) as Record<string, unknown> };
}
async function ok(tool: string, body: unknown): Promise<Record<string, unknown>> {
  const r = await post(tool, body);
  if (r.status !== 200) throw new Error(`${tool}: HTTP ${String(r.status)} ${JSON.stringify(r.body)}`);
  return r.body;
}
interface Block {
  lineage_id: string;
  version_id: string;
  layer: string;
}
/** `(text, layer, version_id)` of every head block of `doc` (none: 404). */
async function blocks(doc: string): Promise<{ text: string; layer: string; version_id: string }[]> {
  const r = await post("blame", { doc_path: doc });
  if (r.status === 404) return [];
  if (r.status !== 200) throw new Error(`blame ${doc}: HTTP ${String(r.status)} ${JSON.stringify(r.body)}`);
  const res = [];
  for (const b of r.body["blocks"] as Block[]) {
    const t = await ok("read_block", { doc_path: doc, lineage_id: b.lineage_id });
    const text = ((t["block"] as Record<string, unknown>)["block"] as Record<string, unknown>)["text"] as string;
    res.push({ text, layer: b.layer, version_id: b.version_id });
  }
  return res;
}
/** The agent: a direct append to the file (one write, outside Obsidian). */
function agentAppend(doc: string, text: string): void {
  const p = join(vault, doc);
  writeFileSync(p, readFileSync(p, "utf8").replace(/\n*$/, "") + text);
}

// the person's hands in the Obsidian window (not the plugin): open, type,
// save and wait for Obsidian's write of the note, rename
const HANDS = `(window.__p21 = {
  writes: [],
  watch() {
    app.vault.setConfig('alwaysUpdateLinks', true);
    if (app.vault.getConfig('alwaysUpdateLinks') !== true) throw new Error('alwaysUpdateLinks not set');
    const a = app.vault.adapter;
    const w = a.write.bind(a);
    a.write = async (p, d, o) => { await w(p, d, o); if (p.endsWith('.md')) window.__p21.writes.push({ path: p, t: Date.now() }); };
    return true;
  },
  async open(path) {
    const f = app.vault.getAbstractFileByPath(path);
    if (!f) throw new Error(path + ' not in the vault');
    await app.workspace.getLeaf(false).openFile(f);
    for (let i = 0; i < 200; i++) {
      const ed = app.workspace.activeEditor;
      if (ed && ed.editor && ed.file && ed.file.path === path) return true;
      await new Promise((ok) => setTimeout(ok, 25));
    }
    throw new Error(path + ' did not become the active editor');
  },
  type(path, text) {
    const ed = app.workspace.activeEditor;
    const e = ed && ed.editor;
    if (!e || !ed.file || ed.file.path !== path) throw new Error('the active editor is not ' + path);
    const last = e.lastLine();
    e.replaceRange(text, { line: last, ch: e.getLine(last).length });
    return true;
  },
  async saveAndWait(path) {
    const n = window.__p21.writes.length;
    await app.commands.executeCommandById('editor:save-file');
    for (let i = 0; i < 400; i++) {
      const w = window.__p21.writes.slice(n).find((x) => x.path === path);
      if (w) return w.t;
      await new Promise((ok) => setTimeout(ok, 5));
    }
    throw new Error(path + ' was not written by Obsidian within 2 s of the save');
  },
  async rename(from, to) {
    const f = app.vault.getAbstractFileByPath(from);
    if (!f) throw new Error(from + ' not in the vault');
    await app.fileManager.renameFile(f, to);
    return true;
  },
  counts() {
    const it = [...document.querySelectorAll('.status-bar-item')].find((e) => (e.textContent || '').startsWith('attested '));
    return it ? it.textContent : null;
  },
}, true)`;

/** Types and saves; when Obsidian finished writing the note (ms, the
 * page's clock = this machine's). */
async function humanTypeAndSave(pg: Page, doc: string, text: string): Promise<number> {
  await pg.ev(`__p21.open(${J(doc)})`);
  await pg.ev(`__p21.type(${J(doc)}, ${J(text)})`);
  return pg.ev<number>(`__p21.saveAndWait(${J(doc)})`);
}

/** Waits until a block of `doc` holding `needle` has a layer in `want`. */
async function waitLayer(doc: string, needle: string, want: string[], ms: number): Promise<string | null> {
  const t0 = Date.now();
  let last: string | null = null;
  while (Date.now() - t0 < ms) {
    const b = (await blocks(doc)).find((x) => x.text.includes(needle));
    last = b?.layer ?? null;
    if (last !== null && want.includes(last)) return last;
    await sleep(50);
  }
  return last;
}

const stemOf = (p: string): string => (p.split("/").pop() ?? p).replace(/\.md$/, "");

/** A fake link update (Y0-5): `text` of `doc` with its first wikilink
 * redirected to `to` plus the variant's change beyond the link targets. */
function fake(variant: string, text: string, to: string, tag: string): string {
  const s = stemOf(to);
  const m = /\[\[([^\]|#]*)/.exec(text);
  if (m === null) throw new Error("fake: no wikilink");
  const redirected = text.slice(0, m.index) + "[[" + s + text.slice(m.index + m[0].length);
  const lines = redirected.split("\n");
  const plain = lines.findIndex((l) => /^[A-Za-z]/.test(l) && !l.includes("[["));
  const linkLine = lines.findIndex((l) => l.includes("[[" + s));
  const heading = lines.findIndex((l) => /^#{1,6} /.test(l));
  const need = (i: number, what: string): number => {
    if (i < 0) throw new Error(`fake ${variant}: no ${what}`);
    return i;
  };
  const line = (i: number): string => lines[i] ?? "";
  switch (variant) {
    case "append_paragraph":
      return redirected.replace(/\n*$/, "") + `\n\n${tag} appended paragraph.\n`;
    case "word_in_canonical": {
      const i = need(plain, "plain paragraph");
      lines[i] = line(i).replace(/^\S+/, tag.replace(" ", ""));
      return lines.join("\n");
    }
    case "link_label": {
      // the redirected link gets (another) label
      const esc = s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      return redirected.replace(new RegExp(`\\[\\[${esc}(#[^\\]|]*)?(\\|[^\\]]*)?\\]\\]`), `[[${s}$1|${tag} label]]`);
    }
    case "heading_text": {
      const i = need(heading, "heading");
      lines[i] = `${line(i)} ${tag}`;
      return lines.join("\n");
    }
    case "delete_paragraph": {
      const i = need(plain, "plain paragraph");
      lines.splice(i, 1);
      return lines.join("\n");
    }
    case "new_link": {
      const i = need(plain, "plain paragraph");
      lines[i] = `${line(i)} [[${s}]]`;
      return lines.join("\n");
    }
    case "list_item":
      return redirected.replace(/\n*$/, "") + `\n\n- ${tag} item\n`;
    case "code_block":
      return redirected.replace(/\n*$/, "") + `\n\n\`\`\`\n${tag} code\n\`\`\`\n`;
    case "target_and_text": {
      const i = need(linkLine, "link line");
      lines[i] = `${line(i)} ${tag} words.`;
      return lines.join("\n");
    }
    case "replace_with_agent": {
      const i = need(linkLine, "link line");
      lines[i] = `${tag} paragraph replacing the human one, see [[${s}]].`;
      return lines.join("\n");
    }
    default:
      throw new Error(`fake: unknown variant ${variant}`);
  }
}

/** The token holder's attestation (contract §2.6), as an agent with the
 * token would send it: session, MAC. */
async function tokenAttest(doc: string, bytes: Uint8Array, extra: Record<string, unknown>): Promise<{ status: number; body: Record<string, unknown> }> {
  const s = await ok("attest_session", {});
  const h = contentHash(bytes);
  const key = Buffer.from(s["key"] as string, "hex");
  const mac = createHmac("sha256", key).update(`rootmark-attest/1\n${doc}\n${h}\n`).digest("hex");
  return post("attest", { doc_path: doc, content_hash: h, session_id: s["session_id"], mac, ...extra });
}

interface ARec {
  id: string;
  kind: string;
  agent_blocks: number;
  agent_canonical: number;
  agent_layers: string[];
  link_rewritten: boolean;
  observed_before_rename: boolean | null;
}
interface CRec {
  id: string;
  doc: string;
  agent_delay_ms: number;
  actual_delay_ms: number;
  human_blocks: number;
  human_canonical: number;
  agent_blocks: number;
  agent_canonical: number;
}
interface Step {
  op: string;
  doc: string;
  text?: string;
  to?: string;
  delay_ms?: number | null;
}
const notes = (): string[] =>
  (readdirSync(vault, { recursive: true }) as string[])
    .map((f) => f.split("\\").join("/"))
    .filter((f) => f.endsWith(".md") && !f.startsWith(".obsidian") && !f.startsWith(".rootmark"))
    .sort();

let page: Page | null = null;
try {
  page = await launchObsidian(obsidian, join(work, "obsidian-data"), vault, await freePort());
  const pg = page;
  await pg.ev("app.plugins.setEnable(true).then(() => app.plugins.enablePlugin('rootmark')).then(() => true)");
  await pg.waitFor("!!app.plugins.plugins.rootmark", "the rootmark plugin to load");
  await pg.waitFor("(() => { const it = [...document.querySelectorAll('.status-bar-item')].find((e) => (e.textContent || '').startsWith('rootmark')); return it && it.textContent !== 'rootmark: offline'; })()", "the plugin to connect");
  await pg.ev(HANDS);
  await pg.ev("__p21.watch()");
  let res: Record<string, unknown>;
  if (set === "d_a_prime") {
    const scenarios = data["scenarios"] as { id: string; kind: string; agent_doc: string; target: string; steps: Step[] }[];
    const recs: ARec[] = [];
    for (const sc of scenarios) {
      const tag = `Agent ${sc.id}`;
      let observedBefore: boolean | null = null;
      for (const st of sc.steps) {
        if (st.op === "human_type") {
          await humanTypeAndSave(pg, st.doc, st.text ?? "");
          const l = await waitLayer(st.doc, `Human ${sc.id}`, ["canonical"], 5000);
          if (l !== "canonical") throw new Error(`${sc.id}: the human's linking paragraph is ${String(l)}, not canonical`);
        } else if (st.op === "agent_append") {
          agentAppend(st.doc, st.text ?? "");
        } else if (st.op === "rename") {
          if (st.delay_ms === null || st.delay_ms === undefined) {
            const l = await waitLayer(sc.agent_doc, tag, ["unverified", "canonical", "proposal"], 5000);
            if (l === null) throw new Error(`${sc.id}: the watcher did not ingest the agent's paragraph within 5 s`);
            observedBefore = true;
          } else {
            // nothing between the write and the rename but the delay (Y3
            // review: an observation here would lengthen it)
            await sleep(st.delay_ms);
            observedBefore = null;
          }
          await pg.ev(`__p21.rename(${J(st.doc)}, ${J(st.to ?? "")})`);
        } else throw new Error(`${sc.id}: unknown step ${st.op}`);
      }
      await sleep(windowS * 1000 + 1000);
      const bl = await blocks(sc.agent_doc);
      const agent = bl.filter((b) => b.text.includes(tag));
      const newName = sc.steps.find((s) => s.op === "rename")?.to ?? "";
      const text = readFileSync(join(vault, sc.agent_doc), "utf8");
      // Obsidian must have rewritten the link (else the scenario measures
      // nothing; Y3 review) — except in the "unobserved" kind, where the
      // rename starts 0–50 ms after the agent's write and Obsidian's own
      // metadata cache may not know the new link yet: then Obsidian leaves
      // it as written (a real outcome of the race the kind measures,
      // development RUN, Y6.md); it is recorded, not an error
      if (!text.includes(`[[${stemOf(newName)}`) && sc.kind !== "unobserved") throw new Error(`${sc.id}: Obsidian did not rewrite the link to ${newName} in ${sc.agent_doc}`);
      recs.push({
        id: sc.id,
        kind: sc.kind,
        agent_blocks: agent.length,
        agent_canonical: agent.filter((b) => b.layer === "canonical").length,
        agent_layers: agent.map((b) => b.layer),
        link_rewritten: text.includes(`[[${stemOf(newName)}`),
        observed_before_rename: observedBefore,
      });
      if (agent.length === 0) throw new Error(`${sc.id}: no block of the agent's note holds ${tag}`);
    }
    const fakes = data["fakes"] as { id: string; variant: string }[];
    const frecs = [];
    for (const [j, f] of fakes.entries()) {
      const tag = `Agent ${f.id}`;
      const all = notes();
      const withLink = all.filter((n) => /\[\[[^\]|#]+/.test(readFileSync(join(vault, n), "utf8")));
      const doc = withLink[j % withLink.length];
      if (doc === undefined) throw new Error("no note with a wikilink");
      const to = all.find((n) => n !== doc && !readFileSync(join(vault, doc), "utf8").includes(`[[${stemOf(n)}`));
      if (to === undefined) throw new Error(`${f.id}: no other note`);
      const before = new Set((await blocks(doc)).map((b) => b.version_id));
      const text = fake(f.variant, readFileSync(join(vault, doc), "utf8"), to, tag);
      writeFileSync(join(vault, doc), text);
      const r = await tokenAttest(doc, new TextEncoder().encode(text), { link_update_of: to });
      await sleep(windowS * 1000 + 1000);
      const fresh = (await blocks(doc)).filter((b) => !before.has(b.version_id));
      const code = ((r.body["error"] as Record<string, unknown> | undefined) ?? {})["code"] ?? null;
      const message = ((r.body["error"] as Record<string, unknown> | undefined) ?? {})["message"] ?? null;
      frecs.push({ id: f.id, variant: f.variant, status: r.status, code, message, denied: r.status === 403 && code === "denied", new_blocks: fresh.length, new_canonical: fresh.filter((b) => b.layer === "canonical").length });
    }
    const agentCanonical = recs.reduce((a, r) => a + r.agent_canonical, 0);
    const denied = frecs.filter((r) => r.denied && r.new_canonical === 0).length;
    res = {
      set: "D-A′",
      window_s: windowS,
      scenarios: recs.length,
      agent_blocks: recs.reduce((a, r) => a + r.agent_blocks, 0),
      agent_canonical: agentCanonical,
      by_kind: Object.fromEntries(["in_agent_block", "in_human_block", "unobserved"].map((k) => [k, { scenarios: recs.filter((r) => r.kind === k).length, agent_canonical: recs.filter((r) => r.kind === k).reduce((a, r) => a + r.agent_canonical, 0) }])),
      links_rewritten: recs.filter((r) => r.link_rewritten).length,
      fakes: frecs.length,
      fakes_denied: denied,
      pass: agentCanonical === 0 && recs.length === 50 && denied === 10 && frecs.length === 10,
      records: recs,
      fake_records: frecs,
    };
  } else {
    const ticks = data["ticks"] as { id: string; doc: string; human: string; agent: string; agent_delay_ms: number }[];
    const recs: CRec[] = [];
    for (const t of ticks) {
      const written = await humanTypeAndSave(pg, t.doc, t.human);
      const wait = written + t.agent_delay_ms - Date.now();
      if (wait > 0) await sleep(wait);
      agentAppend(t.doc, t.agent);
      // the delay that actually separated the two writes (Y3 review)
      const actual = Date.now() - written;
      await sleep(windowS * 1000 + 500);
      const bl = await blocks(t.doc);
      const human = bl.filter((b) => b.text.includes(`Human ${t.id}`));
      const agent = bl.filter((b) => b.text.includes(`Agent ${t.id}`));
      if (human.length === 0 || agent.length === 0) throw new Error(`${t.id}: the tick's blocks are not in ${t.doc}`);
      recs.push({
        id: t.id,
        doc: t.doc,
        agent_delay_ms: t.agent_delay_ms,
        actual_delay_ms: actual,
        human_blocks: human.length,
        human_canonical: human.filter((b) => b.layer === "canonical").length,
        agent_blocks: agent.length,
        agent_canonical: agent.filter((b) => b.layer === "canonical").length,
      });
    }
    const hb = recs.reduce((a, r) => a + r.human_blocks, 0);
    const hc = recs.reduce((a, r) => a + r.human_canonical, 0);
    const ac = recs.reduce((a, r) => a + r.agent_canonical, 0);
    res = {
      set: "D-C′",
      window_s: windowS,
      ticks: recs.length,
      human_blocks: hb,
      human_canonical: hc,
      human_accuracy: hb === 0 ? null : hc / hb,
      agent_blocks: recs.reduce((a, r) => a + r.agent_blocks, 0),
      agent_canonical: ac,
      by_delay: Object.fromEntries([0, 20, 50, 100].map((d) => [String(d), { ticks: recs.filter((r) => r.agent_delay_ms === d).length, human_blocks: recs.filter((r) => r.agent_delay_ms === d).reduce((a, r) => a + r.human_blocks, 0), human_canonical: recs.filter((r) => r.agent_delay_ms === d).reduce((a, r) => a + r.human_canonical, 0), agent_canonical: recs.filter((r) => r.agent_delay_ms === d).reduce((a, r) => a + r.agent_canonical, 0) }])),
      pass: hb > 0 && hc / hb >= 0.95 && ac === 0 && recs.length === 40,
      records: recs,
    };
  }
  const attPath = join(vault, ".rootmark/attestations.jsonl");
  res["attestation_lines"] = existsSync(attPath) ? readFileSync(attPath, "utf8").split("\n").filter((l) => l !== "").length : 0;
  res["plugin_counts"] = await pg.ev<string | null>("__p21.counts()");
  res["events_sha256"] = createHash("sha256").update(readFileSync(join(vault, ".rootmark/events.jsonl"))).digest("hex");
  // the vault's logs, for the report (the cause of any false unverified)
  copyFileSync(join(vault, ".rootmark/events.jsonl"), join(out, "events.jsonl"));
  if (existsSync(attPath)) copyFileSync(attPath, join(out, "attestations.jsonl"));
  else writeFileSync(join(out, "attestations.jsonl"), "");
  writeFileSync(join(out, `${set}.json`), JSON.stringify(res, null, 2) + "\n");
  console.log(JSON.stringify({ ...res, records: undefined, fake_records: undefined }));
} finally {
  if (page !== null) await page.close();
  server.kill("SIGTERM");
}
