// The plugin e2e (GOAL P1-E E-2, E-3, E-4, E-5, E-6; TASKS E4). Obsidian
// cannot run headless here (ASSUMPTIONS E4-1), so the plugin's command layer
// (src/commands.ts, src/api.ts — exactly what its panels call) is driven in
// Node against a real `mdk serve`, through the plugin's own connection
// (`connect`: policy.toml [server] http_port + server.token, read only).
// The agent acts over MCP in the same server process (contract §1).
//
//   node e2e/.build/run.mjs --mdk <mdk> --vault-src <dir of .md> --out <dir>
//
// <out>/plugin_e2e.json: the 11 steps (shows, pass), the access log check
// (E-4), the Markdown changes per step (E-3), the actor of the promotion
// (E-5); no time stamp, port, token or absolute path (E-8). <out>/
// events.jsonl, traces.jsonl, proposals.jsonl, access_log.json beside it.
import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { connect, describeError, MdkError, TOOLS, type BlameBlock, type VaultReader } from "../src/api";
import { blameAt, blameOf, listProposals, nameCheckpoint, promoteAll, recentTraces, runSearch } from "../src/commands";
import { CONSUMER } from "../src/consumer";
import { fetchTransport } from "../src/node-transport";
import { changed, freePort, mdSha, recordingProxy, serve } from "./harness";

const arg = (name: string): string => {
  const i = process.argv.indexOf(`--${name}`);
  const v = i >= 0 ? process.argv[i + 1] : undefined;
  if (v === undefined) throw new Error(`--${name} is required`);
  return v;
};
const mdk = arg("mdk");
const src = arg("vault-src");
const out = arg("out");
mkdirSync(out, { recursive: true });

const DOC = "agent/scene.md";
const TEXT = "# Grind notes\n\nAn agent recommends a coarser grind for the kalimba roast.\n";
const AGENT = { name: "e2e-agent", version: "1.0" };

const work = mkdtempSync(join(tmpdir(), "mdk-plugin-e2e-"));
const vault = join(work, "vault");
mkdirSync(vault);
for (const f of readdirSync(src).filter((f) => f.endsWith(".md"))) copyFileSync(join(src, f), join(vault, f));
const run = (args: string[], input = ""): void => {
  const r = spawnSync(mdk, args, { input, encoding: "utf8" });
  if (r.status !== 0) throw new Error(`mdk ${args.join(" ")}: exit ${String(r.status)}\n${r.stderr}`);
};
run(["ingest-dir", vault, vault]);
// the first `mdk serve` writes the default policy (human_only, append_only)
run(["serve", "--vault", vault, "--mcp", "stdio"]);
const proxyPort = await freePort();
const serverPort = await freePort();
const policyPath = join(vault, ".mdkernel/policy.toml");
let policy = readFileSync(policyPath, "utf8");
policy = policy.replace("[server]\n", `[server]\nhttp_port = ${String(proxyPort)}\n`);
policy = policy.replace("# [identity]\n# human_id = \"<your id>\"\n", '[identity]\nhuman_id = "editor"\n');
if (!policy.includes(`http_port = ${String(proxyPort)}`) || !policy.includes('human_id = "editor"')) {
  throw new Error("could not set [server] http_port and [identity] human_id in the default recipe");
}
writeFileSync(policyPath, policy);

// the plugin reads the vault, never writes it
const reader: VaultReader = {
  read: (p) => Promise.resolve(readFileSync(join(vault, p), "utf8")),
  exists: (p) => Promise.resolve(existsSync(join(vault, p))),
};
const docs = (): string[] => Object.keys(mdSha(vault));

const served = await serve(mdk, vault, serverPort, AGENT);
const proxy = await recordingProxy(proxyPort, serverPort);
const shas = [mdSha(vault)];
const snap = (): void => {
  shas.push(mdSha(vault));
};
type Step = { step: number; actor: string; command: string; shows: unknown; pass: boolean };
const steps: Step[] = [];
const sameBlocks = (a: BlameBlock[] | null, b: BlameBlock[] | null): boolean => JSON.stringify(a) === JSON.stringify(b);
const structured = (r: Record<string, unknown>): Record<string, unknown> =>
  (r["structuredContent"] ?? {}) as Record<string, unknown>;

