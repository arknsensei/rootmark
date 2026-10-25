import { MarkdownView, Notice, Plugin, TFile } from "obsidian";
import { connect, describeError, type ConnectionState, type RootmarkClient } from "./api";
import { align, blockLine, editorBlocks } from "./align";
import { alignNote, blameOf, blameResultOf, historyOf, listProposals, type ProposalRow } from "./commands";
import { gutterMarks, indexNotice } from "./view";
import { blameGutter, setMarks } from "./ui/gutter";
import { HistoryModal } from "./ui/history-modal";
import { ProposalsView, PROPOSALS_VIEW } from "./ui/proposals-view";
import { SearchView, SEARCH_VIEW } from "./ui/search-view";
import { CheckpointsView, CHECKPOINTS_VIEW } from "./ui/checkpoints-view";
import { TracesView, TRACES_VIEW } from "./ui/traces-view";
import { obsidianTransport } from "./obsidian-transport";
import { Attester } from "./attest";
import { observeOwnWrites, type SavedHow } from "./own-writes";

import { CONSUMER } from "./consumer";

/** The rootmark plugin (P1-E). Talks to a running `rootmark serve` over the HTTP
 * contract only; without one it stays passive and says why (it never
 * starts a server) and it never writes a vault file. */
export default class RootmarkPlugin extends Plugin {
  state: ConnectionState = { kind: "offline", reason: "not connected yet" };
  /** the index line of the connection status (K-E1.2); null when `ok` */
  private indexDetail: string | null = null;
  private statusEl: HTMLElement | null = null;
  /** P2-0 (spec 2.0.0 §7.1): attests every save this Obsidian makes */
  private attester: Attester | null = null;
  private attestEl: HTMLElement | null = null;
  private restoreAdapter: (() => void) | null = null;
  /** saves made while no server was connected (not attested) */
  private offlineSaves = 0;
  /** attestations are sent one after another, in the order this Obsidian
   * wrote (a rename reaches the server before the link updates it caused:
   * P2-0.1 Y1 security review B6) */
  private attestQueue: Promise<void> = Promise.resolve();

  override async onload(): Promise<void> {
    this.statusEl = this.addStatusBarItem();
    this.attestEl = this.addStatusBarItem();
    this.observeOwnWrites();
    this.registerEditorExtension(blameGutter((lineage) => void this.showHistory(lineage)));
    this.registerView(PROPOSALS_VIEW, (leaf) => new ProposalsView(leaf, this));
    this.registerView(SEARCH_VIEW, (leaf) => new SearchView(leaf, this));
    this.registerView(CHECKPOINTS_VIEW, (leaf) => new CheckpointsView(leaf, this));
    this.registerView(TRACES_VIEW, (leaf) => new TracesView(leaf, this));
    this.addCommand({ id: "reconnect", name: "Reconnect to rootmark serve", callback: () => void this.reconnect() });
    this.addCommand({ id: "proposals", name: "Show proposal blocks", callback: () => void this.openView(PROPOSALS_VIEW) });
    this.addCommand({ id: "search", name: "Search", callback: () => void this.openView(SEARCH_VIEW) });
    this.addCommand({ id: "checkpoints", name: "Checkpoints", callback: () => void this.openView(CHECKPOINTS_VIEW) });
    this.addCommand({ id: "traces", name: "Traces", callback: () => void this.openView(TRACES_VIEW) });
    this.addCommand({ id: "blame", name: "Refresh blame of the active note", callback: () => void this.refreshBlame() });
    this.registerEvent(this.app.workspace.on("active-leaf-change", () => void this.refreshBlame()));
    this.registerEvent(this.app.metadataCache.on("changed", () => void this.refreshBlame()));
    await this.reconnect();
  }

  override onunload(): void {
    this.restoreAdapter?.();
    this.restoreAdapter = null;
    this.attester = null;
  }

  /** Attests the saves, appends and renames of this Obsidian, and only
   * those (`own-writes.ts`, ASSUMPTIONS Z3-1). */
  private observeOwnWrites(): void {
    this.restoreAdapter = observeOwnWrites(
      this.app.vault.adapter,
      // every rename is attested (P2-0.2 T2): the server checks the renamed
      // bytes are the old path's head and carries each block's layer over as
      // it is — an agent's proposal or an unverified edit stays what it was
      // (before P2-0.2 only an all-canonical note was reported, Z3 review 11)
      (path, bytes, how) => {
        this.attestQueue = this.attestQueue.then(() => this.attest(path, bytes, how));
      },
    );
  }

  private async attest(path: string, bytes: Uint8Array, how: SavedHow): Promise<void> {
    const a = this.attester;
    if (a === null) {
      // offline: the save is not attested (it will be `unverified` if the
      // server sees it); counted, never silent
      this.offlineSaves += 1;
      this.showAttestCounts();
      return;
    }
    try {
      await a.attest(path, bytes, null, how);
    } catch (e) {
      new Notice(`rootmark attest: ${describeError(e)}`, 6000);
    }
    this.showAttestCounts();
  }

  private showAttestCounts(): void {
    const a = this.attester;
    const failed = (a?.counts.failed ?? 0) + this.offlineSaves;
    this.attestEl?.setText(a === null && failed === 0 ? "" : `attested ${String(a?.counts.attested ?? 0)} · unattested ${String(failed)}`);
    this.attestEl?.setAttr(
      "title",
      a?.counts.lastError ??
        (this.offlineSaves > 0
          ? `${String(this.offlineSaves)} save(s) while rootmark serve was not connected: not attested`
          : "every save of this Obsidian is attested to rootmark serve; unattested = saves whose attestation failed or was not sent"),
    );
  }

