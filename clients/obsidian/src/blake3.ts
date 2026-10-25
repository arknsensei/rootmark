// BLAKE3 (hash mode, 32-byte output), a port of the reference
// implementation (github.com/BLAKE3-team/BLAKE3, reference_impl). The
// plugin has no runtime dependency (clients/obsidian/CLAUDE.md) and the
// attestation's `content_hash` is the first 32 hex characters of the
// BLAKE3 hash of the saved file bytes (spec §8, 2.0.0; P2-0/Z3).

const OUT_LEN = 32;
const BLOCK_LEN = 64;
const CHUNK_LEN = 1024;

const CHUNK_START = 1 << 0;
const CHUNK_END = 1 << 1;
const PARENT = 1 << 2;
const ROOT = 1 << 3;

const IV = new Uint32Array([
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
]);

const MSG_PERMUTATION = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8];

/** `a[i]` (always in range here; 0 keeps the type a number) */
const word = (a: Uint32Array | Uint8Array | readonly number[], i: number): number => a[i] ?? 0;

function rotr(x: number, n: number): number {
  return ((x >>> n) | (x << (32 - n))) >>> 0;
}

function g(s: Uint32Array, a: number, b: number, c: number, d: number, mx: number, my: number): void {
  s[a] = (word(s, a) + word(s, b) + mx) >>> 0;
  s[d] = rotr(word(s, d) ^ word(s, a), 16);
  s[c] = (word(s, c) + word(s, d)) >>> 0;
  s[b] = rotr(word(s, b) ^ word(s, c), 12);
  s[a] = (word(s, a) + word(s, b) + my) >>> 0;
  s[d] = rotr(word(s, d) ^ word(s, a), 8);
  s[c] = (word(s, c) + word(s, d)) >>> 0;
  s[b] = rotr(word(s, b) ^ word(s, c), 7);
}

function round(s: Uint32Array, m: Uint32Array): void {
  g(s, 0, 4, 8, 12, word(m, 0), word(m, 1));
  g(s, 1, 5, 9, 13, word(m, 2), word(m, 3));
  g(s, 2, 6, 10, 14, word(m, 4), word(m, 5));
  g(s, 3, 7, 11, 15, word(m, 6), word(m, 7));
  g(s, 0, 5, 10, 15, word(m, 8), word(m, 9));
  g(s, 1, 6, 11, 12, word(m, 10), word(m, 11));
  g(s, 2, 7, 8, 13, word(m, 12), word(m, 13));
  g(s, 3, 4, 9, 14, word(m, 14), word(m, 15));
}

function permute(m: Uint32Array): Uint32Array {
  const out = new Uint32Array(16);
  for (let i = 0; i < 16; i++) out[i] = word(m, word(MSG_PERMUTATION, i));
  return out;
}

function compress(
  cv: Uint32Array,
  block: Uint32Array,
  counter: number,
  blockLen: number,
  flags: number,
): Uint32Array {
  const s = new Uint32Array(16);
  s.set(cv, 0);
  s.set(IV.subarray(0, 4), 8);
  s[12] = counter >>> 0;
  s[13] = Math.floor(counter / 0x100000000) >>> 0;
  s[14] = blockLen;
  s[15] = flags;
  let m = block;
  for (let r = 0; r < 7; r++) {
    round(s, m);
    if (r < 6) m = permute(m);
  }
  for (let i = 0; i < 8; i++) {
    s[i] = (word(s, i) ^ word(s, i + 8)) >>> 0;
    s[i + 8] = (word(s, i + 8) ^ word(cv, i)) >>> 0;
  }
  return s;
}

function words(bytes: Uint8Array, at: number, len: number): Uint32Array {
  const block = new Uint8Array(BLOCK_LEN);
  block.set(bytes.subarray(at, at + len));
  const w = new Uint32Array(16);
  for (let i = 0; i < 16; i++) {
    w[i] =
      (word(block, 4 * i) | (word(block, 4 * i + 1) << 8) | (word(block, 4 * i + 2) << 16) | (word(block, 4 * i + 3) << 24)) >>> 0;
  }
  return w;
}

