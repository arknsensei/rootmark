// A fake `mdk serve` HTTP endpoint for contract tests: records every
// request and answers with whatever the test queues (default 200 `{}`).
import { createServer, type IncomingMessage, type Server } from "node:http";
import type { AddressInfo } from "node:net";

export interface Recorded {
  method: string;
  path: string;
  headers: IncomingMessage["headers"];
  body: string;
}

export interface Fake {
  url: string;
  requests: Recorded[];
  reply: (status: number, body: unknown) => void;
  close: () => Promise<void>;
}

export async function startFake(): Promise<Fake> {
  const requests: Recorded[] = [];
  const queue: { status: number; body: string }[] = [];
  const server: Server = createServer((req, res) => {
    let body = "";
    req.on("data", (c: Buffer) => (body += c.toString("utf8")));
    req.on("end", () => {
      requests.push({ method: req.method ?? "", path: req.url ?? "", headers: req.headers, body });
      const r = queue.shift() ?? { status: 200, body: "{}" };
      res.writeHead(r.status, { "Content-Type": "application/json" });
      res.end(r.body);
    });
  });
  await new Promise<void>((ok) => server.listen(0, "127.0.0.1", ok));
  const port = (server.address() as AddressInfo).port;
  return {
    url: `http://127.0.0.1:${String(port)}`,
    requests,
    reply: (status, body) => {
      queue.push({ status, body: typeof body === "string" ? body : JSON.stringify(body) });
    },
    close: () =>
      new Promise<void>((ok) => {
        server.close(() => {
          ok();
        });
      }),
  };
}

/** The Node transport (the e2e harness uses the same). */
export { fetchTransport } from "../src/node-transport";
