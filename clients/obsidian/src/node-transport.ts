// The Node transport of the API client (tests, e2e harness): global fetch.
// The plugin uses Obsidian's `requestUrl` instead (CORS-free); both send
// exactly the request `MdkClient` builds.
import type { Transport } from "./api";

export const fetchTransport: Transport = async (req) => {
  const res = await fetch(req.url, {
    method: req.method,
    headers: req.headers,
    ...(req.body === undefined ? {} : { body: req.body }),
  });
  return { status: res.status, text: await res.text() };
};
