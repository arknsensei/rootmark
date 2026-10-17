// The real-Obsidian e2e (K-E1.3; GOAL P1-E.1 E1-3, E1-9): the plugin bundle
// (main.js, manifest.json, styles.css) installed in a fresh vault, a separate
// Obsidian instance (cdp.ts) driving the plugin's own panels, buttons and
// modal; the agent over MCP in the same `rootmark serve` process (contract §1);
// a recording proxy between the plugin and the server (E-4) and a Markdown
// sha256 per step (E-3). Then the connection screen on a vault whose index
// was written by the p1d-final `mdk` (`/2`, auto_rebuild = false): the
// status bar names the state and the way to rebuild, and after `rootmark index
// build` + Reconnect search works (E1-9).
//
//   node e2e/real/.build/run.mjs --rootmark <rootmark> --obsidian <Obsidian binary>
//     --vault-src <dir of .md> --old-index <fixture vault> --plugin <dir with
//     main.js, manifest.json, styles.css> --out <dir>
//
// <out>/plugin_e2e_real.json: the 11 steps and the connection step (shows,
// pass), E-3, E-4; no time stamp, port, token or absolute path.
import { spawnSync } from "node:child_process";
import { copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { TOOLS } from "../../src/api";
import { CONSUMER } from "../../src/consumer";
import { changed, freePort, mdSha, recordingProxy, serve, type Served } from "../harness";
import { launchObsidian, PAGE_HELPERS, type Page } from "./cdp";

const arg = (name: string): string => {
  const i = process.argv.indexOf(`--${name}`);
  const v = i >= 0 ? process.argv[i + 1] : undefined;
  if (v === undefined) throw new Error(`--${name} is required`);
  return v;
};
const rootmark = arg("rootmark");
const obsidian = arg("obsidian");
const src = arg("vault-src");
const oldIndex = arg("old-index");
const plugin = arg("plugin");
const out = arg("out");
mkdirSync(out, { recursive: true });

const DOC = "agent/scene.md";
const TEXT = "# Grind notes\n\nAn agent recommends a coarser grind for the kalimba roast.\n";
const AGENT = { name: "real-agent", version: "1.0" };
const REASON = "approved in Obsidian";
const sleep = (ms: number): Promise<void> => new Promise((ok) => setTimeout(ok, ms));
const work = mkdtempSync(join(tmpdir(), "rootmark-real-e2e-"));

const run = (args: string[]): string => {
  const r = spawnSync(rootmark, args, { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`rootmark ${args.join(" ")}: exit ${String(r.status)}\n${r.stderr}`);
  return r.stdout;
};

/** A vault with the plugin installed and the policy pointing the plugin at
 * `proxyPort` (`[server] http_port`), human id `editor`. */
function prepare(vault: string, proxyPort: number, extraIndex: string): void {
  if (!existsSync(join(vault, ".rootmark/policy.toml"))) run(["serve", "--vault", vault, "--mcp", "stdio"]);
  const p = join(vault, ".rootmark/policy.toml");
  let policy = readFileSync(p, "utf8");
  policy = policy.replace("[server]\n", `[server]\nhttp_port = ${String(proxyPort)}\n`);
  policy = policy.replace("# [identity]\n# human_id = \"<your id>\"\n", '[identity]\nhuman_id = "editor"\n');
  policy = policy.replace("[index]\n", `[index]\n${extraIndex}`);
  if (!policy.includes(`http_port = ${String(proxyPort)}`) || !policy.includes('human_id = "editor"')) {
    throw new Error("could not set [server] http_port and [identity] human_id in the default recipe");
  }
  writeFileSync(p, policy);
  const pd = join(vault, ".obsidian/plugins/rootmark");
  mkdirSync(pd, { recursive: true });
  for (const f of ["main.js", "manifest.json", "styles.css"]) copyFileSync(join(plugin, f), join(pd, f));
}

/** Leaves restricted mode through the API (the verifier's way: clicking the
 * trust dialog over CDP left the throw-away instance without windows) and
 * waits for the plugin's status bar item. */
async function enablePlugin(page: Page): Promise<void> {
  await page.ev("app.plugins.setEnable(true).then(() => app.plugins.enablePlugin('rootmark')).then(() => true)");
  await page.waitFor("!!app.plugins.plugins.rootmark", "the rootmark plugin to load");
  await page.ev(PAGE_HELPERS);
  await page.waitFor("(() => { const s = __rootmark.statusBar(); return s && s.text !== 'rootmark: offline' || false; })()", "the plugin to connect");
}

type Step = { step: number | string; actor: string; command: string; shows: unknown; pass: boolean };
const steps: Step[] = [];
const structured = (r: Record<string, unknown>): Record<string, unknown> => (r["structuredContent"] ?? {}) as Record<string, unknown>;
const J = (s: string): string => JSON.stringify(s);

/** the server's own answers (direct, not through the proxy: the access log
 * holds only the plugin's requests) */
async function direct(port: number, token: string, tool: string, body: unknown): Promise<Record<string, unknown>> {
  const r = await fetch(`http://127.0.0.1:${String(port)}/v1/${tool}`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}`, "X-Consumer": "real-e2e-check", "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return (await r.json()) as Record<string, unknown>;
}

const events = (vault: string): { kind: string; meta: Record<string, unknown>; lineage_ids: string[] }[] =>
  readFileSync(join(vault, ".rootmark/events.jsonl"), "utf8").split("\n").filter((l) => l !== "").map((l) => JSON.parse(l) as never);

// ---- the scene: 11 steps -------------------------------------------------------
const vault = join(work, "vault");
mkdirSync(vault);
for (const f of readdirSync(src).filter((f) => f.endsWith(".md"))) copyFileSync(join(src, f), join(vault, f));
run(["ingest-dir", vault, vault]);
const proxyPort = await freePort();
const serverPort = await freePort();
prepare(vault, proxyPort, "");
const shas = [mdSha(vault)];
const snap = (): void => {
  shas.push(mdSha(vault));
};
let served: Served | null = null;
let proxy: Awaited<ReturnType<typeof recordingProxy>> | null = null;
let page: Page | null = null;
let obsidianExit: number | null = null;
try {
  served = await serve(rootmark, vault, serverPort, AGENT);
  proxy = await recordingProxy(proxyPort, serverPort);
  const token = readFileSync(join(vault, ".rootmark/server.token"), "utf8").trim();
  page = await launchObsidian(obsidian, join(work, "obsidian-data"), vault, await freePort());
  const pg = page;
  await enablePlugin(pg);
  const cmd = (id: string): Promise<unknown> => pg.ev(`app.commands.executeCommandById(${J("rootmark:" + id)})`);

  // 1. the connection status
  const sb1 = await pg.ev<{ text: string; title: string }>("__rootmark.statusBar()");
  snap();
  steps.push({ step: 1, actor: "human", command: "status bar", shows: { text: sb1.text }, pass: sb1.text === "rootmark 0.1.0 · human:editor" });

  // 2. checkpoint panel: name the state "before"
  await cmd("checkpoints");
  await pg.waitFor("!!document.querySelector('.workspace-leaf-content[data-type=\"rootmark-checkpoints\"] .setting-item')", "the checkpoint panel");
  await pg.ev(`(__rootmark.field('rootmark-checkpoints', 'Name the current state', 'before'), __rootmark.button(__rootmark.setting('rootmark-checkpoints', 'Name the current state'), 'Name').click(), true)`);
  await pg.waitFor("[...__rootmark.view('rootmark-checkpoints').querySelectorAll('li')].some((l) => l.textContent.startsWith('before ·'))", "the named checkpoint in the list");
  const cps = (await direct(serverPort, token, "checkpoints", {}))["checkpoints"] as { checkpoint_id: string; name: string | null }[];
  const before = cps.filter((c) => c.name === "before");
  snap();
  steps.push({ step: 2, actor: "human", command: "checkpoint panel: Name 'before'", shows: { named: before.length, checkpoints: cps.length }, pass: before.length === 1 && cps.length === 1 });
  const beforeId = before[0]?.checkpoint_id ?? "";

  // 3. the agent proposes (MCP)
  const pr = await served.mcp("propose", { doc_path: DOC, content: TEXT, reason: "product scene" });
  snap();
  steps.push({ step: 3, actor: "agent", command: "MCP propose", shows: { is_error: pr["isError"] }, pass: pr["isError"] === false });

  // 4. the gutter of the agent's note and the proposal view
  await pg.waitFor(`!!app.vault.getAbstractFileByPath(${J(DOC)})`, "the agent's note in the vault");
  await pg.ev(`__rootmark.open(${J(DOC)}).then(() => true)`);
  await pg.waitFor("__rootmark.gutter().length === 2", "two gutter marks on the agent's note");
  const g4 = await pg.ev<{ label: string; title: string }[]>("__rootmark.gutter()");
  await cmd("proposals");
  await pg.waitFor("!!document.querySelector('.workspace-leaf-content[data-type=\"rootmark-proposals\"] table.rootmark-proposals')", "the proposal table");
  const rows4 = await pg.ev<string[][]>("[...__rootmark.view('rootmark-proposals').querySelectorAll('tr.rootmark-layer-proposal')].map((tr) => [...tr.querySelectorAll('td')].slice(1).map((td) => td.textContent))");
  snap();
  steps.push({
    step: 4, actor: "human", command: "gutter + proposal view",
    shows: { gutter: g4, proposal_rows: rows4 },
    pass: g4.length === 2 && g4.every((m) => m.label === "P" && m.title.startsWith(`proposal · Agent · source: ${AGENT.name}/${AGENT.version} · `))
      && rows4.length === 2 && rows4.every((r) => r[0] === DOC && r[1] === "Grind notes" && r[2] === `${AGENT.name}/${AGENT.version}`),
  });

  // search panel: query, lexical, include proposals or not, at
  const search = async (q: string, inc: boolean, at = ""): Promise<{ meta: string; canonical: string[]; proposal: string[]; error: string | null }> => {
    await cmd("search");
    await pg.waitFor("!!document.querySelector('.workspace-leaf-content[data-type=\"rootmark-search\"] .setting-item')", "the search panel");
    await pg.ev(`(() => { __rootmark.field('rootmark-search', 'Query', ${J(q)}); __rootmark.field('rootmark-search', 'Mode', 'lexical');
      __rootmark.check('rootmark-search', 'Include proposals', ${String(inc)}); __rootmark.field('rootmark-search', 'At checkpoint', ${J(at)});
      __rootmark.view('rootmark-search').querySelectorAll('.rootmark-meta, .rootmark-error').forEach((e) => e.remove());
      __rootmark.button(__rootmark.view('rootmark-search'), 'Search').click(); return true; })()`);
    await pg.waitFor("!!__rootmark.view('rootmark-search').querySelector('.rootmark-meta, .rootmark-error')", "search results");
    return pg.ev(`(() => { const v = __rootmark.view('rootmark-search'); return {
      meta: v.querySelector('.rootmark-meta')?.textContent ?? '',
      canonical: [...v.querySelectorAll('li.rootmark-hit-canonical')].map((l) => l.textContent),
      proposal: [...v.querySelectorAll('li.rootmark-hit-proposal')].map((l) => l.textContent),
      error: v.querySelector('.rootmark-error')?.textContent ?? null }; })()`);
  };
  const onDoc = (hits: string[]): string[] => hits.filter((h) => h.startsWith(`${DOC} · `));

  // 5. default search does not find the agent's text; with proposals it is apart
  const s5 = await search("kalimba", false);
  const s5p = await search("kalimba", true);
  snap();
  steps.push({
    step: 5, actor: "human", command: "search panel (default; include proposals)",
    shows: { default_on_doc: onDoc(s5.canonical).length, with_proposals: { canonical_on_doc: onDoc(s5p.canonical).length, proposal_on_doc: onDoc(s5p.proposal).length } },
    pass: s5.error === null && onDoc(s5.canonical).length === 0 && onDoc(s5p.canonical).length === 0 && onDoc(s5p.proposal).length > 0,
  });

  // 6. Promote selected, through the reason modal
  await cmd("proposals");
  await pg.ev("(() => { const v = __rootmark.view('rootmark-proposals'); v.querySelectorAll('tr.rootmark-layer-proposal input[type=\"checkbox\"]').forEach((c) => { if (!c.checked) c.click(); }); __rootmark.button(v, 'Promote selected').click(); return true; })()");
  await pg.waitFor("[...document.querySelectorAll('.modal')].some((m) => m.querySelector('.modal-title')?.textContent === 'Promote 2 block(s)')", "the reason modal");
  const modal = "[...document.querySelectorAll('.modal')].find((m) => m.querySelector('.modal-title')?.textContent === 'Promote 2 block(s)')";
  const demoteDisabled = await pg.ev<boolean>("__rootmark.button(__rootmark.view('rootmark-proposals'), 'Demote').disabled");
  const empty = await pg.ev<boolean>(`__rootmark.button(${modal}, 'Promote').disabled`);
  await pg.ev(`(__rootmark.type(${modal}.querySelector('textarea.rootmark-reason'), '   '), true)`);
  const blank = await pg.ev<boolean>(`__rootmark.button(${modal}, 'Promote').disabled`);
  await pg.ev(`(__rootmark.type(${modal}.querySelector('textarea.rootmark-reason'), ${J(REASON)}), true)`);
  const filled = await pg.ev<boolean>(`__rootmark.button(${modal}, 'Promote').disabled`);
  await pg.ev(`(__rootmark.button(${modal}, 'Promote').click(), true)`);
  await pg.waitFor("__rootmark.notices().some((n) => n.includes('2 promoted'))", "the '2 promoted' notice");
  const promoted = events(vault).filter((e) => e.kind === "Promoted");
  snap();
  steps.push({
    step: 6, actor: "human", command: "proposal view: Promote selected + reason modal",
    shows: { promote_disabled: { empty, blank, filled }, demote_disabled: demoteDisabled, promoted: promoted.length, actors: promoted.map((e) => e.meta["actor"]), reasons: promoted.map((e) => e.meta["reason"]) },
    pass: empty && blank && !filled && demoteDisabled && promoted.length === 2
      && promoted.every((e) => JSON.stringify(e.meta["actor"]) === JSON.stringify({ id: "editor", kind: "human" }) && e.meta["reason"] === REASON),
  });

  // 7. search finds it, canonical
  const s7 = await search("kalimba", false);
  snap();
  steps.push({ step: 7, actor: "human", command: "search panel", shows: { on_doc: onDoc(s7.canonical) }, pass: onDoc(s7.canonical).length > 0 && onDoc(s7.canonical).every((h) => h.includes(" · canonical · ")) });

  // 8. trace viewer: the latest trace is the plugin's step 7 search; its link opens the note at the block
  await cmd("traces");
  await pg.ev("(app.workspace.getLeavesOfType('rootmark-traces')[0].view.refresh(), true)");
  await pg.waitFor("!!__rootmark.view('rootmark-traces').querySelector('details summary')", "the traces");
  const t8 = await pg.ev<{ summary: string; links: string[] }>("(() => { const d = __rootmark.view('rootmark-traces').querySelector('details'); return { summary: d.querySelector('summary').textContent, links: [...d.querySelectorAll('a')].map((a) => a.textContent) }; })()");
  const blame8 = (await direct(serverPort, token, "blame", { doc_path: DOC }))["blocks"] as { lineage_id: string }[];
  const linkIdx = t8.links.findIndex((l) => l.startsWith(`${DOC} · `));
  const lineage = t8.links[linkIdx]?.split(" · ")[1] ?? "";
  const blockIdx = blame8.findIndex((b) => b.lineage_id.startsWith(lineage));
  await pg.ev(`(__rootmark.view('rootmark-traces').querySelector('details').querySelectorAll('a')[${String(linkIdx)}].click(), true)`);
  await pg.waitFor(`app.workspace.getActiveFile()?.path === ${J(DOC)}`, "the note opened from the trace");
  const wantLine = [0, 2][blockIdx] ?? -1;
  await pg.waitFor(`app.workspace.activeEditor?.editor?.getCursor().line === ${String(wantLine)}`, "the cursor on the block's line", 10_000).catch(() => undefined);
  const cursor = await pg.ev<number>("app.workspace.activeEditor?.editor?.getCursor().line ?? -1");
  snap();
  steps.push({
    step: 8, actor: "human", command: "trace viewer + jump",
    shows: { summary_consumer: t8.summary.split(" · ")[1], links_on_doc: t8.links.filter((l) => l.startsWith(`${DOC} · `)).length, cursor_line: cursor, block_line: wantLine },
    pass: t8.summary.split(" · ")[1] === CONSUMER && t8.links.filter((l) => l.startsWith(`${DOC} · `)).length === 2 && linkIdx >= 0 && blockIdx >= 0 && cursor === wantLine,
  });

  // 9. back to "before": search at; the checkpoint panel's read-only past view
  const s9 = await search("kalimba", false, "before");
  const pastOf = async (doc: string): Promise<string> => {
    await pg.ev(`__rootmark.open(${J(doc)}).then(() => true)`);
    await cmd("checkpoints");
    await pg.ev("app.workspace.getLeavesOfType('rootmark-checkpoints')[0].view.refresh().then(() => true)");
    await pg.ev(`(() => { const li = [...__rootmark.view('rootmark-checkpoints').querySelectorAll('li')].find((l) => l.textContent.startsWith('before ·')); __rootmark.button(li, ${J(`blame ${doc} then`)}).click(); return true; })()`);
    await pg.waitFor("!!__rootmark.view('rootmark-checkpoints').querySelector('pre.rootmark-past')?.textContent", "the past view");
    return pg.ev<string>("__rootmark.view('rootmark-checkpoints').querySelector('pre.rootmark-past').textContent");
  };
  const past9doc = await pastOf(DOC);
  const past9esp = await pastOf("espresso.md");
  snap();
  steps.push({
    step: 9, actor: "human", command: "search at 'before' + checkpoint past view",
    shows: { same_checkpoint_as_step_2: s9.meta.startsWith(`checkpoint ${beforeId.slice(0, 8)}`), hits_on_doc: onDoc(s9.canonical).length, past_doc: past9doc.split("\n")[0], past_espresso: past9esp.split("\n")[0], espresso_blocks: past9esp.split("\n").length - 1 },
    pass: s9.error === null && s9.meta.startsWith(`checkpoint ${beforeId.slice(0, 8)}`) && onDoc(s9.canonical).length === 0
      && past9doc === `${DOC} had no snapshot at checkpoint ${beforeId}` && past9esp.startsWith(`espresso.md at ${beforeId} (read only)`) && past9esp.split("\n").length > 1,
  });

  // 10. the agent tries to change canonical content (MCP); 11. nothing changed
  await pg.ev("__rootmark.open('espresso.md').then(() => true)");
  await pg.waitFor("__rootmark.gutter().length > 0", "the gutter of espresso.md");
  const g10 = await pg.ev<unknown[]>("__rootmark.gutter()");
  const s10 = await search("espresso recipe", false);
  const ev10 = readFileSync(join(vault, ".rootmark/events.jsonl"), "utf8");
  const r10a = await served.mcp("propose", { doc_path: "espresso.md", mode: "replace_section", heading: "Recipe", content: "Agents rewrite the recipe.", reason: "scene: change canonical" });
  const r10b = await served.mcp("propose", { doc_path: "espresso.md", mode: "append", content: "An agent line without a heading.", reason: "scene: heading-less append" });
  const e10a = (structured(r10a)["error"] ?? {}) as { code?: string };
  const e10b = (structured(r10b)["error"] ?? {}) as { code?: string; hint?: string };
  snap();
  steps.push({
    step: 10, actor: "agent", command: "MCP propose replace_section + heading-less append",
    shows: { replace_section: e10a.code, append: e10b.code, hint: e10b.hint },
    pass: e10a.code === "denied" && e10b.code === "policy_violation" && e10b.hint === "start the appended text with a heading" && readFileSync(join(vault, ".rootmark/events.jsonl"), "utf8") === ev10,
  });
  await pg.ev("app.workspace.getLeavesOfType('markdown').length > 0 && app.plugins.plugins.rootmark.refreshBlame().then(() => true)");
  await pg.ev("__rootmark.open('espresso.md').then(() => true)");
  await pg.waitFor("__rootmark.gutter().length > 0", "the gutter of espresso.md");
  const g11 = await pg.ev<unknown[]>("__rootmark.gutter()");
  const s11 = await search("espresso recipe", false);
  snap();
  steps.push({
    step: 11, actor: "human", command: "gutter + search panel unchanged",
    shows: { gutter_marks: g11.length, gutter_unchanged: JSON.stringify(g11) === JSON.stringify(g10), search_unchanged: JSON.stringify(s11.canonical) === JSON.stringify(s10.canonical) },
    pass: g11.length > 0 && g11.every((m) => (m as { label: string }).label === "C") && JSON.stringify(g11) === JSON.stringify(g10) && s11.canonical.length > 0 && JSON.stringify(s11.canonical) === JSON.stringify(s10.canonical),
  });
} catch (e) {
  steps.push({ step: steps.length + 1, actor: "-", command: "error", shows: { error: e instanceof Error ? e.message : String(e) }, pass: false });
} finally {
  if (page) obsidianExit = await page.close();
  if (proxy) await proxy.close();
  const code = served ? await served.stop() : null;
  if (served && code !== 0) steps.push({ step: steps.length + 1, actor: "-", command: "rootmark serve exit", shows: { code, stderr_tail: served.stderr().slice(-400) }, pass: false });
}
const sceneLog = proxy?.log ?? [];
// E-3: Markdown changes per step (index i = step i+1): only the propose step
const mdChanged = steps.map((_, i) => {
  const a = shas[i];
  const b = shas[i + 1];
  return a && b ? changed(a, b) : null;
});
for (const [i, s] of steps.entries()) {
  const want = s.step === 3 ? [DOC] : [];
  if (JSON.stringify(mdChanged[i]) !== JSON.stringify(want)) s.pass = false;
}

// ---- E1-9: the connection screen on a stale index, then the rebuild ------------
let oldObsidianExit: number | null = null;
let connection: Step = { step: "E1-9", actor: "human", command: "connection screen (stale index) + rebuild", shows: null, pass: false };
const oldVault = join(work, "old", "vault");
const oldLog: { method: string; path: string; consumer: string | null; status: number }[] = [];
{
  let s2: Served | null = null;
  let p2: Awaited<ReturnType<typeof recordingProxy>> | null = null;
  let pg2: Page | null = null;
  try {
    cpSync(oldIndex, oldVault, { recursive: true });
    // the fixture keeps its pre-rename metadata directory (K-GH.1)
    const mig = spawnSync(rootmark, ["migrate", "--rename-store", oldVault], { encoding: "utf8" });
    if (mig.status !== 0) throw new Error(`rootmark migrate --rename-store: ${mig.stderr}`);
    const keptIndex = join(work, "old", "index.keep");
    cpSync(join(oldVault, ".rootmark/index"), keptIndex, { recursive: true });
    const proxy2 = await freePort();
    const server2 = await freePort();
    prepare(oldVault, proxy2, "auto_rebuild = false\n");
    // `prepare`'s first `rootmark serve` (no policy yet) runs with the default
    // auto_rebuild = true and would rebuild the old index: the fixture's
    // (migrated) index is put back before the scene starts
    rmSync(join(oldVault, ".rootmark/index"), { recursive: true, force: true });
    cpSync(keptIndex, join(oldVault, ".rootmark/index"), { recursive: true });
    s2 = await serve(rootmark, oldVault, server2, AGENT);
    p2 = await recordingProxy(proxy2, server2);
    pg2 = await launchObsidian(obsidian, join(work, "obsidian-data-old"), oldVault, await freePort());
    const pgo = pg2;
    await enablePlugin(pgo);
    const sb = await pgo.ev<{ text: string; title: string }>("__rootmark.statusBar()");
    await pgo.waitFor("__rootmark.notices().some((n) => n.includes('index stale'))", "the index notice", 10_000).catch(() => undefined);
    const notices = await pgo.ev<string[]>("__rootmark.notices()");
    // the search panel says why search is unavailable
    await pgo.ev("app.commands.executeCommandById('rootmark:search')");
    await pgo.waitFor("!!document.querySelector('.workspace-leaf-content[data-type=\"rootmark-search\"] .setting-item')", "the search panel");
    const runSearch = async (): Promise<{ error: string | null; hits: number }> => {
      await pgo.ev("(() => { __rootmark.field('rootmark-search', 'Query', 'extinguisher'); __rootmark.view('rootmark-search').querySelectorAll('.rootmark-meta, .rootmark-error').forEach((e) => e.remove()); __rootmark.button(__rootmark.view('rootmark-search'), 'Search').click(); return true; })()");
      await pgo.waitFor("!!__rootmark.view('rootmark-search').querySelector('.rootmark-meta, .rootmark-error')", "search results");
      return pgo.ev("(() => { const v = __rootmark.view('rootmark-search'); return { error: v.querySelector('.rootmark-error')?.textContent ?? null, hits: v.querySelectorAll('li.rootmark-hit-canonical').length }; })()");
    };
    const before = await runSearch();
    // the rebuild path the screen names: stop the server, `rootmark index build`,
    // start it again, Reconnect
    await s2.stop();
    s2 = null;
    run(["index", "build", oldVault]);
    s2 = await serve(rootmark, oldVault, server2, AGENT);
    await pgo.ev("app.commands.executeCommandById('rootmark:reconnect')");
    await pgo.waitFor("(() => { const s = __rootmark.statusBar(); return !!s && !s.text.includes('index'); })()", "the status bar without an index warning");
    const sbAfter = await pgo.ev<{ text: string; title: string }>("__rootmark.statusBar()");
    const after = await runSearch();
    connection = {
      ...connection,
      shows: {
        status_bar: sb.text, tooltip: sb.title, notice_shown: notices.some((n) => n.includes("index stale") && n.includes("rootmark index build")),
        search_before_rebuild: before, status_bar_after_rebuild: sbAfter.text, search_after_rebuild: after,
      },
      pass: sb.text === "rootmark 0.1.0 · human:editor · index stale"
        && sb.title.includes("mdkernel-index/2") && sb.title.includes("mdkernel-index/3") && sb.title.includes("rootmark index build") && sb.title.includes("auto_rebuild = true")
        && notices.some((n) => n.includes("index stale") && n.includes("rootmark index build"))
        && before.error !== null && before.error.startsWith("index_stale: ") && before.hits === 0
        && sbAfter.text === "rootmark 0.1.0 · human:editor" && after.error === null && after.hits > 0,
    };
  } catch (e) {
    connection = { ...connection, shows: { error: e instanceof Error ? e.message : String(e) }, pass: false };
  } finally {
    if (pg2) oldObsidianExit = await pg2.close();
    if (p2) {
      oldLog.push(...p2.log);
      await p2.close();
    }
    if (s2) await s2.stop();
  }
}

// E-4: every request the plugin sent is a contract endpoint, as the plugin
const endpoints = new Set([...TOOLS.map((t) => `POST /v1/${t}`), "GET /v1/status"]);
const all = [...sceneLog, ...oldLog];
const outside = all.filter((l) => !endpoints.has(`${l.method} ${l.path}`) || l.consumer !== CONSUMER);
const used = [...new Set(all.map((l) => `${l.method} ${l.path}`))].sort();
const passed = steps.filter((s) => s.pass).length;
const result = {
  harness: "real Obsidian, separate instance (own --user-data-dir, CDP), plugin bundle installed; K-E1.3",
  consumer: CONSUMER,
  agent: `${AGENT.name}/${AGENT.version}`,
  steps,
  passed,
  total: 11,
  e3_md_changed_per_step: mdChanged,
  e3_md_changes_only_at_propose: mdChanged.every((c, i) => JSON.stringify(c) === JSON.stringify(steps[i]?.step === 3 ? [DOC] : [])),
  e4_access_log: { requests: all.length, endpoints_used: used, outside_contract: outside.length, outside },
  e1_9_connection: connection,
  obsidian_exit: obsidianExit,
  obsidian_exit_connection_step: oldObsidianExit,
};
const pass = passed === 11 && steps.length === 11 && result.e3_md_changes_only_at_propose && outside.length === 0 && connection.pass;
writeFileSync(join(out, "plugin_e2e_real.json"), JSON.stringify({ ...result, pass }, null, 2) + "\n");
writeFileSync(join(out, "access_log.json"), JSON.stringify(all.map((l) => ({ method: l.method, path: l.path, consumer: l.consumer, status: l.status })), null, 2) + "\n");
if (process.env["ROOTMARK_E2E_KEEP"] !== "1") rmSync(work, { recursive: true, force: true });
for (const s of steps) console.log(`plugin-e2e-real: ${s.pass ? "ok  " : "FAIL"} ${String(s.step)} ${s.actor} ${s.command}`);
console.log(`plugin-e2e-real: ${connection.pass ? "ok  " : "FAIL"} E1-9 ${connection.command}`);
console.log(`plugin-e2e-real: ${String(passed)}/11; E1-9 ${String(connection.pass)}; E-3 ${String(result.e3_md_changes_only_at_propose)}; E-4 outside contract ${String(outside.length)} of ${String(all.length)} requests`);
await sleep(0);
process.exit(pass ? 0 : 1);
