// P2-0 real-Obsidian sessions (GOAL P2-0 D-B; TASKS Z3, Z5): human edit
// sessions in a separate Obsidian instance (cdp.ts, own --user-data-dir)
// with the plugin installed, against `rootmark serve --http --watch`. Each
// session's steps are typed into Obsidian's editor (the person), saved the
// way a person saves (the save command, Obsidian's autosave, a focus loss),
// renamed through Obsidian; the plugin attests every save (contract §2.6).
// For every save the harness records when Obsidian wrote the file (in the
// page), polls `blame` until the new blocks are `canonical` (latency), and
// after the window counts the new blocks still `unverified` (false
// unverified, H-P2-0b; late raises, H-P2-0b′).
//
//   node e2e/p2_0/.build/sessions.mjs --rootmark <bin> --obsidian <bin>
//     --plugin <dir> --vault-src <dir of .md> --scenarios <json> --window <s>
//     --out <dir> [--server-policy-extra <toml>] [--ui true]
//
// With `--ui true`, after the sessions: an agent-like direct write to a note
// (outside Obsidian), then in Obsidian the gutter's `U` mark, the proposal
// view's "Show unverified" tab and one "Mine" click (contract §2.6 UI;
// P2-0/Z3). The latency of a save is measured from Obsidian's write of its
// last save of the document in the session to the first `blame` where every
// new block is `canonical`; a document without a captured write has no
// latency (`no_write`).
//
// <scenarios>: {"sessions": [{"id", "kind", "steps": [{"op": "type", "doc",
// "text"} | {"op": "save", "how": "command"|"autosave"|"blur"} | {"op":
// "rename", "doc", "to"} | {"op": "open", "doc"}]}]}.
// <out>/sessions.json: one record per save (session, doc, new blocks,
// unverified after the window, late raises, latency ms) and totals; the
// plugin's counters; no token, port or absolute path.
import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { freePort } from "../harness";
import { launchObsidian, type Page } from "../real/cdp";

const arg = (name: string, dflt?: string): string => {
  const i = process.argv.indexOf(`--${name}`);
  const v = i >= 0 ? process.argv[i + 1] : dflt;
  if (v === undefined) throw new Error(`--${name} is required`);
  return v;
};
const rootmark = arg("rootmark");
const obsidian = arg("obsidian");
const plugin = arg("plugin");
const src = arg("vault-src");
const scenarios = JSON.parse(readFileSync(arg("scenarios"), "utf8")) as { sessions: Session[] };
const windowS = Number(arg("window"));
const out = arg("out");
mkdirSync(out, { recursive: true });

interface Step {
  op: "type" | "save" | "rename" | "open";
  doc?: string;
  text?: string;
  how?: "command" | "autosave" | "blur";
  to?: string;
}
interface Session {
  id: string;
  kind: string;
  steps: Step[];
}
interface Block {
  lineage_id: string;
  version_id: string;
  layer: string;
}

const sleep = (ms: number): Promise<void> => new Promise((ok) => setTimeout(ok, ms));
const J = (s: string): string => JSON.stringify(s);
const work = mkdtempSync(join(tmpdir(), "rootmark-p2-0-sessions-"));
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
policy = policy.replace("# [identity]\n# human_id = \"<your id>\"\n", '[identity]\nhuman_id = "editor"\n');
policy += `\n[attestation]\nunattested = "unverified"\nwindow_s = ${String(windowS)}\n${arg("server-policy-extra", "")}`;
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

