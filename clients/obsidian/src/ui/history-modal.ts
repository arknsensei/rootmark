import { Modal, type App } from "obsidian";
import type { HistoryResult } from "../api";
import { historyLine } from "../view";

/** The `history` of one lineage (click on a gutter mark). */
export class HistoryModal extends Modal {
  constructor(app: App, private readonly h: HistoryResult) {
    super(app);
  }

  override onOpen(): void {
    this.titleEl.setText(`Lineage ${this.h.lineage_id.slice(0, 8)} · ${this.h.layer}`);
    const ul = this.contentEl.createEl("ul", { cls: "rootmark-history" });
    for (const e of this.h.events) ul.createEl("li", { text: historyLine(e) });
  }

  override onClose(): void {
    this.contentEl.empty();
  }
}
