// A separate Obsidian instance driven over the Chrome DevTools Protocol
// (K-E1.3; the P1-E verifier's method, reports/rounds/P1-E/VERIFICATION.md
// V3): its own --user-data-dir with a pre-written obsidian.json (the vault
// registry), so the user's Obsidian and its configuration are never touched
// (Electron's single-instance lock is per user-data-dir). Only the process
// started here is ever stopped. Node only; not part of the plugin bundle.
import { spawn, type ChildProcess } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const sleep = (ms: number): Promise<void> => new Promise((ok) => setTimeout(ok, ms));

export interface Page {
  /** evaluates `expr` (awaited) in the Obsidian window; its JSON value */
  ev<T = unknown>(expr: string): Promise<T>;
  /** polls `expr` until truthy; throws `what` after `ms` */
  waitFor(expr: string, what: string, ms?: number): Promise<void>;
  close(): Promise<number | null>;
}

/** Starts Obsidian on `vault` with its own data dir `dataDir` and CDP on
 * `port`; resolves once the workspace layout is ready. */
export async function launchObsidian(bin: string, dataDir: string, vault: string, port: number): Promise<Page> {
  mkdirSync(dataDir, { recursive: true });
  // the vault registry of this instance only (fixed id, no time stamp)
  writeFileSync(join(dataDir, "obsidian.json"), JSON.stringify({ vaults: { "0123456789abcdef": { path: vault, ts: 1, open: true } } }));
  const child: ChildProcess = spawn(bin, [`--user-data-dir=${dataDir}`, `--remote-debugging-port=${String(port)}`], {
    stdio: ["ignore", "ignore", "pipe"],
  });
  let err = "";
  child.stderr?.on("data", (d: Buffer) => (err += d.toString("utf8")));
  let exited: number | null | undefined;
  child.on("exit", (c) => (exited = c));
  const stop = (): Promise<number | null> =>
    new Promise((ok) => {
      if (exited !== undefined) {
        ok(exited);
        return;
      }
      const t = setTimeout(() => child.kill("SIGKILL"), 15_000);
      child.on("exit", (c) => {
        clearTimeout(t);
        ok(c);
      });
      child.kill("SIGTERM");
    });
  try {
    let url: string | undefined;
    for (let i = 0; i < 120 && url === undefined; i++) {
      if (exited !== undefined) throw new Error(`Obsidian exited (${String(exited)}): ${err.slice(-400)}`);
      try {
        const list = (await (await fetch(`http://127.0.0.1:${String(port)}/json/list`)).json()) as { type: string; url: string; webSocketDebuggerUrl: string }[];
        url = list.find((t) => t.type === "page" && t.url.startsWith("app://obsidian.md/index.html"))?.webSocketDebuggerUrl;
      } catch {
        // not listening yet
      }
      if (url === undefined) await sleep(500);
    }
    if (url === undefined) throw new Error(`no Obsidian window on CDP port ${String(port)}: ${err.slice(-400)}`);
    const ws = new WebSocket(url);
    await new Promise<void>((ok, fail) => {
      ws.onopen = () => { ok(); };
      ws.onerror = () => { fail(new Error("CDP websocket error")); };
    });
    let id = 0;
    const pending = new Map<number, (m: { result?: { result?: { value?: unknown }; exceptionDetails?: { exception?: { description?: string }; text?: string } } }) => void>();
    ws.onmessage = (m: MessageEvent) => {
      const d = JSON.parse(String(m.data)) as { id?: number };
      if (typeof d.id === "number" && pending.has(d.id)) {
        const f = pending.get(d.id);
        pending.delete(d.id);
        f?.(d as never);
      }
    };
    const ev = <T>(expr: string): Promise<T> =>
      new Promise((ok, fail) => {
        const i = ++id;
        const timer = setTimeout(() => {
          pending.delete(i);
          fail(new Error(`CDP evaluate timed out: ${expr.slice(0, 120)}`));
        }, 60_000);
        pending.set(i, (m) => {
          clearTimeout(timer);
          const ex = m.result?.exceptionDetails;
          if (ex) fail(new Error(`in Obsidian: ${ex.exception?.description ?? ex.text ?? "exception"} (${expr.slice(0, 120)})`));
          else ok(m.result?.result?.value as T);
        });
        ws.send(JSON.stringify({ id: i, method: "Runtime.evaluate", params: { expression: `(async () => (${expr}))()`, awaitPromise: true, returnByValue: true } }));
      });
    const waitFor = async (expr: string, what: string, ms = 30_000): Promise<void> => {
      const t0 = Date.now();
      for (;;) {
        let v: unknown;
        try {
          v = await ev(expr);
        } catch {
          v = false;
        }
        if (v) return;
        if (Date.now() - t0 > ms) throw new Error(`timed out waiting for ${what}`);
        await sleep(200);
      }
    };
    await waitFor("typeof app !== 'undefined' && app.workspace.layoutReady === true", "the Obsidian workspace", 60_000);
    return {
      ev,
      waitFor,
      close: async () => {
        ws.close();
        return stop();
      },
    };
  } catch (e) {
    await stop();
    throw e;
  }
}

/** Helpers defined in the Obsidian window (`window.__mdk`): the plugin's
 * views are driven through their own DOM (inputs, buttons, modal), as a
 * person would. Every helper throws when its element is not there. */
export const PAGE_HELPERS = `(window.__mdk = {
  view(type) {
    const el = document.querySelector('.workspace-leaf-content[data-type="' + type + '"] .view-content');
    if (!el) throw new Error('view ' + type + ' not open');
    return el;
  },
  setting(type, name) {
    const items = [...this.view(type).querySelectorAll('.setting-item')];
    const it = items.find((i) => (i.querySelector('.setting-item-name')?.textContent ?? '') === name);
    if (!it) throw new Error('setting ' + name + ' not in ' + type);
    return it;
  },
  type(input, value) {
    input.value = value;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.dispatchEvent(new Event('change', { bubbles: true }));
  },
  field(type, name, value) {
    const it = this.setting(type, name);
    const el = it.querySelector('input[type="text"], input:not([type]), select');
    if (!el) throw new Error('no field in setting ' + name);
    this.type(el, value);
  },
  check(type, name, on) {
    const cb = this.setting(type, name).querySelector('input[type="checkbox"]');
    if (!cb) throw new Error('no checkbox in setting ' + name);
    if (cb.checked !== on) cb.click();
  },
  button(root, text) {
    const b = [...root.querySelectorAll('button')].find((x) => x.textContent === text);
    if (!b) throw new Error('no button ' + text);
    return b;
  },
  statusBar() {
    const it = [...document.querySelectorAll('.status-bar-item')].find((e) => (e.textContent ?? '').startsWith('mdk'));
    return it ? { text: it.textContent, title: it.getAttribute('title') } : null;
  },
  notices() {
    return [...document.querySelectorAll('.notice')].map((n) => n.textContent);
  },
  gutter() {
    const ed = app.workspace.activeLeaf?.view?.containerEl;
    if (!ed) return [];
    return [...ed.querySelectorAll('.mdk-gutter .mdk-mark')].map((m) => ({ label: m.textContent, title: m.getAttribute('title'), cls: m.className }));
  },
  async open(path) {
    const f = app.vault.getAbstractFileByPath(path);
    if (!f) throw new Error(path + ' not in the vault');
    await app.workspace.getLeaf(false).openFile(f);
  },
}, true)`;
