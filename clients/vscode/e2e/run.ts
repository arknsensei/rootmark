// The P2-B e2e of the VS Code skeleton (GOAL P2-B B-7, B-8; TASKS B4): its
// command layer (src/core.ts — what the commands call) against a real
// `rootmark serve` (MCP for the agent, HTTP for the extension, one process),
// through its own connection (`connect` with the Node vault reader). VS Code
// itself does not run here: the extension glue (src/extension.ts) only maps
// these results to editor decorations and pickers.
// Steps: status; an agent proposes over MCP; search with a path prefix, a
// time filter and proposals; blame of each note as decorations from the
// server's locations; history; a human promote and demote of the agent's
// block (X-Consumer: vscode); every HTTP exchange checked by the contract
// checker. Saves are not attested: nothing here calls attest.
//
//   node e2e/.build/run.mjs --rootmark <rootmark> --out <dir>
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { connect } from "../../obsidian/src/api";
import { report, setRecorder, type CallRecord } from "../../obsidian/src/contract-check";
import { freePort, serve } from "../../obsidian/e2e/harness";
import { blameView, CONSUMER, demote, historyLines, promote, search, statusLine } from "../src/core";
import { fetchTransport, fsReader } from "../src/node-io";

const arg = (name: string): string => {
  const i = process.argv.indexOf(`--${name}`);
  const v = i >= 0 ? process.argv[i + 1] : undefined;
  if (v === undefined) throw new Error(`--${name} is required`);
  return v;
};
const rootmark = arg("rootmark");
const out = arg("out");
mkdirSync(out, { recursive: true });

const FILES: Record<string, string> = {
  "journal/2026/10/24.md": "# Dün\n\nÇay demlendi, kahve beklendi.\n",
  "journal/2026/10/25.md": "# Bugün\n\n- çay\n- kahve\n  - filtre\n\n    girintili not\n",
  "notes/demleme.md": "---\ntags: [çay]\n---\n# Demleme\n\nÇay demlenirken su kaynamaz.\n\n> [!note] Not\n> Kahve demlenmez.\n",
};
const work = mkdtempSync(join(tmpdir(), "rootmark-p2b-vscode-"));
const vault = join(work, "vault");
for (const [p, t] of Object.entries(FILES)) {
  mkdirSync(dirname(join(vault, p)), { recursive: true });
  writeFileSync(join(vault, p), t);
}
const ing = spawnSync(rootmark, ["ingest-dir", vault, vault], { encoding: "utf8" });
if (ing.status !== 0) throw new Error(`ingest-dir: ${ing.stderr}`);
const port = await freePort();
writeFileSync(
  join(vault, ".rootmark/policy.toml"),
  `[promotion]\npolicy = "human_only"\n\n[edit]\npolicy = "append_only"\n\n[version_trigger]\nidle_ms = 30000\nmin_sim_change = 0.2\n\n[journal]\nretention_hours = 168\n\n[server]\nhttp_port = ${String(port)}\ntoken_rotate_days = 30\nmax_propose_bytes = 1048576\n\n[index]\ncompact_every_s = 300\ndate_from_path = "{yyyy}/{mm}/{dd}"\n\n[identity]\nhuman_id = "ada"\n\n[attestation]\nunattested = "canonical"\n`,
);

const records: CallRecord[] = [];
setRecorder((r) => records.push(r));
const served = await serve(rootmark, vault, port, { name: "p2b-agent", version: "1.0" });
const steps: Record<string, unknown> = {};
const st = await connect(fsReader(vault), fetchTransport, CONSUMER);
if (st.kind !== "online") throw new Error(`offline: ${st.reason}`);
const client = st.client;
steps["status"] = { text: statusLine(st.status).text.replace(/rootmark \S+ /, "rootmark <v> "), human: st.status.server_policy.identity.human_id };

// the agent proposes (MCP)
const prop = await served.mcp("propose", { doc_path: "journal/2026/10/25.md", mode: "append", content: "## Öneri\n\nYeşil çay da denensin.\n", reason: "e2e" });
if (prop["isError"] === true) throw new Error(`propose: ${JSON.stringify(prop)}`);

// search with filters
const all = await search(client, "çay kahve", { mode: "lexical", k: 10, include_proposal: true });
const dated = await search(client, "çay kahve", { mode: "lexical", k: 10, date_from: "2026-10-25", date_to: "2026-10-25" });
const prefixed = await search(client, "çay", { mode: "lexical", k: 10, path_prefix: ["notes"] });
steps["search"] = {
  all: all.items.map((i) => [i.doc_path, i.layer, i.line]),
  dated: dated.items.map((i) => [i.doc_path, i.line]),
  undated_excluded: dated.undated_excluded,
  prefixed: prefixed.items.map((i) => i.doc_path),
};

// blame decorations from locations
const blame: Record<string, unknown> = {};
let agentBlock: string | null = null;
for (const p of Object.keys(FILES).sort()) {
  const v = await blameView(client, p, readFileSync(join(vault, p), "utf8"));
  blame[p] = v.kind === "located" ? v.blocks.map((b) => [b.layer, b.start, b.end]) : v.kind;
  if (v.kind === "located") agentBlock = v.blocks.find((b) => b.layer === "proposal")?.lineage_id ?? agentBlock;
}
steps["blame"] = blame;
if (agentBlock === null) throw new Error("no proposal block located");
steps["history_before"] = (await historyLines(client, agentBlock)).slice(1);
steps["promote"] = await promote(client, agentBlock, "e2e: approved in VS Code");
steps["demote"] = await demote(client, agentBlock, "e2e: undone in VS Code");
steps["history_after"] = (await historyLines(client, agentBlock)).slice(1);
// a saved, not yet ingested edit: the decorations are withheld ("stale")
writeFileSync(join(vault, "notes/demleme.md"), `${FILES["notes/demleme.md"] ?? ""}\nYeni satır.\n`);
const changed = await blameView(client, "notes/demleme.md", readFileSync(join(vault, "notes/demleme.md"), "utf8"));
steps["after_save"] = changed.kind;
await served.stop();
setRecorder(null);

const contract = report(records);
const located = Object.values(blame).filter((b) => Array.isArray(b)).length;
const result = {
  steps,
  contract,
  pass:
    located === Object.keys(FILES).length &&
    String(steps["promote"]).startsWith("promoted") &&
    String(steps["demote"]).startsWith("demoted") &&
    dated.items.length > 0 &&
    dated.items.every((i) => i.doc_path === "journal/2026/10/25.md") &&
    dated.undated_excluded === 1 &&
    prefixed.items.every((i) => i.doc_path.startsWith("notes/")) &&
    changed.kind === "stale" &&
    contract.request_violations.length === 0 &&
    contract.response_violations.length === 0 &&
    contract.deliberate_violations.length === 0,
};
writeFileSync(join(out, "vscode_e2e.json"), JSON.stringify(result, null, 1) + "\n");
rmSync(work, { recursive: true, force: true });
console.log(`p2b vscode e2e: ${String(located)}/${String(Object.keys(FILES).length)} notes located; contract: ${String(contract.calls)} calls, ${String(contract.request_violations.length + contract.response_violations.length)} violations → ${result.pass ? "pass" : "FAIL"}`);
process.exit(result.pass ? 0 : 3);
