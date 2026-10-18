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
// While renaming a note Obsidian rewrites the links to it in other notes
// (and in the note itself) through `adapter.process` (measured in the real
// Obsidian, P2-0.1/Y1): a `process` write of a note within
// `LINK_UPDATE_MS` after a rename whose bytes name the renamed note (its
// file name without `.md`; the newest such rename) is reported as that
// rename's link update (`linkUpdateOf`: the renamed note's new path; Y1
// review 4, 5: renames in a row, unrelated `process` writes); a folder
// rename is no note rename (its notes' link updates stay unattested, a
// known limit). The server accepts it only
// when the bytes differ from the note's head in link targets naming that
// note, in canonical blocks (contract §2.6), so an agent's bytes in the
// note are never vouched for. A `process` write outside a rename is not
// reported (it is not a save of this Obsidian's editor).
// Every comparison is on raw bytes (automated security reviews of P2-0/Z3).
// The wrapped calls run
// unchanged; the plugin never writes a file itself. The static scan
// (test/e2.test.ts) allows the adapter's methods to be bound only here.
// No Obsidian import: tested with a fake adapter (test/p2_0.test.ts).

/** The adapter methods this module wraps or reads. */
export interface OwnWritesAdapter {
  write(path: string, data: string, options?: unknown): Promise<void>;
  process(path: string, fn: (data: string) => string, options?: unknown): Promise<string>;
  append(path: string, data: string, options?: unknown): Promise<void>;
  rename(from: string, to: string): Promise<void>;
  exists(path: string): Promise<boolean>;
  read(path: string): Promise<string>;
  readBinary(path: string): Promise<ArrayBuffer>;
}

/** How a reported write came about (contract §2.6). */
export interface SavedHow {
  /** a rename: the old path (the server checks the bytes are the canonical
   * head of that document) */
  renamedFrom?: string;
  /** a link update during a rename: the renamed note's new path (the
   * server checks the bytes differ from the head only in link targets
   * naming it) */
  linkUpdateOf?: string;
  /** with `linkUpdateOf`: the renamed note's old path (the server checks
   * every changed link named it; Y1 security review) */
  linkUpdateFrom?: string;
}

/** Called with a note this Obsidian saved and the exact bytes it wrote (an
 * append: the bytes before it followed by the appended ones). */
export type OnSaved = (path: string, bytes: Uint8Array, how: SavedHow) => void;

/** How long after a rename a `process` write counts as its link update. */
export const LINK_UPDATE_MS = 10_000;

import { contentHash } from "./blake3";

const isNote = (p: string): boolean => p.endsWith(".md");

/** Wraps `adapter`'s `write`, `append` and `rename`; returns the function
 * that restores the adapter as it was. */
export function observeOwnWrites(
  adapter: OwnWritesAdapter,
  onSaved: OnSaved,
  mayReportRename: (from: string) => Promise<boolean> = () => Promise.resolve(true),
  now: () => number = () => Date.now(),
): () => void {
  const target = adapter as unknown as Record<string, unknown>;
  const names = ["write", "append", "rename", "process"] as const;
  const own = names.map((n) => [n, Object.getOwnPropertyDescriptor(target, n)] as const);
  const write = adapter.write.bind(adapter);
  const append = adapter.append.bind(adapter);
  const rename = adapter.rename.bind(adapter);
  const proc = adapter.process.bind(adapter);
  // the recent note renames this Obsidian made: (new path, when it
  // finished), newest last
  const renames: { from: string; to: string; at: number }[] = [];
  // the content_hash of the bytes this Obsidian last reported, by path
  const reported = new Map<string, string>();
  const report = (path: string, bytes: Uint8Array, how: SavedHow = {}): void => {
    reported.set(path, contentHash(bytes));
    onSaved(path, bytes, how);
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
    if (isNote(to)) {
      renames.push({ from, to, at: now() });
      if (renames.length > 16) renames.shift();
    }
    if (!allowed) return;
    const after = await bytesOf(to);
    if (after !== null && same(after, before)) report(to, after, { renamedFrom: from });
  };
  adapter.process = async (path: string, fn: (data: string) => string, options?: unknown): Promise<string> => {
    const t = now();
    const during = isNote(path) && renames.some((r) => t - r.at <= LINK_UPDATE_MS);
    // the text Obsidian read inside `process` (not a separate read, which
    // another process could change in between: Y1 security review B2)
    const read: { data: string | null } = { data: null };
    const out = await proc(path, (data: string) => ((read.data = data), fn(data)), options);
    const input = read.data;
    if (!during || input === null || input === out) return out;
    // the newest recent rename whose note the rewrite links to more often
    // than before (Y1 review 4, 5)
    const stem = (p: string): string => (p.split("/").pop() ?? p).replace(/\.md$/, "");
    // links whose target names a note with this file name (Y1 review 2,
    // round 2: a raw substring count misses a new name inside the old one)
    const count = (text: string, s: string): number =>
      [...text.matchAll(/\[\[([^[\]|#]*)|\]\(\s*<?([^()<>#\s]*)/g)].filter((m) => {
        const t = (m[1] ?? m[2] ?? "").trim();
        let d = t;
        try {
          d = decodeURIComponent(t);
        } catch {
          // not percent-encoded
        }
        return stem(d) === s;
      }).length;
    const before = input;
    const r = [...renames].reverse().find((x) => t - x.at <= LINK_UPDATE_MS && count(out, stem(x.to)) > count(before, stem(x.to)));
    if (r === undefined) return out;
    const bytes = new TextEncoder().encode(out);
    const after = await bytesOf(path);
    // reported only when the file holds what this process call wrote
    if (after !== null && same(after, bytes)) report(path, bytes, { linkUpdateOf: r.to, linkUpdateFrom: r.from });
    return out;
  };
  return () => {
    for (const [n, d] of own) {
      if (d === undefined) Reflect.deleteProperty(target, n);
      else Object.defineProperty(target, n, d);
    }
  };
}
