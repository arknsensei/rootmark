// The one place the plugin touches Obsidian's file adapter (P2-0/Z3;
// ASSUMPTIONS Z3-1): it observes the writes and renames that go through this
// Obsidian process, so that the saves among them are attested (contract
// §2.6). Measured in the real Obsidian: an editor save goes `vault.modify` →
// `adapter.write`, a rename `adapter.rename`; a file another process writes
// reaches Obsidian only as a `modify` event and does not pass here (another
// plugin of the same Obsidian does: the human channel, Z3-1). A rewrite of
// the bytes already on disk is no save. An append is reported only when the
// file held, before it, exactly the bytes this Obsidian last reported and
// holds after it those bytes followed by the appended ones; a rename only
// when the caller allows it (`mayReportRename`: the old document holds
// nothing but canonical content, Z3 review, finding 11) and the file holds
// after it the very bytes it held before — the server then checks they are
// the canonical head of the old document (`renamed_from`, contract §2.6).
// Every comparison is on raw bytes (automated security reviews of P2-0/Z3).
// The wrapped calls run
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

/** Called with a note this Obsidian saved and the exact bytes it wrote (an
 * append: the bytes before it followed by the appended ones); a rename
 * carries the old path (`renamedFrom`: the server checks the bytes are the
 * canonical head of that document, contract §2.6). */
export type OnSaved = (path: string, bytes: Uint8Array, renamedFrom?: string) => void;

import { contentHash } from "./blake3";

const isNote = (p: string): boolean => p.endsWith(".md");

/** Wraps `adapter`'s `write`, `append` and `rename`; returns the function
 * that restores the adapter as it was. */
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
  // the content_hash of the bytes this Obsidian last reported, by path
  const reported = new Map<string, string>();
  const report = (path: string, bytes: Uint8Array, renamedFrom?: string): void => {
    reported.set(path, contentHash(bytes));
    onSaved(path, bytes, renamedFrom);
  };
  const bytesOf = async (path: string): Promise<Uint8Array | null> =>
    (await adapter.exists(path)) ? new Uint8Array(await adapter.readBinary(path)) : null;
  const same = (a: Uint8Array, b: Uint8Array): boolean => a.length === b.length && a.every((x, i) => x === b[i]);
  adapter.write = async (path: string, data: string, options?: unknown): Promise<void> => {
    const before = isNote(path) ? await bytesOf(path) : null;
    const bytes = new TextEncoder().encode(data);
    await write(path, data, options);
    if (isNote(path) && (before === null || !same(before, bytes))) report(path, bytes);
  };
  adapter.append = async (path: string, data: string, options?: unknown): Promise<void> => {
    const before = isNote(path) && data !== "" ? await bytesOf(path) : null;
    const own = before !== null && reported.get(path) === contentHash(before);
    await append(path, data, options);
    if (!own) return;
    const added = new TextEncoder().encode(data);
    const want = new Uint8Array(before.length + added.length);
    want.set(before, 0);
    want.set(added, before.length);
    const after = await bytesOf(path);
    if (after !== null && same(after, want)) report(path, after);
  };
  adapter.rename = async (from: string, to: string): Promise<void> => {
    // decided before the rename: the server still knows the old path
    const before = isNote(to) ? await bytesOf(from) : null;
    const allowed = before !== null && (await mayReportRename(from).catch(() => false));
    await rename(from, to);
    if (!allowed) return;
    const after = await bytesOf(to);
    if (after !== null && same(after, before)) report(to, after, from);
  };
  return () => {
    for (const [n, d] of own) {
      if (d === undefined) Reflect.deleteProperty(target, n);
      else Object.defineProperty(target, n, d);
    }
  };
}
