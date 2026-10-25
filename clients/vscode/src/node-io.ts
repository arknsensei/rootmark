// The Node side of the client: the vault reader (read only: the settings
// file and the token; never a write) and the HTTP transport (global fetch).
import { readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import type { VaultReader } from "../../obsidian/src/api";

export { fetchTransport } from "../../obsidian/src/node-transport";

export function fsReader(root: string): VaultReader {
  return {
    read: (p) => readFile(join(root, p), "utf8"),
    exists: async (p) => {
      try {
        await stat(join(root, p));
        return true;
      } catch {
        return false;
      }
    },
  };
}