try {
  // 1. connect + status (the plugin's start)
  const st = await connect(reader, fetchTransport, CONSUMER);
  if (st.kind !== "online") throw new Error(`plugin offline: ${st.reason}`);
  const client = st.client;
  snap();
  steps.push({
    step: 1, actor: "human", command: "connect (status)",
    shows: { spec_version: st.status.spec_version, promotion: st.status.policy.promotion, edit: st.status.policy.edit, human_id: st.settings.humanId },
    pass: st.status.policy.promotion === "human_only" && st.status.policy.edit === "append_only" && st.settings.humanId === "editor",
  });

  // 2. checkpoint panel: name the state "before"
  const cp = await nameCheckpoint(client, "before");
  snap();
  steps.push({ step: 2, actor: "human", command: "nameCheckpoint before", shows: { name: cp.name, event_seq: cp.event_seq }, pass: cp.name === "before" });

  // 3. the agent proposes (MCP)
  const pr = await served.mcp("propose", { doc_path: DOC, content: TEXT, reason: "product scene" });
  const prs = structured(pr);
  snap();
  steps.push({
    step: 3, actor: "agent", command: "MCP propose", shows: { is_error: pr["isError"], edit_policy: prs["edit_policy"] },
    pass: pr["isError"] === false,
  });

  // 4. blame gutter + proposal view
  const bl4 = await blameOf(client, DOC);
  const rows = await listProposals(client, docs());
  snap();
  const docRows = rows.filter((r) => r.doc_path === DOC);
  steps.push({
    step: 4, actor: "human", command: "blameOf + listProposals",
    shows: { blocks: bl4?.map((b) => ({ layer: b.layer, author_kind: b.author_kind, source: b.source })), proposal_rows: rows.map((r) => ({ doc_path: r.doc_path, source: r.source })) },
    pass: bl4 !== null && bl4.length > 0 && bl4.every((b) => b.layer === "proposal" && b.author_kind === "Agent" && b.source === "e2e-agent/1.0") && docRows.length === bl4.length,
  });

  // deliberate tests (scripts/test_tamper_p1e.sh): the harness itself
  // breaks E-4 (a request outside the contract, as the plugin) or E-3 (a
  // Markdown write outside the propose step); never set in a real run
  const tamper = process.env["MDK_E2E_TAMPER"] ?? "";
  if (tamper === "offcontract") {
    await fetch(`http://127.0.0.1:${String(proxyPort)}/v1/demote`, {
      method: "POST",
      headers: { Authorization: `Bearer ${st.settings.token}`, "X-Consumer": CONSUMER },
      body: "{}",
    });
  }
  if (tamper === "mdwrite") writeFileSync(join(vault, "espresso.md"), readFileSync(join(vault, "espresso.md"), "utf8") + "\nedited\n");

  // 5. search panel, default
  const s5 = await runSearch(client, "kalimba", { mode: "lexical" });
  snap();
  steps.push({ step: 5, actor: "human", command: "runSearch default", shows: { hits_on_doc: s5.canonical.filter((h) => h.doc_path === DOC).length }, pass: s5.canonical.every((h) => h.doc_path !== DOC) && s5.proposal.length === 0 });

  // 6. Promote from the proposal view, with a reason; once without (E-5)
  const noReason = await promoteAll(client, [docRows[0]?.lineage_id ?? ""], "");
  const promoted = await promoteAll(client, docRows.map((r) => r.lineage_id), "approved in the scene");
  snap();
  const nr = noReason[0]?.error ?? null;
  const e5Actor = promoted.every((o) => o.result?.decision === "promoted" && JSON.stringify(o.result.event.meta.actor) === JSON.stringify({ id: "editor", kind: "human" }));
  const e5NoReason = nr instanceof MdkError && nr.status === 400 && nr.code === "invalid_arguments";
  steps.push({
    step: 6, actor: "human", command: "promoteAll (reason) + promoteAll (no reason)",
    shows: {
      decisions: promoted.map((o) => o.result?.decision ?? o.error?.code),
      actors: promoted.map((o) => (o.result?.decision === "promoted" ? o.result.event.meta.actor : null)),
      without_reason: { status: nr?.status ?? null, code: nr?.code ?? null },
    },
    pass: promoted.length > 0 && e5Actor && e5NoReason,
  });

  // 7. search finds it, canonical
  const s7 = await runSearch(client, "kalimba", { mode: "lexical" });
  snap();
  const on7 = s7.canonical.filter((h) => h.doc_path === DOC);
  steps.push({ step: 7, actor: "human", command: "runSearch default", shows: { hits_on_doc: on7.map((h) => h.layer) }, pass: on7.length > 0 });

  // 8. trace viewer: the latest trace is step 7's, and rests on the document
  const tr = await recentTraces(client, { consumer: CONSUMER });
  snap();
  const last = tr[0];
  steps.push({
    step: 8, actor: "human", command: "recentTraces",
    shows: { traces: tr.length, latest_is_step_7: last?.trace_id === s7.trace_id, jump_targets_on_doc: last?.returned.filter((r) => r.doc_path === DOC).length ?? 0 },
    pass: last?.trace_id === s7.trace_id && (last.returned.some((r) => r.doc_path === DOC)),
  });

  // 9. back to "before": search at, blame at (read-only past view)
  const s9 = await runSearch(client, "kalimba", { mode: "lexical", at: "before" });
  const doc9 = await blameAt(client, DOC, "before");
  const esp9 = await blameAt(client, "espresso.md", "before");
  const espNow = await blameOf(client, "espresso.md");
  snap();
  steps.push({
    step: 9, actor: "human", command: "runSearch at + blameAt",
    shows: { same_checkpoint_as_step_2: s9.checkpoint_id === cp.checkpoint_id, hits_on_doc: s9.canonical.filter((h) => h.doc_path === DOC).length, doc_then: doc9 === null ? "no snapshot" : "present", espresso_then_equals_now: sameBlocks(esp9, espNow) },
    pass: s9.checkpoint_id === cp.checkpoint_id && s9.canonical.every((h) => h.doc_path !== DOC) && doc9 === null && esp9 !== null && sameBlocks(esp9, espNow),
  });

  // 10. the agent tries to change canonical content (MCP)
  const before10 = { blame: await blameOf(client, "espresso.md"), hits: (await runSearch(client, "espresso recipe", { mode: "lexical" })).canonical };
  const ev10 = readFileSync(join(vault, ".mdkernel/events.jsonl"), "utf8");
  const r10a = await served.mcp("propose", { doc_path: "espresso.md", mode: "replace_section", heading: "Recipe", content: "Agents rewrite the recipe.", reason: "scene: change canonical" });
  const r10b = await served.mcp("propose", { doc_path: "espresso.md", mode: "append", content: "An agent line without a heading.", reason: "scene: heading-less append" });
  snap();
  const e10a = (structured(r10a)["error"] ?? {}) as { code?: string };
  const e10b = (structured(r10b)["error"] ?? {}) as { code?: string; message?: string; hint?: string };
  // what the plugin would show a user for such an error: the hint included
  const shown = describeError(new MdkError("policy_violation", e10b.message ?? "", 422, e10b.hint ?? null));
  steps.push({
    step: 10, actor: "agent", command: "MCP propose replace_section + heading-less append",
    shows: { replace_section: e10a.code, append: e10b.code, hint: e10b.hint, plugin_shows_hint: shown.includes("start the appended text with a heading") },
    pass: e10a.code === "denied" && e10b.code === "policy_violation" && e10b.hint === "start the appended text with a heading"
      && readFileSync(join(vault, ".mdkernel/events.jsonl"), "utf8") === ev10 && shown.includes("start the appended text with a heading"),
  });

  // 11. blame and search unchanged (E-6)
  const after = { blame: await blameOf(client, "espresso.md"), hits: (await runSearch(client, "espresso recipe", { mode: "lexical" })).canonical };
  snap();
  steps.push({
    step: 11, actor: "human", command: "blameOf + runSearch",
    shows: { blame_unchanged: sameBlocks(after.blame, before10.blame), search_unchanged: JSON.stringify(after.hits) === JSON.stringify(before10.hits) },
    pass: sameBlocks(after.blame, before10.blame) && JSON.stringify(after.hits) === JSON.stringify(before10.hits) && after.hits.length > 0,
  });
} catch (e) {
  steps.push({ step: steps.length + 1, actor: "-", command: "error", shows: { error: describeError(e) }, pass: false });
} finally {
  await proxy.close();
  const code = await served.stop();
  if (code !== 0) steps.push({ step: steps.length + 1, actor: "-", command: "mdk serve exit", shows: { code, stderr_tail: served.stderr().slice(-400) }, pass: false });
}

