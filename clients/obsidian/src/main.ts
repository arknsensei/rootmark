import { Plugin } from "obsidian";
import { connect, type ConnectionState } from "./api";
import { obsidianTransport } from "./obsidian-transport";

/** `X-Consumer` of the plugin: the trace label of its reads (contract §1). */
export const CONSUMER = "mdkernel-obsidian/0.1.0";

/** The mdkernel plugin (P1-E). Connects to a running `mdk serve` over the
 * HTTP contract; without one it stays passive and says why (it never
 * starts a server). */
export default class MdkernelPlugin extends Plugin {
  state: ConnectionState = { kind: "offline", reason: "not connected yet" };
  private statusEl: HTMLElement | null = null;

  override async onload(): Promise<void> {
    this.statusEl = this.addStatusBarItem();
    this.addCommand({ id: "reconnect", name: "Reconnect to mdk serve", callback: () => void this.reconnect() });
    await this.reconnect();
  }

  async reconnect(): Promise<void> {
    const adapter = this.app.vault.adapter;
    this.state = await connect(
      { read: (p) => adapter.read(p), exists: (p) => adapter.exists(p) },
      obsidianTransport,
      CONSUMER,
    );
    this.statusEl?.setText(
      this.state.kind === "online"
        ? `mdk ${this.state.status.mdk_version} · spec ${this.state.status.spec_version}`
        : "mdk: offline",
    );
    this.statusEl?.setAttr("title", this.state.kind === "offline" ? this.state.reason : "mdk serve connected");
  }
}
