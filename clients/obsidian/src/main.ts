import { MarkdownView, Notice, Plugin, TFile } from "obsidian";
import { connect, describeError, type ConnectionState, type MdkClient } from "./api";
import { align, editorBlocks } from "./align";
import { blameOf, historyOf, listProposals, type ProposalRow } from "./commands";
import { gutterMarks } from "./view";
import { blameGutter, setMarks } from "./ui/gutter";
import { HistoryModal } from "./ui/history-modal";
import { ProposalsView, PROPOSALS_VIEW } from "./ui/proposals-view";
import { obsidianTransport } from "./obsidian-transport";

/** `X-Consumer` of the plugin: the trace label of its reads (contract §1). */
export const CONSUMER = "mdkernel-obsidian/0.1.0";

/** The mdkernel plugin (P1-E). Talks to a running `mdk serve` over the HTTP
 * contract only; without one it stays passive and says why (it never
 * starts a server) and it never writes a vault file. */
export default class MdkernelPlugin extends Plugin {
  state: ConnectionState = { kind: "offline", reason: "not connected yet" };
  private statusEl: HTMLElement | null = null;

  override async onload(): Promise<void> {
    this.statusEl = this.addStatusBarItem();
    this.registerEditorExtension(blameGutter((lineage) => void this.showHistory(lineage)));
    this.registerView(PROPOSALS_VIEW, (leaf) => new ProposalsView(leaf, this));
    this.addCommand({ id: "reconnect", name: "Reconnect to mdk serve", callback: () => void this.reconnect() });
    this.addCommand({ id: "proposals", name: "Show proposal blocks", callback: () => void this.openProposals() });
    this.addCommand({ id: "blame", name: "Refresh blame of the active note", callback: () => void this.refreshBlame() });
    this.registerEvent(this.app.workspace.on("active-leaf-change", () => void this.refreshBlame()));
    this.registerEvent(this.app.metadataCache.on("changed", () => void this.refreshBlame()));
    await this.reconnect();
  }

  client(): MdkClient | null {
    return this.state.kind === "online" ? this.state.client : null;
  }

  offlineReason(): string {
    return this.state.kind === "offline" ? this.state.reason : "";
  }

  async reconnect(): Promise<void> {
    const adapter = this.app.vault.adapter;
    this.state = await connect(
      { read: (p) => adapter.read(p), exists: (p) => adapter.exists(p) },
      obsidianTransport,
      CONSUMER,
    );
    const online = this.state.kind === "online";
    this.statusEl?.setText(
      this.state.kind === "online"
        ? `mdk ${this.state.status.mdk_version} · ${this.state.settings.humanId === null ? "no human_id" : `human:${this.state.settings.humanId}`}`
        : "mdk: offline",
    );
    this.statusEl?.setAttr("title", this.state.kind === "offline" ? this.state.reason : "mdk serve connected");
    if (online) await this.refreshBlame();
  }

  /** The block ranges of a note from Obsidian's own block map. */
  private rangesOf(file: TFile): ReturnType<typeof editorBlocks> {
    const c = this.app.metadataCache.getFileCache(file);
    if (!c?.sections) return [];
    return editorBlocks(
      c.sections.map((s) => ({ type: s.type, start: s.position.start.line, end: s.position.end.line })),
      (c.listItems ?? []).map((l) => ({ start: l.position.start.line, end: l.position.end.line, parent: l.parent })),
      (c.headings ?? []).map((h) => ({ line: h.position.start.line, heading: h.heading })),
    );
  }

  async refreshBlame(): Promise<void> {
    const view = this.app.workspace.getActiveViewOfType(MarkdownView);
    const client = this.client();
    if (!view?.file || client === null) return;
    // the editor's CodeMirror view (Obsidian exposes it as `editor.cm`)
    const cm = (view.editor as unknown as { cm?: { dispatch: (t: unknown) => void } }).cm;
    try {
      const blocks = await blameOf(client, view.file.path);
      const al = blocks === null ? { aligned: false as const, reason: "not in the mdk log" } : align(blocks, this.rangesOf(view.file));
      cm?.dispatch({ effects: setMarks.of(gutterMarks(al)) });
      if (!al.aligned && blocks !== null) this.statusEl?.setAttr("title", al.reason);
    } catch (e) {
      new Notice(`mdkernel blame: ${describeError(e)}`);
    }
  }

  async showHistory(lineage: string): Promise<void> {
    const client = this.client();
    if (client === null) return;
    try {
      new HistoryModal(this.app, await historyOf(client, lineage)).open();
    } catch (e) {
      new Notice(`mdkernel history: ${describeError(e)}`);
    }
  }

  async proposals(): Promise<ProposalRow[]> {
    const client = this.client();
    if (client === null) return [];
    const files = this.app.vault.getMarkdownFiles();
    const byPath = new Map(files.map((f) => [f.path, f]));
    return listProposals(
      client,
      files.map((f) => f.path),
      (doc, i) => {
        const f = byPath.get(doc);
        return f ? (this.rangesOf(f)[i]?.heading ?? null) : null;
      },
    );
  }

  async openProposals(): Promise<void> {
    const existing = this.app.workspace.getLeavesOfType(PROPOSALS_VIEW)[0];
    const leaf = existing ?? this.app.workspace.getRightLeaf(false);
    if (!leaf) return;
    await leaf.setViewState({ type: PROPOSALS_VIEW, active: true });
    void this.app.workspace.revealLeaf(leaf);
  }
}
