// Checkpoints: the list (creation order), naming the current state, and a
// read-only past view of the active note (`blame` at a checkpoint).
import { ItemView, Notice, Setting, type WorkspaceLeaf } from "obsidian";
import { describeError } from "../api";
import { blameAt, listCheckpoints, nameCheckpoint } from "../commands";
import type MdkernelPlugin from "../main";
import { pastView } from "../view";

export const CHECKPOINTS_VIEW = "mdkernel-checkpoints";

export class CheckpointsView extends ItemView {
  private name = "";

  constructor(leaf: WorkspaceLeaf, private readonly plugin: MdkernelPlugin) {
    super(leaf);
  }
  override getViewType(): string {
    return CHECKPOINTS_VIEW;
  }
  override getDisplayText(): string {
    return "mdkernel checkpoints";
  }
  override getIcon(): string {
    return "history";
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
    new Setting(c)
      .setName("Name the current state")
      .addText((t) => t.setPlaceholder("yesterday").onChange((v) => (this.name = v)))
      .addButton((b) =>
        b.setButtonText("Name").onClick(async () => {
          try {
            const cp = await nameCheckpoint(client, this.name);
            new Notice(`mdkernel: checkpoint ${cp.checkpoint_id.slice(0, 8)} named ${cp.name ?? ""}`);
          } catch (e) {
            new Notice(`mdkernel checkpoint: ${describeError(e)}`, 8000);
          }
          await this.refresh();
        }),
      );
    let cps;
    try {
      cps = await listCheckpoints(client);
    } catch (e) {
      c.createEl("p", { cls: "mdk-error", text: describeError(e) });
      return;
    }
    const doc = this.app.workspace.getActiveFile()?.path ?? null;
    const ul = c.createEl("ul");
    for (const cp of cps) {
      const li = ul.createEl("li");
      li.createSpan({ text: `${cp.name ?? "(unnamed)"} · ${cp.checkpoint_id.slice(0, 8)} · seq ${String(cp.event_seq)} · ${String(Object.keys(cp.docs).length)} docs ` });
      if (doc !== null) {
        const b = li.createEl("button", { text: `blame ${doc} then` });
        b.addEventListener("click", () => void this.past(li, doc, cp.checkpoint_id));
      }
    }
  }

  /** The note as of a checkpoint: read only (no editor, no write). */
  private async past(el: HTMLElement, doc: string, at: string): Promise<void> {
    const client = this.plugin.client();
    if (client === null) {
      new Notice(`mdk serve not connected: ${this.plugin.offlineReason()}`);
      return;
    }
    const pre = el.createEl("pre", { cls: "mdk-past" });
    try {
      pre.setText(pastView(doc, at, await blameAt(client, doc, at)));
    } catch (e) {
      pre.setText(describeError(e));
    }
  }
}