// E-3: Markdown changes per step (index i = step i+1)
const mdChanged = steps.map((_, i) => {
  const a = shas[i];
  const b = shas[i + 1];
  return a && b ? changed(a, b) : null;
});
// mark each step's pass with "nothing but the propose step changes Markdown"
for (const [i, s] of steps.entries()) {
  const want = s.step === 3 ? [DOC] : [];
  if (JSON.stringify(mdChanged[i]) !== JSON.stringify(want)) s.pass = false;
}
// E-4: every request the plugin sent is a contract endpoint, as the plugin
const endpoints = new Set([...TOOLS.map((t) => `POST /v1/${t}`), "GET /v1/status"]);
const outside = proxy.log.filter((l) => !endpoints.has(`${l.method} ${l.path}`) || l.consumer !== CONSUMER);
const used = [...new Set(proxy.log.map((l) => `${l.method} ${l.path}`))].sort();
const result = {
  harness: "plugin command layer in Node against a real mdk serve (Obsidian not headless; ASSUMPTIONS E4-1)",
  consumer: CONSUMER,
  agent: `${AGENT.name}/${AGENT.version}`,
  steps,
  passed: steps.filter((s) => s.pass).length,
  total: 11,
  e3_md_changed_per_step: mdChanged,
  e3_md_changes_only_at_propose: mdChanged.every((c, i) => JSON.stringify(c) === JSON.stringify(steps[i]?.step === 3 ? [DOC] : [])),
  e4_access_log: { requests: proxy.log.length, endpoints_used: used, outside_contract: outside.length, outside: outside },
  e5: { promoted_by_human_editor: steps.find((s) => s.step === 6)?.pass ?? false },
};
const pass = result.passed === 11 && steps.length === 11 && result.e3_md_changes_only_at_propose && outside.length === 0;
writeFileSync(join(out, "plugin_e2e.json"), JSON.stringify({ ...result, pass }, null, 2) + "\n");
writeFileSync(join(out, "access_log.json"), JSON.stringify(proxy.log.map((l) => ({ method: l.method, path: l.path, consumer: l.consumer, status: l.status })), null, 2) + "\n");
for (const f of ["events.jsonl", "traces.jsonl", "proposals.jsonl"]) copyFileSync(join(vault, ".mdkernel", f), join(out, f));
if (process.env["MDK_E2E_KEEP"] !== "1") rmSync(work, { recursive: true, force: true });
for (const s of steps) console.log(`plugin-e2e: ${s.pass ? "ok  " : "FAIL"} ${String(s.step)} ${s.actor} ${s.command}`);
console.log(`plugin-e2e: ${String(result.passed)}/11; E-3 ${String(result.e3_md_changes_only_at_propose)}; E-4 outside contract ${String(outside.length)} of ${String(proxy.log.length)} requests`);
process.exit(pass ? 0 : 1);
