// The search panel: `search` with mode, k and include_proposal (and `at`, a
// checkpoint id or name: the state as it was); proposal hits apart, set off.
import { ItemView, Setting, type WorkspaceLeaf } from "obsidian";
import { describeError, type Hit } from "../api";
import { runSearch } from "../commands";
import type RootmarkPlugin from "../main";
import { openDoc } from "./open";

export const SEARCH_VIEW = "rootmark-search";

export class SearchView extends ItemView {
  private query = "";
  private mode: "hybrid" | "lexical" | "flat" = "hybrid";
  private k = 10;
  private includeProposal = false;
  private at = "";

  constructor(leaf: WorkspaceLeaf, private readonly plugin: RootmarkPlugin) {
    super(leaf);
  }
  override getViewType(): string {
    return SEARCH_VIEW;
  }
  override getDisplayText(): string {
    return "rootmark search";
  }
  override getIcon(): string {
    return "search";
  }

  override onOpen(): Promise<void> {
    this.render(null);
    return Promise.resolve();
  }

  private render(results: HTMLElement | null): void {
    const c = this.containerEl.children[1] as HTMLElement;
    c.empty();
    // form fields are set through their elements (the static no-write scan
    // forbids every `setValue`, the editor's included)
    new Setting(c).setName("Query").addText((t) => {
      t.inputEl.value = this.query;
      t.onChange((v) => (this.query = v));
    });
    new Setting(c).setName("Mode").addDropdown((d) => {
      d.addOptions({ hybrid: "hybrid", lexical: "lexical", flat: "flat" });
      d.selectEl.value = this.mode;
      d.onChange((v) => (this.mode = v as typeof this.mode));
    });
    new Setting(c).setName("k").addText((t) => {
      t.inputEl.value = String(this.k);
      t.onChange((v) => (this.k = Number(v)));
    });
    const inc = new Setting(c)
      .setName("Include proposals")
      .setDesc("unapproved agent text, labelled and listed after the canonical hits")
      .controlEl.createEl("input", { type: "checkbox" });
    inc.checked = this.includeProposal;
    inc.addEventListener("change", () => (this.includeProposal = inc.checked));
    new Setting(c).setName("At checkpoint").setDesc("id or name; empty: now").addText((t) => {
      t.inputEl.value = this.at;
      t.onChange((v) => (this.at = v));
    });
    new Setting(c).addButton((b) => b.setButtonText("Search").setCta().onClick(() => void this.search()));
    if (results) c.appendChild(results);
  }

  private async search(): Promise<void> {
    const out = createDiv();
    const client = this.plugin.client();
    if (client === null) {
      out.createEl("p", { text: `rootmark serve not connected: ${this.plugin.offlineReason()}` });
      this.render(out);
      return;
    }
    try {
      const r = await runSearch(client, this.query, {
        mode: this.mode,
        k: this.k,
        include_proposal: this.includeProposal,
        ...(this.at.trim() === "" ? {} : { at: this.at.trim() }),
      });
      out.createEl("p", { cls: "rootmark-meta", text: `checkpoint ${r.checkpoint_id.slice(0, 8)} · trace ${r.trace_id.slice(0, 8)}` });
      this.hits(out, r.canonical, "rootmark-hit-canonical");
      if (this.includeProposal) {
        out.createEl("h4", { text: `Proposals (not approved) · ${String(r.proposal.length)}` });
        this.hits(out, r.proposal, "rootmark-hit-proposal");
      }
    } catch (e) {
      out.createEl("p", { cls: "rootmark-error", text: describeError(e) });
    }
    this.render(out);
  }

  private hits(el: HTMLElement, hits: Hit[], cls: string): void {
    const ol = el.createEl("ol");
    for (const h of hits) {
      const li = ol.createEl("li", { cls });
      const a = li.createEl("a", { text: h.doc_path, href: "#" });
      a.addEventListener("click", (ev) => {
        ev.preventDefault();
        openDoc(this.app, h.doc_path);
      });
      li.createSpan({ text: ` · ${h.layer} · ${h.score.toFixed(4)} · ${String(h.lineage_ids.length)} blocks` });
    }
  }
}
