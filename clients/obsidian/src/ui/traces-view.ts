// Traces: the latest consumption traces and the blocks each answer rested
// on; a click opens the note of a returned block.
import { ItemView, type WorkspaceLeaf } from "obsidian";
import { describeError } from "../api";
import { recentTraces } from "../commands";
import type MdkernelPlugin from "../main";
import { openDoc } from "./open";

export const TRACES_VIEW = "mdkernel-traces";

export class TracesView extends ItemView {
  constructor(leaf: WorkspaceLeaf, private readonly plugin: MdkernelPlugin) {
    super(leaf);
  }
  override getViewType(): string {
    return TRACES_VIEW;
  }
  override getDisplayText(): string {
    return "mdkernel traces";
  }
  override getIcon(): string {
    return "list-tree";
  }
  override async onOpen(): Promise<void> {
    await this.refresh();
  }

  async refresh(): Promise<void> {
    const c = this.containerEl.children[1] as HTMLElement;
    c.empty();
    const client = this.plugin.client();
    if (client === null) {
      c.createEl("p", { text: `mdk serve not connected: ${this.plugin.offlineReason()}` });
      return;
    }
    const btn = c.createEl("button", { text: "Refresh" });
    btn.addEventListener("click", () => void this.refresh());
    try {
      for (const t of await recentTraces(client, { limit: 50 })) {
        const d = c.createEl("details");
        d.createEl("summary", { text: `#${String(t.at_seq)} · ${t.consumer} · ${String(t.returned.length)} blocks · checkpoint ${t.checkpoint_id.slice(0, 8)}` });
        const ul = d.createEl("ul");
        for (const r of t.returned) {
          const a = ul.createEl("li").createEl("a", { text: `${r.doc_path} · ${r.lineage_id.slice(0, 8)}`, href: "#" });
          a.addEventListener("click", (ev) => {
            ev.preventDefault();
            openDoc(this.app, r.doc_path);
          });
        }
      }
    } catch (e) {
      c.createEl("p", { cls: "mdk-error", text: describeError(e) });
    }
  }
}
