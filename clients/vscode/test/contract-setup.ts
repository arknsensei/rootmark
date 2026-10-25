// The contract checker in every test file (P2-B/B4 (c)): each HTTP
// exchange the client makes (`exchange`) is recorded and checked against
// the schema generated from the contract. A request outside the contract
// fails the file; response violations (a fake server answering outside the
// contract on purpose) are recorded, not failed. With
// ROOTMARK_CONTRACT_LOG_DIR set, the file's record is written there.
import { mkdirSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { afterAll, expect } from "vitest";
import { report, setRecorder, type CallRecord } from "../../obsidian/src/contract-check";

const records: CallRecord[] = [];
setRecorder((r) => records.push(r));

afterAll(() => {
  setRecorder(null);
  const rep = report(records);
  const dir = process.env["ROOTMARK_CONTRACT_LOG_DIR"];
  const file = basename(expect.getState().testPath ?? "unknown");
  if (dir !== undefined && dir !== "") {
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, `${process.env["ROOTMARK_CONTRACT_CLIENT"] ?? "client"}-${file}.json`), JSON.stringify({ file, ...rep }, null, 1) + "\n");
  }
  if (rep.request_violations.length > 0) {
    throw new Error(`calls outside the contract in ${file}:\n${rep.request_violations.join("\n")}`);
  }
});
