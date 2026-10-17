// Pieces of the plugin e2e harness (TASKS E4): one `mdk serve` process with
// MCP stdio (the agent) and HTTP (the plugin), a recording proxy between the
// plugin and the server (E-4: the HTTP access log), and a Markdown sha256
// snapshot (E-3). Node only; nothing here is part of the plugin bundle.
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { createServer, request, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { join, relative } from "node:path";
import { createInterface } from "node:readline";

/** A free 127.0.0.1 port (bound and released). */
export async function freePort(): Promise<number> {
  const s = createServer();
  await new Promise<void>((ok) => s.listen(0, "127.0.0.1", ok));
  const p = (s.address() as AddressInfo).port;
  await new Promise<void>((ok) => s.close(() => { ok(); }));
  return p;
}

export interface Served {
  /** one MCP tools/call; resolves with `result` (tool errors included) */
  mcp(name: string, args: unknown): Promise<Record<string, unknown>>;
  stop(): Promise<number | null>;
  stderr(): string;
}

/** `mdk serve --vault <v> --mcp stdio --http 127.0.0.1:<port>`, the MCP
 * handshake done as `clientInfo` (contract §1: one process for both). */
export async function serve(mdk: string, vault: string, httpPort: number, clientInfo: { name: string; version: string }): Promise<Served> {
  const child: ChildProcessWithoutNullStreams = spawn(mdk, ["serve", "--vault", vault, "--mcp", "stdio", "--http", `127.0.0.1:${String(httpPort)}`], {
    stdio: ["pipe", "pipe", "pipe"],
  });
  let err = "";
  const listening = new Promise<void>((ok, fail) => {
    child.stderr.on("data", (d: Buffer) => {
      err += d.toString("utf8");
      if (err.includes(`http listening on 127.0.0.1:${String(httpPort)}`)) ok();
    });
    child.on("exit", (c) => {
      fail(new Error(`mdk serve exited (${String(c)}) before listening:\n${err}`));
    });
  });
  const waiting = new Map<number, (r: Record<string, unknown>) => void>();
  createInterface({ input: child.stdout }).on("line", (line) => {
    const m = JSON.parse(line) as { id?: number; result?: Record<string, unknown>; error?: unknown };
    if (typeof m.id === "number" && waiting.has(m.id)) {
      const w = waiting.get(m.id);
      waiting.delete(m.id);
      w?.(m.result ?? { jsonrpc_error: m.error });
    }
  });
  let next = 1;
  const rpc = (method: string, params: unknown): Promise<Record<string, unknown>> =>
    new Promise((ok) => {
      const id = next++;
      waiting.set(id, ok);
      child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
    });
  await rpc("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo });
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }) + "\n");
  await listening;
  return {
    mcp: (name, args) => rpc("tools/call", { name, arguments: args }),
    stderr: () => err,
    stop: () =>
      new Promise((ok) => {
        child.on("exit", (code) => { ok(code); });
        child.stdin.end(); // end of the MCP stream stops the service
      }),
  };
}

export interface Logged {
  method: string;
  path: string;
  consumer: string | null;
  status: number;
}

/** A proxy on `listenPort` → 127.0.0.1:`targetPort` that records every
 * request (method, path, X-Consumer, status): the HTTP access log of E-4. */
export async function recordingProxy(listenPort: number, targetPort: number): Promise<{ log: Logged[]; close: () => Promise<void> }> {
  const log: Logged[] = [];
  const server: Server = createServer((req, res) => {
    const chunks: Buffer[] = [];
    req.on("data", (c: Buffer) => chunks.push(c));
    req.on("end", () => {
      const up = request(
        { host: "127.0.0.1", port: targetPort, method: req.method, path: req.url, headers: req.headers },
        (r) => {
          const xc = req.headers["x-consumer"];
          log.push({ method: req.method ?? "", path: req.url ?? "", consumer: typeof xc === "string" ? xc : null, status: r.statusCode ?? 0 });
          res.writeHead(r.statusCode ?? 502, r.headers);
          r.pipe(res);
        },
      );
      up.on("error", (e) => {
        log.push({ method: req.method ?? "", path: req.url ?? "", consumer: null, status: 502 });
        res.writeHead(502);
        res.end(String(e));
      });
      up.end(Buffer.concat(chunks));
    });
  });
  await new Promise<void>((ok) => server.listen(listenPort, "127.0.0.1", ok));
  return { log, close: () => new Promise<void>((ok) => server.close(() => { ok(); })) };
}

/** {path: sha256} of every Markdown file of the vault (outside .mdkernel). */
export function mdSha(vault: string): Record<string, string> {
  const out: Record<string, string> = {};
  const walk = (d: string): void => {
    for (const f of readdirSync(d).sort()) {
      if (f.startsWith(".")) continue;
      const p = join(d, f);
      if (statSync(p).isDirectory()) walk(p);
      else if (f.endsWith(".md")) out[relative(vault, p)] = createHash("sha256").update(readFileSync(p)).digest("hex");
    }
  };
  walk(vault);
  return out;
}

export function changed(a: Record<string, string>, b: Record<string, string>): string[] {
  return [...new Set([...Object.keys(a), ...Object.keys(b)])].filter((k) => a[k] !== b[k]).sort();
}
