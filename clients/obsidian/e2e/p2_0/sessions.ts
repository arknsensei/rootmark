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
// P2-0/Z3). While a session runs, `blame` of its documents is polled every
// 25 ms (a log of observations). The latency of a save is measured from the
// end of Obsidian's write of its last save of the document in the session to
// the first observation, among those that answered after the write began,
// whose log position (`seq_last`) is past the one observed before the write
// began and where every new block is `canonical`; a document without a
// captured write has no latency (`no_write`); a written save that never
// became canonical counts as an infinite latency in the p95. A typed note
// that Obsidian does not write, or an HTTP error, stops the run. Notes the
// session did not edit but Obsidian rewrote (the links of a renamed note)
// are measured too (`side`).
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
  seq_last: number;
}
/** one `blame` of a document: when it was asked, when it answered */
interface Obs {
  asked: number;
  answered: number;
  seq: number;
  canonical: boolean;
  unverified: boolean;
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

async function post(tool: string, body: unknown): Promise<{ status: number; body: Record<string, unknown> }> {
  const r = await fetch(`http://127.0.0.1:${String(httpPort)}/v1/${tool}`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}`, "X-Consumer": "p2-0-sessions", "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return { status: r.status, body: (await r.json()) as Record<string, unknown> };
}
/** `blame` of a document; null only for a document the server does not
 * know (404). Any other failure is an error (no silent drop). */
async function blame(doc: string): Promise<Block[] | null> {
  const r = await post("blame", { doc_path: doc });
  if (r.status === 404) return null;
  if (r.status !== 200 || !Array.isArray(r.body["blocks"])) throw new Error(`blame ${doc}: HTTP ${String(r.status)} ${JSON.stringify(r.body)}`);
  return r.body["blocks"] as Block[];
}
/** The vault's notes (outside `.obsidian/` and `.rootmark/`) and their bytes. */
function notes(): Map<string, string> {
  const m = new Map<string, string>();
  for (const f of readdirSync(vault, { recursive: true }) as string[]) {
    if (!f.endsWith(".md") || f.startsWith(".obsidian") || f.startsWith(".rootmark")) continue;
    m.set(f.split("\\").join("/"), readFileSync(join(vault, f), "latin1"));
  }
  return m;
}

// the person's hands, in the Obsidian window (not the plugin)
const HANDS = `(window.__p2 = {
  writes: [],
  watch() {
    // a person who renames notes lets Obsidian update the links to them
    // (otherwise Obsidian asks in a modal and the rename waits for it)
    app.vault.setConfig('alwaysUpdateLinks', true);
    if (app.vault.getConfig('alwaysUpdateLinks') !== true) throw new Error('alwaysUpdateLinks not set');
    const a = app.vault.adapter;
    const w = a.write.bind(a);
    a.write = async (p, d, o) => { const ts = Date.now(); await w(p, d, o); if (p.endsWith('.md')) window.__p2.writes.push({ path: p, ts, t: Date.now() }); };
    return true;
  },
  async open(path) {
    const f = app.vault.getAbstractFileByPath(path);
    if (!f) throw new Error(path + ' not in the vault');
    await app.workspace.getLeaf(false).openFile(f);
    // a person types once the note is in front of them: wait until the
    // active editor shows this note
    for (let i = 0; i < 200; i++) {
      const ed = app.workspace.activeEditor;
      if (ed && ed.editor && ed.file && ed.file.path === path) return true;
      await new Promise((ok) => setTimeout(ok, 25));
    }
    throw new Error(path + ' did not become the active editor');
  },
  type(text) {
    const ed = app.workspace.activeEditor;
    const e = ed && ed.editor;
    if (!e) throw new Error('no active editor');
    if (!ed.file || ed.file.path !== window.__p2.current) throw new Error('the active editor is not ' + window.__p2.current);
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
  /** a note the session did not edit but Obsidian rewrote (the links of a
   * renamed note; ASSUMPTIONS Z5-5): measured, no latency */
  side: boolean;
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
    const obs = new Map<string, Obs[]>();
    const observe = async (doc: string): Promise<Block[] | null> => {
      const asked = Date.now();
      const bl = await blame(doc);
      if (bl !== null) {
        const old = before.get(doc) ?? new Set<string>();
        const fresh = bl.filter((b) => !old.has(b.version_id));
        const seq = bl.reduce((m, b) => Math.max(m, b.seq_last), -1);
        const list = obs.get(doc) ?? [];
        list.push({ asked, answered: Date.now(), seq, canonical: fresh.length > 0 && fresh.every((b) => b.layer === "canonical"), unverified: fresh.some((b) => b.layer === "unverified") });
        obs.set(doc, list);
      }
      return bl;
    };
    const polling = { on: true };
    const poller = (async () => {
      while (polling.on) {
        for (const d of [...docs]) await observe(d);
        await sleep(25);
      }
    })();
    const startWrites = await pg.ev<number>("__p2.writes.length");
    // when each document was last typed into: a save on focus loss is
    // Obsidian's own (delayed) save, so the session ends once every typed
    // document has been written after its last keystroke (at most 6 s)
    const typed = new Map<string, number>();
    // every note's bytes before the session; with a rename, also every
    // note's blocks (Obsidian rewrites the notes that link to it)
    const bytesBefore = notes();
    const renames = s.steps.some((st) => st.op === "rename");
    const allBefore = new Map<string, Set<string>>();
    if (renames) for (const d of bytesBefore.keys()) allBefore.set(d, new Set(((await blame(d)) ?? []).map((b) => b.version_id)));
    const renamedFrom = new Set<string>();
    for (const st of s.steps) {
      if (st.op === "open" || st.op === "type") {
        const doc = st.doc ?? "";
        if (!before.has(doc)) before.set(doc, new Set(((await blame(doc)) ?? []).map((b) => b.version_id)));
        docs.add(doc);
        await pg.ev(`__p2.open(${J(doc)})`);
        await pg.ev(`(window.__p2.current = ${J(doc)}, true)`);
        if (st.op === "type") {
          typed.set(doc, Date.now());
          await pg.ev(`__p2.type(${J(st.text ?? "")})`);
        }
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
        renamedFrom.add(from);
        before.set(to, new Set());
        docs.delete(from);
        docs.add(to);
      }
    }
    // the writes of this session; then the server's view of each document
    const written = async (): Promise<boolean> => {
      const ws = await pg.ev<{ path: string; t: number }[]>(`__p2.writes.slice(${String(startWrites)})`);
      return [...typed].every(([d, t]) => ws.some((x) => x.path === d && x.t >= t));
    };
    for (let i = 0; i < 240 && !(await written()); i++) await sleep(25);
    if (!(await written())) {
      const ws = await pg.ev<{ path: string; t: number }[]>(`__p2.writes.slice(${String(startWrites)})`);
      const missing = [...typed].filter(([d, t]) => !ws.some((x) => x.path === d && x.t >= t)).map(([d]) => d);
      throw new Error(`session ${s.id}: typed notes not written by Obsidian within 6 s: ${missing.join(", ")}; writes ${JSON.stringify(ws.map((x) => x.path))}`);
    }
    await sleep(150);
    polling.on = false;
    await poller;
    // notes Obsidian rewrote without the session editing them
    const side = new Set<string>();
    for (const [d, b] of notes()) {
      if (docs.has(d) || bytesBefore.get(d) === b) continue;
      const old = allBefore.get(d);
      if (old === undefined) throw new Error(`session ${s.id}: ${d} changed outside the session's notes`);
      before.set(d, old);
      side.add(d);
    }
    for (const d of renamedFrom) docs.delete(d);
    const writes = await pg.ev<{ path: string; ts: number; t: number }[]>(`__p2.writes.slice(${String(startWrites)})`);
    for (const doc of [...docs, ...side]) {
      const w = writes.filter((x) => x.path === doc).pop();
      const t0 = w?.t ?? Date.now();
      const noWrite = w === undefined;
      const old = before.get(doc) ?? new Set<string>();
      // the log position the server had before the last write began
      const s0 = (obs.get(doc) ?? []).filter((o) => o.answered <= (w?.ts ?? t0)).reduce((m, o) => Math.max(m, o.seq), -1);
      const settled = (): Obs | undefined => (obs.get(doc) ?? []).find((o) => o.asked >= (w?.ts ?? t0) && o.seq > s0 && o.canonical);
      let latency: number | null = null;
      let last: Block[] = [];
      const deadline = t0 + windowS * 1000 + 1500;
      for (;;) {
        const hit = settled();
        if (hit !== undefined) {
          latency = Math.max(0, hit.answered - t0);
          break;
        }
        if (Date.now() >= deadline) break;
        await observe(doc);
        await sleep(25);
      }
      // seen unverified after the write began, then canonical: a late raise
      const seenUnverified = (obs.get(doc) ?? []).some((o) => o.asked >= (w?.ts ?? t0) && o.unverified);
      // the server's view now (after the window, when it never settled)
      const bl = await blame(doc);
      if (bl !== null) last = bl;
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
        side: side.has(doc),
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
  // a written save that never became canonical within W + 1.5 s counts as
  // an infinite latency (it is not dropped); a p95 that lands on one is null
  // with `latency_p95_over_window`
  const written_ = records.filter((r) => !r.no_write);
  const lat = written_.map((r) => r.latency_ms ?? Number.POSITIVE_INFINITY).sort((a, b) => a - b);
  const p95raw = lat.length === 0 ? null : (lat[Math.min(lat.length - 1, Math.ceil(lat.length * 0.95) - 1)] ?? null);
  const p95 = p95raw === null || !Number.isFinite(p95raw) ? null : p95raw;
  const res = {
    window_s: windowS,
    sessions: scenarios.sessions.length,
    saves: records.length,
    new_blocks: blocks,
    unverified_after_window: unverified,
    false_unverified_rate: blocks === 0 ? null : unverified / blocks,
    raised_late: records.filter((r) => r.raised_late).length,
    latency_p95_ms: p95,
    latency_p95_over_window: p95raw !== null && !Number.isFinite(p95raw),
    latency_measured: lat.length,
    latency_unsettled: written_.filter((r) => r.latency_ms === null).length,
    side_notes: records.filter((r) => r.side).length,
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