  client(): RootmarkClient | null {
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
    this.attester = this.state.kind === "online" ? new Attester(this.state.client.conn, obsidianTransport, window.crypto.subtle) : null;
    this.showAttestCounts();
    // the index state (K-E1.2): `status` answers whatever it is; anything
    // but `ok` is shown with the way to rebuild
    const ix = this.state.kind === "online" ? indexNotice(this.state.status.index) : null;
    this.indexDetail = ix?.detail ?? null;
    this.statusEl?.setText(
      this.state.kind === "online"
        ? `rootmark ${this.state.status.mdk_version} · ${this.state.settings.humanId === null ? "no human_id" : `human:${this.state.settings.humanId}`}${ix === null ? "" : ` · ${ix.label}`}`
        : "rootmark: offline",
    );
    this.statusEl?.setAttr("title", this.state.kind === "offline" ? this.state.reason : ix === null ? "rootmark serve connected" : ix.detail);
    this.statusEl?.toggleClass("rootmark-index-warning", ix !== null);
    if (ix !== null) new Notice(`rootmark: ${ix.detail}`, 15000);
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

  /** The status-bar hint of the blame gutter: always the current state of
   * the active note (no stale reason survives a later success). */
  private gutterHint(text: string): void {
    // the index state stays first in the tooltip while it is not `ok`
    this.statusEl?.setAttr("title", this.indexDetail === null ? text : `${this.indexDetail}\n${text}`);
  }

  async refreshBlame(): Promise<void> {
    const view = this.app.workspace.getActiveViewOfType(MarkdownView);
    const client = this.client();
    if (!view?.file || client === null) return;
    // the editor's CodeMirror view (Obsidian exposes it as `editor.cm`)
    const cm = (view.editor as unknown as { cm?: { dispatch: (t: unknown) => void } }).cm;
    if (cm === undefined) {
      this.gutterHint("blame gutter unavailable: this editor exposes no CodeMirror view (use the proposal view)");
      return;
    }
    try {
      const r = await blameResultOf(client, view.file.path);
      if (r === null) {
        cm.dispatch({ effects: setMarks.of([]) });
        this.gutterHint(`${view.file.path} is not in the rootmark log: no blame`);
        return;
      }
      // contract v2: the server's locations, checked against the note's
      // bytes on disk (read only); else the editor's block map by order
      const bytes = r.blocks.some((b) => b.loc !== undefined)
        ? new Uint8Array(await this.app.vault.adapter.readBinary(view.file.path))
        : null;
      const c = this.app.metadataCache.getFileCache(view.file);
      const headings = (c?.headings ?? []).map((h) => ({ line: h.position.start.line, heading: h.heading }));
      const { al, by } = alignNote(r, bytes, this.rangesOf(view.file), headings);
      cm.dispatch({ effects: setMarks.of(gutterMarks(al)) });
      this.gutterHint(al.aligned ? `blame: ${String(r.blocks.length)} blocks aligned (${by === "loc" ? "by location" : "by order"})` : al.reason);
    } catch (e) {
      this.gutterHint(`blame failed: ${describeError(e)}`);
      new Notice(`rootmark blame: ${describeError(e)}`);
    }
  }

  /** Opens an existing note at the line of one of its blocks (trace
   * viewer); when the position is not known the note opens and the reason
   * is shown. Never creates a note. */
  async openAtBlock(path: string, lineage: string): Promise<void> {
    const f = this.app.vault.getAbstractFileByPath(path);
    if (!(f instanceof TFile)) {
      new Notice(`rootmark: ${path} is not in the vault (moved or deleted)`);
      return;
    }
    const leaf = this.app.workspace.getLeaf(false);
    await leaf.openFile(f);
    const client = this.client();
    if (client === null) return;
    try {
      const blocks = await blameOf(client, path);
      const at = blocks === null ? { reason: `${path} is not in the rootmark log` } : blockLine(blocks, this.rangesOf(f), lineage);
      if ("line" in at) {
        const v = leaf.view;
        if (v instanceof MarkdownView) {
          v.editor.setCursor({ line: at.line, ch: 0 });
          v.editor.scrollIntoView({ from: { line: at.line, ch: 0 }, to: { line: at.line, ch: 0 } }, true);
        } else {
          new Notice(`rootmark: opened ${path}; not a Markdown editor, block line ${String(at.line + 1)}`);
        }
      } else {
        new Notice(`rootmark: opened ${path}; block position unknown — ${at.reason}`, 8000);
      }
    } catch (e) {
      new Notice(`rootmark: ${describeError(e)}`);
    }
  }

  async showHistory(lineage: string): Promise<void> {
    const client = this.client();
    if (client === null) return;
    try {
      new HistoryModal(this.app, await historyOf(client, lineage)).open();
    } catch (e) {
      new Notice(`rootmark history: ${describeError(e)}`);
    }
  }

  async proposals(layer: "proposal" | "unverified" = "proposal"): Promise<ProposalRow[]> {
    const client = this.client();
    if (client === null) return [];
    const files = this.app.vault.getMarkdownFiles();
    const byPath = new Map(files.map((f) => [f.path, f]));
    return listProposals(
      client,
      files.map((f) => f.path),
      (doc, blocks) => {
        const f = byPath.get(doc);
        if (!f) return null;
        const al = align(blocks, this.rangesOf(f));
        return al.aligned ? al.pairs.map((p) => p.range.heading) : null;
      },
      [layer],
    );
  }

  async openView(type: string): Promise<void> {
    const existing = this.app.workspace.getLeavesOfType(type)[0];
    const leaf = existing ?? this.app.workspace.getRightLeaf(false);
    if (!leaf) return;
    await leaf.setViewState({ type, active: true });
    void this.app.workspace.revealLeaf(leaf);
  }
}
