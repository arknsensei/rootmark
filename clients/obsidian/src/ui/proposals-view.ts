// The proposal view: every `proposal` block of the vault (document,
// section, source, log offset), Promote over HTTP with a required reason;
// Demote shown disabled — not in the contract (K-E2-T).
import { ItemView, Notice, Setting, TFile, type WorkspaceLeaf } from "obsidian";
import { describeError } from "../api";
import { DEMOTE_UNAVAILABLE, promoteAll, promoteSummary, type ProposalRow } from "../commands";
import type MdkernelPlugin from "../main";
import { askReason } from "./reason-modal";

export const PROPOSALS_VIEW = "mdkernel-proposals";

export class ProposalsView extends ItemView {
  private rows: ProposalRow[] = [];
  private selected = new Set<string>();

  constructor(leaf: WorkspaceLeaf, private readonly plugin: MdkernelPlugin) {
    super(leaf);
  }

  override getViewType(): string {
    return PROPOSALS_VIEW;
  }
  override getDisplayText(): string {
    return "mdkernel proposals";
  }
  override getIcon(): string {
    return "git-pull-request";
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
    try {
      this.rows = await this.plugin.proposals();
    } catch (e) {
      c.createEl("p", { cls: "mdk-error", text: describeError(e) });
      return;
    }
    const bar = new Setting(c).setName(`${String(this.rows.length)} proposal blocks`);
    bar.addButton((b) =>
      b.setButtonText("Promote selected").setCta().onClick(() => {
        void this.promoteSelected();
      }),
    );
    bar.addButton((b) => b.setButtonText("Demote").setDisabled(true).setTooltip(DEMOTE_UNAVAILABLE));
    bar.addButton((b) => b.setButtonText("Refresh").onClick(() => void this.refresh()));
    const table = c.createEl("table", { cls: "mdk-proposals" });
    const head = table.createEl("tr");
    for (const h of ["", "Document", "Section", "Source", "Seq"]) head.createEl("th", { text: h });
    for (const r of this.rows) {
      const tr = table.createEl("tr", { cls: "mdk-layer-proposal" });
      const cb = tr.createEl("td").createEl("input", { type: "checkbox" });
      cb.checked = this.selected.has(r.lineage_id);
      cb.addEventListener("change", () => {
        this.selected = cb.checked
          ? new Set([...this.selected, r.lineage_id])
          : new Set([...this.selected].filter((x) => x !== r.lineage_id));
      });
      const doc = tr.createEl("td").createEl("a", { text: r.doc_path, href: "#" });
      doc.addEventListener("click", (ev) => {
        ev.preventDefault();
        this.openDoc(r.doc_path);
      });
      tr.createEl("td", { text: r.section ?? "—" });
      tr.createEl("td", { text: r.source === "" ? "—" : r.source });
      tr.createEl("td", { text: String(r.seq_created) });
    }
  }

  /** Opens an existing note only: a path that is gone is an error, never a
   * new note (the plugin writes no file). */
  private openDoc(path: string): void {
    const f = this.app.vault.getAbstractFileByPath(path);
    if (!(f instanceof TFile)) {
      new Notice(`mdkernel: ${path} is not in the vault (moved or deleted)`);
      return;
    }
    void this.app.workspace.getLeaf(false).openFile(f);
  }

  private async promoteSelected(): Promise<void> {
    const client = this.plugin.client();
    if (client === null || this.selected.size === 0) {
      new Notice(client === null ? "mdk serve not connected" : "Select proposal blocks first");
      return;
    }
    const reason = await askReason(this.app, `Promote ${String(this.selected.size)} block(s)`);
    if (reason === null) return;
    try {
      const outcomes = await promoteAll(client, [...this.selected], reason);
      new Notice(`mdkernel: ${promoteSummary(outcomes)}`, 8000);
    } catch (e) {
      new Notice(`mdkernel promote: ${describeError(e)}`, 8000);
    }
    this.selected = new Set();
    await this.refresh();
    await this.plugin.refreshBlame();
  }
}