async function call(tool: string, body: unknown): Promise<Record<string, unknown>> {
  const r = await fetch(`http://127.0.0.1:${String(httpPort)}/v1/${tool}`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}`, "X-Consumer": "p2-0-sessions", "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return (await r.json()) as Record<string, unknown>;
}
async function blame(doc: string): Promise<Block[] | null> {
  const r = await call("blame", { doc_path: doc });
  return Array.isArray(r["blocks"]) ? (r["blocks"] as Block[]) : null;
}

// the person's hands, in the Obsidian window (not the plugin)
const HANDS = `(window.__p2 = {
  writes: [],
  watch() {
    const a = app.vault.adapter;
    const w = a.write.bind(a);
    a.write = async (p, d, o) => { await w(p, d, o); if (p.endsWith('.md')) window.__p2.writes.push({ path: p, t: Date.now() }); };
    return true;
  },
  async open(path) {
    const f = app.vault.getAbstractFileByPath(path);
    if (!f) throw new Error(path + ' not in the vault');
    await app.workspace.getLeaf(false).openFile(f);
    return true;
  },
  type(text) {
    const e = app.workspace.activeEditor && app.workspace.activeEditor.editor;
    if (!e) throw new Error('no active editor');
    const last = e.lastLine();
    e.replaceRange(text, { line: last, ch: e.getLine(last).length });
    return true;
  },
  save() { return app.commands.executeCommandById('editor:save-file'); },
  blur() { window.dispatchEvent(new Event('blur')); document.dispatchEvent(new Event('visibilitychange')); return true; },
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

/** The unverified UI in the real Obsidian (Z3): an agent-like direct write,
 * the gutter's U mark, the "Show unverified" tab, one "Mine" click. */
async function uiCheck(pg: Page): Promise<Record<string, unknown>> {
  const doc = "roasting.md";
  const p = join(vault, doc);
  writeFileSync(p, readFileSync(p, "utf8") + "\n\nAn agent wrote this line straight to the file.\n");
  let unverified = false;
  for (let i = 0; i < 80 && !unverified; i++) {
    await sleep(100);
    unverified = ((await blame(doc)) ?? []).some((b) => b.layer === "unverified");
  }
  await pg.ev(`__p2.open(${J(doc)})`);
  // Obsidian reloads the note from disk; the gutter aligns once its block
  // map has the new paragraph
  let marks: string[] = [];
  for (let i = 0; i < 40 && !marks.includes("U"); i++) {
    await sleep(250);
    await pg.ev("app.commands.executeCommandById('rootmark:blame')");
    // CodeMirror renders the gutter of visible lines only: show the end
    await pg.ev("(() => { const e = app.workspace.activeEditor && app.workspace.activeEditor.editor; if (e) { const l = e.lastLine(); e.setCursor({ line: l, ch: 0 }); e.scrollIntoView({ from: { line: l, ch: 0 }, to: { line: l, ch: 0 } }, true); } return true; })()");
    await sleep(150);
    marks = await pg.ev<string[]>("[...(app.workspace.activeLeaf?.view?.containerEl ?? document).querySelectorAll('.rootmark-gutter .rootmark-mark')].map((m) => m.textContent)");
  }
  const gutterHint = await pg.ev<string | null>("(() => { const it = [...document.querySelectorAll('.status-bar-item')].find((e) => (e.textContent || '').startsWith('rootmark')); return it ? it.getAttribute('title') : null; })()");
  await pg.ev("app.commands.executeCommandById('rootmark:proposals')");
  await pg.waitFor("!!document.querySelector('.workspace-leaf-content[data-type=\"rootmark-proposals\"] table.rootmark-proposals')", "the proposal table");
  await pg.ev("(() => { const v = document.querySelector('.workspace-leaf-content[data-type=\"rootmark-proposals\"] .view-content'); [...v.querySelectorAll('button')].find((b) => b.textContent === 'Show unverified').click(); return true; })()");
  await pg.waitFor("document.querySelectorAll('.workspace-leaf-content[data-type=\"rootmark-proposals\"] tr.rootmark-layer-unverified').length > 0", "the unverified rows");
  const rows = await pg.ev<number>("document.querySelectorAll('.workspace-leaf-content[data-type=\"rootmark-proposals\"] tr.rootmark-layer-unverified').length");
  await pg.ev("(() => { const tr = document.querySelector('.workspace-leaf-content[data-type=\"rootmark-proposals\"] tr.rootmark-layer-unverified'); [...tr.querySelectorAll('button')].find((b) => b.textContent === 'Mine').click(); return true; })()");
  let promoted = false;
  for (let i = 0; i < 50 && !promoted; i++) {
    await sleep(100);
    promoted = !((await blame(doc)) ?? []).some((b) => b.layer === "unverified");
  }
  const evs = readFileSync(join(vault, ".rootmark/events.jsonl"), "utf8").split("\n").filter((l) => l !== "").map((l) => JSON.parse(l) as { kind: string; meta: Record<string, unknown> });
  const last = evs.filter((e) => e.kind === "Promoted").pop();
  const res = {
    agent_write_unverified: unverified,
    gutter_has_U: marks.includes("U"),
    gutter_hint: marks.includes("U") ? null : gutterHint,
    unverified_rows: rows,
    mine_promoted: promoted,
    promoted_from: last?.meta["from"] ?? null,
  };
  return { ...res, pass: unverified && marks.includes("U") && rows === 1 && promoted && last?.meta["from"] === "unverified" };
}

interface SaveRecord {
  session: string;
  kind: string;
  doc: string;
  new_blocks: number;
  unverified_after_window: number;
  raised_late: boolean;
  latency_ms: number | null;
  /** no write of this document was captured in the page: not a save */
  no_write: boolean;
}
const records: SaveRecord[] = [];
let page: Page | null = null;
try {
  page = await launchObsidian(obsidian, join(work, "obsidian-data"), vault, await freePort());
  const pg = page;
  await pg.ev("app.plugins.setEnable(true).then(() => app.plugins.enablePlugin('rootmark')).then(() => true)");
  await pg.waitFor("!!app.plugins.plugins.rootmark", "the rootmark plugin to load");
  await pg.waitFor("(() => { const it = [...document.querySelectorAll('.status-bar-item')].find((e) => (e.textContent || '').startsWith('rootmark')); return it && it.textContent !== 'rootmark: offline'; })()", "the plugin to connect");
  await pg.ev(HANDS);
  await pg.ev("__p2.watch()");

  for (const s of scenarios.sessions) {
    const docs = new Set<string>();
    const before = new Map<string, Set<string>>();
    const startWrites = await pg.ev<number>("__p2.writes.length");
    for (const st of s.steps) {
      if (st.op === "open" || st.op === "type") {
        const doc = st.doc ?? "";
        if (!before.has(doc)) before.set(doc, new Set(((await blame(doc)) ?? []).map((b) => b.version_id)));
        docs.add(doc);
        await pg.ev(`__p2.open(${J(doc)})`);
        if (st.op === "type") await pg.ev(`__p2.type(${J(st.text ?? "")})`);
      } else if (st.op === "save") {
        if (st.how === "autosave") await sleep(2600);
        else if (st.how === "blur") {
          await pg.ev("__p2.blur()");
          await sleep(400);
        } else await pg.ev("__p2.save()");
      } else {
        const from = st.doc ?? "";
        const to = st.to ?? "";
        await pg.ev(`__p2.rename(${J(from)}, ${J(to)})`);
        before.set(to, new Set());
        docs.delete(from);
        docs.add(to);
      }
    }
    // the writes of this session; then the server's view of each document
    await sleep(150);
    const writes = await pg.ev<{ path: string; t: number }[]>(`__p2.writes.slice(${String(startWrites)})`);
    for (const doc of docs) {
      const w = writes.filter((x) => x.path === doc).pop();
      const t0 = w?.t ?? Date.now();
      const noWrite = w === undefined;
      const old = before.get(doc) ?? new Set<string>();
      let latency: number | null = null;
      let seenUnverified = false;
      let last: Block[] = [];
      const deadline = t0 + windowS * 1000 + 1500;
      while (Date.now() < deadline) {
        const bl = await blame(doc);
        if (bl !== null) {
          last = bl;
          const fresh = bl.filter((b) => !old.has(b.version_id));
          if (fresh.some((b) => b.layer === "unverified")) seenUnverified = true;
          if (fresh.length > 0 && fresh.every((b) => b.layer === "canonical")) {
            latency = Date.now() - t0;
            break;
          }
        }
        await sleep(25);
      }
      if (latency === null) {
        // the window is over: what stays unverified is a false unverified
        const bl = await blame(doc);
        if (bl !== null) last = bl;
      }
      const fresh = last.filter((b) => !old.has(b.version_id));
      records.push({
        session: s.id,
        kind: s.kind,
        doc,
        new_blocks: fresh.length,
        unverified_after_window: fresh.filter((b) => b.layer === "unverified").length,
        raised_late: seenUnverified && latency !== null,
        latency_ms: noWrite ? null : latency,
        no_write: noWrite,
      });
    }
  }
  let ui: Record<string, unknown> | null = null;
  if (arg("ui", "false") === "true") ui = await uiCheck(pg);
  const counts = await pg.ev<string | null>("__p2.counts()");
  const diag = await pg.ev("({ notices: [...document.querySelectorAll('.notice')].map((n) => n.textContent), bar: [...document.querySelectorAll('.status-bar-item')].map((e) => e.textContent + ' | ' + e.getAttribute('title')) })");
  const attPath = join(vault, ".rootmark/attestations.jsonl");
  const att = existsSync(attPath) ? readFileSync(attPath, "utf8").split("\n").filter((l) => l !== "").length : 0;
  const blocks = records.reduce((a, r) => a + r.new_blocks, 0);
  const unverified = records.reduce((a, r) => a + r.unverified_after_window, 0);
  const lat = records.map((r) => r.latency_ms).filter((x): x is number => x !== null).sort((a, b) => a - b);
  const p95 = lat.length === 0 ? null : (lat[Math.min(lat.length - 1, Math.ceil(lat.length * 0.95) - 1)] ?? null);
  const res = {
    window_s: windowS,
    sessions: scenarios.sessions.length,
    saves: records.length,
    new_blocks: blocks,
    unverified_after_window: unverified,
    false_unverified_rate: blocks === 0 ? null : unverified / blocks,
    raised_late: records.filter((r) => r.raised_late).length,
    latency_p95_ms: p95,
    latency_measured: lat.length,
    plugin_counts: counts,
    no_write: records.filter((r) => r.no_write).length,
    ui,
    plugin_diagnostics: att === 0 ? diag : undefined,
    attestation_lines: att,
    records,
  };
  writeFileSync(join(out, "sessions.json"), JSON.stringify(res, null, 2) + "\n");
  console.log(JSON.stringify({ ...res, records: undefined }));
} finally {
  if (page !== null) await page.close();
  server.kill("SIGTERM");
}
