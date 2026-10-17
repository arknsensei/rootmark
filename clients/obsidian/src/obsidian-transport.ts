// The plugin's transport: Obsidian's `requestUrl` (runs outside the
// renderer's CORS rules; `rootmark serve` sends no CORS headers). `throw: false`
// so every HTTP status reaches `decode` (contract §3).
import { requestUrl } from "obsidian";
import type { Transport } from "./api";

export const obsidianTransport: Transport = async (req) => {
  const res = await requestUrl({
    url: req.url,
    method: req.method,
    headers: req.headers,
    ...(req.body === undefined ? {} : { body: req.body }),
    throw: false,
  });
  return { status: res.status, text: res.text };
};
