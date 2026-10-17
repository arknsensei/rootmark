import { Modal, Setting, type App } from "obsidian";

/** Asks the human for a reason; resolves null on cancel. The button stays
 * disabled while the reason is empty (contract §2: `reason` not empty). */
export function askReason(app: App, title: string): Promise<string | null> {
  return new Promise((resolve) => {
    let done = false;
    const m = new Modal(app);
    m.titleEl.setText(title);
    let reason = "";
    const area = m.contentEl.createEl("textarea", { cls: "mdk-reason", attr: { rows: "3", placeholder: "Reason (required)" } });
    const s = new Setting(m.contentEl);
    s.addButton((b) => {
      b.setButtonText("Promote").setCta().setDisabled(true).onClick(() => {
        done = true;
        m.close();
        resolve(reason.trim());
      });
      area.addEventListener("input", () => {
        reason = area.value;
        b.setDisabled(reason.trim() === "");
      });
    });
    s.addButton((b) => b.setButtonText("Cancel").onClick(() => { m.close(); }));
    m.onClose = () => {
      if (!done) resolve(null);
    };
    m.open();
  });
}
