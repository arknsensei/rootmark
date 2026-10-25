// The P2-B e2e of the Obsidian plugin (GOAL P2-B B-7, B-8; TASKS B4): the
// plugin's command layer (src/commands.ts `blameResultOf`, `alignNote` —
// exactly what the blame gutter calls) against a real `rootmark serve`,
// through the plugin's own connection (`connect`), on a vault of D-NOTES'
// real notes (every 16th note of research/data/MANIFEST.d_notes.json, no
// hidden path). Obsidian cannot run headless here (P1-E ASSUMPTIONS E4-1).
// Checks: every note aligns **by location** (contract v2) and each block's
// lines are its located bytes' lines; a note changed on disk and not
// ingested says "stale" and guesses no line; every HTTP exchange is checked
// by the contract checker (requests and the real server's answers).
//
//   node e2e/p2_b/.build/run.mjs --rootmark <rootmark> --repo <repo root> --out <dir>
//
// <out>/obsidian_e2e.json: per note (path, blocks, aligned by), the stale
// case, the contract report; no time, port, token or absolute path.
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { connect, type VaultReader } from "../../src/api";
import { alignNote, blameResultOf } from "../../src/commands";
import { CONSUMER } from "../../src/consumer";
import { report, setRecorder, type CallRecord } from "../../src/contract-check";
import { fetchTransport } from "../../src/node-transport";
import { freePort, serve } from "../harness";

const arg = (name: string): string => {
  const i = process.argv.indexOf(`--${name}`);
  const v = i >= 0 ? process.argv[i + 1] : undefined;
  if (v === undefined) throw new Error(`--${name} is required`);
  return v;
};
const rootmark = arg("rootmark");
const repo = arg("repo");
const out = arg("out");
mkdirSync(out, { recursive: true });

interface ManifestNote {
  path: string;
}
const manifest = JSON.parse(readFileSync(join(repo, "research/data/MANIFEST.d_notes.json"), "utf8")) as { notes: ManifestNote[] };
const notes = manifest.notes
  .map((n) => n.path)
  .filter((p) => !p.split("/").some((c) => c.startsWith(".")))
  .filter((_, i) => i % 16 === 0);
if (notes.length < 20) throw new Error(`only ${String(notes.length)} notes (at least 20)`);

const work = mkdtempSync(join(tmpdir(), "rootmark-p2b-obsidian-"));
const vault = join(work, "vault");
for (const p of notes) {
  mkdirSync(dirname(join(vault, p)), { recursive: true });
  copyFileSync(join(repo, "research/data/d_notes/notes", p), join(vault, p));
}
const ing = spawnSync(rootmark, ["ingest-dir", vault, vault], { encoding: "utf8" });
if (ing.status !== 0) throw new Error(`ingest-dir: ${ing.stderr}`);
const port = await freePort();
writeFileSync(
  join(vault, ".rootmark/policy.toml"),
  // `rootmark serve`'s default recipe, with the port and the human
  `[promotion]\npolicy = "human_only"\n\n[edit]\npolicy = "append_only"\n\n[version_trigger]\nidle_ms = 30000\nmin_sim_change = 0.2\n\n[journal]\nretention_hours = 168\n\n[server]\nhttp_port = ${String(port)}\ntoken_rotate_days = 30\nmax_propose_bytes = 1048576\n\n[index]\ncompact_every_s = 300\n\n[identity]\nhuman_id = "ada"\n\n[attestation]\nunattested = "canonical"\n`,
);

const records: CallRecord[] = [];
setRecorder((r) => records.push(r));
const served = await serve(rootmark, vault, port, { name: "p2b-e2e", version: "1.0" });
const reader: VaultReader = {
  read: (p) => Promise.resolve(readFileSync(join(vault, p), "utf8")),
  exists: (p) => Promise.resolve((() => { try { readFileSync(join(vault, p)); return true; } catch { return false; } })()),
};
const st = await connect(reader, fetchTransport, CONSUMER);
if (st.kind !== "online") throw new Error(`offline: ${st.reason}`);
const client = st.client;

const lineOf = (b: Uint8Array, off: number): number => b.subarray(0, off).filter((x) => x === 10).length;
const perNote: { path: string; blocks: number; by: string; aligned: boolean; lines_ok: boolean }[] = [];
for (const p of notes) {
  const r = await blameResultOf(client, p);
  if (r === null) throw new Error(`${p}: not in the log`);
  const bytes = new Uint8Array(readFileSync(join(vault, p)));
  const { al, by } = alignNote(r, bytes, []);
  let linesOk = al.aligned;
  if (al.aligned) {
    for (const { block, range } of al.pairs) {
      const l = block.loc;
      if (l === undefined || range.start !== lineOf(bytes, l.byte_start) || range.end !== lineOf(bytes, l.byte_end - 1)) linesOk = false;
    }
  }
  perNote.push({ path: p, blocks: r.blocks.length, by, aligned: al.aligned, lines_ok: linesOk });
}
// stale: a note changed on disk, not ingested (no --watch)
const stalePath = notes[0] as string;
writeFileSync(join(vault, stalePath), "Inserted first line.\n\n" + readFileSync(join(vault, stalePath), "utf8"));
const sr = await blameResultOf(client, stalePath);
const sb = new Uint8Array(readFileSync(join(vault, stalePath)));
const stale = sr === null ? null : alignNote(sr, sb, []);
const staleCase = {
  loc_stale: sr?.loc_stale ?? null,
  aligned: stale?.al.aligned ?? null,
  says_stale: stale !== null && !stale.al.aligned && "stale" in stale.al,
  marks: stale !== null && stale.al.aligned ? stale.al.pairs.length : 0,
};
await served.stop();
setRecorder(null);
const contract = report(records);
const aligned = perNote.filter((n) => n.aligned && n.by === "loc" && n.lines_ok).length;
const result = {
  notes: perNote.length,
  aligned_by_location: aligned,
  per_note: perNote,
  stale: staleCase,
  contract,
  pass:
    aligned === perNote.length &&
    perNote.length >= 20 &&
    staleCase.loc_stale === true &&
    staleCase.says_stale &&
    staleCase.marks === 0 &&
    contract.request_violations.length === 0 &&
    contract.response_violations.length === 0 &&
    contract.deliberate_violations.length === 0,
};
writeFileSync(join(out, "obsidian_e2e.json"), JSON.stringify(result, null, 1) + "\n");
rmSync(work, { recursive: true, force: true });
console.log(`p2b obsidian e2e: ${String(aligned)}/${String(perNote.length)} notes aligned by location; stale ${String(staleCase.says_stale)}; contract: ${String(contract.calls)} calls, ${String(contract.request_violations.length + contract.response_violations.length)} violations → ${result.pass ? "pass" : "FAIL"}`);
process.exit(result.pass ? 0 : 3);
