// E1 contract tests (TASKS E1): every tool's request against a fake
// `mdk serve` is exactly the contract's (docs/CLIENT_CONTRACT.md §1);
// every §3 code maps to an `MdkError` with that code (and the hint);
// no server → passive state; a tool outside the contract does not compile.
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
  connect,
  describeError,
  ERROR_STATUS,
  MdkClient,
  MdkError,
  readSettings,
  tomlValue,
  TOOLS,
  type ToolArgs,
  type ToolName,
  type VaultReader,
} from "../src/api";
import { fetchTransport, startFake, type Fake } from "./fake-server";

const TOKEN = "a".repeat(64);
let fake: Fake;
let client: MdkClient;

beforeEach(async () => {
  fake = await startFake();
  client = new MdkClient({ baseUrl: fake.url, token: TOKEN, consumer: "mdkernel-obsidian/0.1.0" }, fetchTransport);
});
afterEach(async () => {
  await fake.close();
});

/** One valid argument object per contract tool. */
const ARGS: { [T in ToolName]: ToolArgs[T] } = {
  status: {},
  search: { query: "kalimba", k: 5, mode: "lexical", include_proposal: true },
  read_block: { doc_path: "a.md", lineage_id: "0".repeat(32) },
  history: { lineage_id: "0".repeat(32) },
  blame: { doc_path: "a.md", at: "yesterday" },
  propose: { doc_path: "agent/p.md", content: "# P", reason: "r", mode: "create" },
  promote: { lineage_id: "0".repeat(32), reason: "approved" },
  checkpoint: { name: "yesterday" },
  checkpoints: {},
  trace: { since_seq: 0, consumer: "scene-agent/1.0" },
};

describe("requests (contract §1)", () => {
  it.each(TOOLS.map((t) => [t]))("%s: POST /v1/<tool>, bearer token, X-Consumer, JSON body = arguments", async (tool) => {
    await client.call(tool, ARGS[tool]);
    expect(fake.requests).toHaveLength(1);
    const r = fake.requests[0];
    expect(r?.method).toBe("POST");
    expect(r?.path).toBe(`/v1/${tool}`);
    expect(r?.headers.authorization).toBe(`Bearer ${TOKEN}`);
    expect(r?.headers["x-consumer"]).toBe("mdkernel-obsidian/0.1.0");
    expect(r?.headers["content-type"]).toBe("application/json");
    expect(JSON.parse(r?.body ?? "")).toEqual(ARGS[tool]);
  });

  it("the contract has exactly the ten tools of §2", () => {
    expect([...TOOLS].sort()).toEqual(
      ["blame", "checkpoint", "checkpoints", "history", "promote", "propose", "read_block", "search", "status", "trace"],
    );
  });

  it("status() is GET /v1/status without a body", async () => {
    await client.status();
    const r = fake.requests[0];
    expect(r?.method).toBe("GET");
    expect(r?.path).toBe("/v1/status");
    expect(r?.body).toBe("");
    expect(r?.headers["content-type"]).toBeUndefined();
  });

  it("a tool outside the contract or a wrong argument does not compile", () => {
    // these lines are type-checked by `tsc --noEmit` (npm run lint); each
    // directive fails the build if its call ever type-checks
    const typeOnly = (): void => {
      // @ts-expect-error demote is not a contract tool
      void client.call("demote", { lineage_id: "x" });
      // @ts-expect-error unknown argument (schemas are closed)
      void client.call("search", { query: "x", limit: 3 });
      // @ts-expect-error missing required reason
      void client.call("promote", { lineage_id: "x" });
      // @ts-expect-error mode outside the enum
      void client.call("search", { query: "x", mode: "fuzzy" });
    };
    expect(typeof typeOnly).toBe("function");
  });
});

