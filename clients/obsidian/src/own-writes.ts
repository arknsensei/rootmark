// The one place the plugin touches Obsidian's file adapter (P2-0/Z3;
// ASSUMPTIONS Z3-1): it observes the writes and renames that go through this
// Obsidian process, so that the saves among them are attested (contract
// §2.6). Measured in the real Obsidian: an editor save goes `vault.modify` →
// `adapter.write`, a rename `adapter.rename`; a file another process writes
// reaches Obsidian only as a `modify` event and does not pass here (another
// plugin of the same Obsidian does: the human channel, Z3-1). A rewrite of
// the bytes already on disk is no save; a rename is reported only when the
// caller allows it (`mayReportRename`: the old document holds nothing but
// canonical content, Z3 review, finding 11). The wrapped calls run
// unchanged; the plugin never writes a file itself. The static scan
// (test/e2.test.ts) allows the adapter's methods to be bound only here.
// No Obsidian import: tested with a fake adapter (test/p2_0.test.ts).

/** The adapter methods this module wraps or reads. */
export interface OwnWritesAdapter {
  write(path: string, data: string, options?: unknown): Promise<void>;
  append(path: string, data: string, options?: unknown): Promise<void>;
  rename(from: string, to: string): Promise<void>;
  exists(path: string): Promise<boolean>;
  read(path: string): Promise<string>;
  readBinary(path: string): Promise<ArrayBuffer>;
}

/** Called with a note this Obsidian saved and the bytes now on disk (for a
 * rename or an append: read back after the call). */
export type OnSaved = (path: string, bytes: Uint8Array) => void;

const isNote = (p: string): boolean => p.endsWith(".md");

/** Wraps `adapter`'s `write`, `append` and `rename`; returns the function
 * that restores the adapter as it was. A write of the bytes already on disk
 * is no save (Obsidian rewriting what another process wrote, e.g. after a
 * reload) and is not reported: attesting it would vouch for that process's
 * bytes (Z3 review, finding 1). */
export function observeOwnWrites(
  adapter: OwnWritesAdapter,
  onSaved: OnSaved,
  mayReportRename: (from: string) => Promise<boolean> = () => Promise.resolve(true),
): () => void {
  const target = adapter as unknown as Record<string, unknown>;
  const names = ["write", "append", "rename"] as const;
  const own = names.map((n) => [n, Object.getOwnPropertyDescriptor(target, n)] as const);
  const write = adapter.write.bind(adapter);
  const append = adapter.append.bind(adapter);
  const rename = adapter.rename.bind(adapter);
  const current = async (path: string): Promise<string | null> => ((await adapter.exists(path)) ? adapter.read(path) : null);
  const readBack = async (path: string): Promise<void> => {
    onSaved(path, new Uint8Array(await adapter.readBinary(path)));
  };
  adapter.write = async (path: string, data: string, options?: unknown): Promise<void> => {
    const before = isNote(path) ? await current(path) : null;
    await write(path, data, options);
    if (isNote(path) && before !== data) onSaved(path, new TextEncoder().encode(data));
  };
  adapter.append = async (path: string, data: string, options?: unknown): Promise<void> => {
    await append(path, data, options);
    if (isNote(path) && data !== "") await readBack(path);
  };
  adapter.rename = async (from: string, to: string): Promise<void> => {
    // decided before the rename: the server still knows the old path
    const report = isNote(to) && (await mayReportRename(from).catch(() => false));
    await rename(from, to);
    if (report) await readBack(to);
  };
  return () => {
    for (const [n, d] of own) {
      if (d === undefined) Reflect.deleteProperty(target, n);
      else Object.defineProperty(target, n, d);
    }
  };
}