interface Output {
  cv: Uint32Array;
  block: Uint32Array;
  counter: number;
  blockLen: number;
  flags: number;
}

function chainingValue(o: Output): Uint32Array {
  return compress(o.cv, o.block, o.counter, o.blockLen, o.flags).subarray(0, 8);
}

/** The output of one chunk (`bytes[at, at + len)`, `len` ≤ 1024). */
function chunkOutput(bytes: Uint8Array, at: number, len: number, counter: number): Output {
  let cv: Uint32Array = IV.slice();
  const blocks = Math.max(1, Math.ceil(len / BLOCK_LEN));
  for (let b = 0; b < blocks - 1; b++) {
    const flags = b === 0 ? CHUNK_START : 0;
    cv = compress(cv, words(bytes, at + b * BLOCK_LEN, BLOCK_LEN), counter, BLOCK_LEN, flags).subarray(0, 8);
  }
  const last = blocks - 1;
  const lastLen = len - last * BLOCK_LEN;
  return {
    cv,
    block: words(bytes, at + last * BLOCK_LEN, lastLen),
    counter,
    blockLen: lastLen,
    flags: (last === 0 ? CHUNK_START : 0) | CHUNK_END,
  };
}

function parentOutput(left: Uint32Array, right: Uint32Array): Output {
  const block = new Uint32Array(16);
  block.set(left, 0);
  block.set(right, 8);
  return { cv: IV.slice(), block, counter: 0, blockLen: BLOCK_LEN, flags: PARENT };
}

function popCv(stack: Uint32Array[]): Uint32Array {
  const cv = stack.pop();
  if (cv === undefined) throw new Error("blake3: empty chaining-value stack");
  return cv;
}

/** BLAKE3 of `bytes`, the 32-byte default output. */
export function blake3(bytes: Uint8Array): Uint8Array {
  const chunks = Math.max(1, Math.ceil(bytes.length / CHUNK_LEN));
  // the reference implementation's chaining-value stack
  const stack: Uint32Array[] = [];
  let out: Output | null = null;
  for (let c = 0; c < chunks; c++) {
    const at = c * CHUNK_LEN;
    const o = chunkOutput(bytes, at, Math.min(CHUNK_LEN, bytes.length - at), c);
    if (c === chunks - 1) {
      out = o;
      break;
    }
    let cv = chainingValue(o);
    let total = c + 1;
    while ((total & 1) === 0) {
      cv = chainingValue(parentOutput(popCv(stack), cv));
      total >>= 1;
    }
    stack.push(cv);
  }
  let o = out ?? chunkOutput(bytes, 0, 0, 0);
  while (stack.length > 0) {
    o = parentOutput(popCv(stack), chainingValue(o));
  }
  const words32 = compress(o.cv, o.block, 0, o.blockLen, o.flags | ROOT);
  const res = new Uint8Array(OUT_LEN);
  for (let i = 0; i < OUT_LEN / 4; i++) {
    const w = word(words32, i);
    res[4 * i] = w & 0xff;
    res[4 * i + 1] = (w >>> 8) & 0xff;
    res[4 * i + 2] = (w >>> 16) & 0xff;
    res[4 * i + 3] = (w >>> 24) & 0xff;
  }
  return res;
}

/** The attestation's `content_hash`: the first 32 hex characters (spec §8). */
export function contentHash(bytes: Uint8Array): string {
  return Array.from(blake3(bytes).subarray(0, 16), (b) => b.toString(16).padStart(2, '0')).join('');
}

/** The kernel's `hash32` of a string (its UTF-8 bytes): `version_id` and
 * the other 32-hex ids (spec §1). */
export function hash32(s: string): string {
  return contentHash(new TextEncoder().encode(s));
}
