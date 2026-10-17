import { Notice, TFile, type App } from "obsidian";

/** Opens an existing note only: a path that is gone is reported, never a
 * new note (the plugin writes no file). */
export function openDoc(app: App, path: string): void {
  const f = app.vault.getAbstractFileByPath(path);
  if (!(f instanceof TFile)) {
    new Notice(`rootmark: ${path} is not in the vault (moved or deleted)`);
    return;
  }
  void app.workspace.getLeaf(false).openFile(f);
}