describe("errors (contract §3)", () => {
  it.each(Object.entries(ERROR_STATUS))("%s (HTTP %i) → MdkError with that code", async (code, status) => {
    fake.reply(status, { error: { code, message: `m-${code}` } });
    const e = await client.call("search", { query: "x" }).catch((x: unknown) => x);
    expect(e).toBeInstanceOf(MdkError);
    expect((e as MdkError).code).toBe(code);
    expect((e as MdkError).status).toBe(status);
    expect((e as MdkError).message).toBe(`m-${code}`);
  });

  it("the hint travels and is shown (K-E0.4)", async () => {
    fake.reply(422, {
      error: { code: "policy_violation", message: "denied by edit policy", hint: "start the appended text with a heading" },
    });
    const e = (await client.call("propose", ARGS.propose).catch((x: unknown) => x)) as MdkError;
    expect(e.hint).toBe("start the appended text with a heading");
    expect(describeError(e)).toContain("start the appended text with a heading");
  });

  it("an answer outside the contract is bad_response, never a silent success", async () => {
    fake.reply(500, "<html>oops</html>");
    expect(((await client.call("status", {}).catch((x: unknown) => x)) as MdkError).code).toBe("bad_response");
    fake.reply(418, { error: { code: "teapot", message: "x" } });
    expect(((await client.call("status", {}).catch((x: unknown) => x)) as MdkError).code).toBe("bad_response");
    fake.reply(200, { error: { code: "internal", message: "x" } });
    expect(((await client.call("status", {}).catch((x: unknown) => x)) as MdkError).code).toBe("bad_response");
  });

  it("a promote refusal is an answer, not an error (§2.4)", async () => {
    fake.reply(200, { decision: "denied", lineage_id: "l", layer: "proposal", reason: "r", actor: { kind: "agent", id: "a/1" }, policy: "human_only", policy_version: "1" });
    const r = await client.call("promote", ARGS.promote);
    expect(r.decision).toBe("denied");
  });
});

/** An in-memory vault (read-only by type). */
const vaultOf = (files: Record<string, string>): VaultReader => ({
  read: (p) => (p in files ? Promise.resolve(files[p] ?? "") : Promise.reject(new Error(`no ${p}`))),
  exists: (p) => Promise.resolve(p in files),
});

describe("connection (contract §1; GOAL: no server → passive)", () => {
  const policy = '[promotion]\npolicy = "human_only"\n\n[server]\nhttp_port = 7357 # the port\n\n[identity]\nhuman_id = "ada"\n';

  it("reads [server] http_port, [identity] human_id and the token", async () => {
    const s = await readSettings(vaultOf({ ".mdkernel/policy.toml": policy, ".mdkernel/server.token": `${TOKEN}\n` }));
    expect(s).toEqual({ port: 7357, token: TOKEN, humanId: "ada" });
    expect(tomlValue(policy, "promotion", "policy")).toBe("human_only");
    expect(tomlValue(policy, "server", "human_id")).toBeNull();
  });

  it("without http_port, token or policy: passive with the reason", async () => {
    for (const [files, why] of [
      [{}, "policy.toml not found"],
      [{ ".mdkernel/policy.toml": '[server]\nmax_propose_bytes = 4\n' }, "http_port"],
      [{ ".mdkernel/policy.toml": policy }, "server.token not found"],
      [{ ".mdkernel/policy.toml": policy, ".mdkernel/server.token": "short" }, "64-hex"],
    ] as const) {
      const st = await connect(vaultOf(files), fetchTransport, "c");
      expect(st.kind).toBe("offline");
      expect(st.kind === "offline" ? st.reason : "").toContain(why);
    }
  });

  it("nothing listening: passive (unreachable), never started", async () => {
    const port = Number(new URL(fake.url).port);
    await fake.close();
    const p = policy.replace("7357", String(port));
    const st = await connect(vaultOf({ ".mdkernel/policy.toml": p, ".mdkernel/server.token": TOKEN }), fetchTransport, "c");
    expect(st.kind).toBe("offline");
    expect(st.kind === "offline" ? st.reason : "").toMatch(/^unreachable/);
    fake = await startFake(); // for afterEach
  });

  it("a server that answers status: online", async () => {
    const port = Number(new URL(fake.url).port);
    fake.reply(200, { vault: "/v", spec_version: "1.0.3", mdk_version: "0.2.0", level_claimed: 3 });
    const p = policy.replace("7357", String(port));
    const st = await connect(vaultOf({ ".mdkernel/policy.toml": p, ".mdkernel/server.token": TOKEN }), fetchTransport, "c");
    expect(st.kind).toBe("online");
    expect(fake.requests[0]?.path).toBe("/v1/status");
  });
});
