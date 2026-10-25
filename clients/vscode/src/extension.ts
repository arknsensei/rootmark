// The VS Code extension skeleton (P2-B/B4 (b)): the glue between VS Code and
// the command layer (`core.ts`). It reads the connection settings from the
// workspace's vault (`.rootmark/policy.toml` `[server] http_port`,
// `.rootmark/server.token`) at each connect and keeps the token in memory
// only: never in settings, `globalState` or `SecretStorage` (README,
// docs/THREAT_MODEL.md). It never writes a vault file and never attests a
// save: a VS Code save is an unattested external change.

import * as vscode from "vscode";
import { connect, type RootmarkClient } from "../../obsidian/src/api";
import { fsReader, fetchTransport } from "./node-io";
import { blameView, blockAt, CONSUMER, demote, describeError, historyLines, promote, search, statusLine, type BlameView } from "./core";

const LAYERS = ["canonical", "proposal", "unverified", "derived"] as const;

export function activate(ctx: vscode.ExtensionContext): void {
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (folder === undefined) return;
  const root = folder.uri.fsPath;
  const bar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left);
  bar.command = "rootmark.status";
  ctx.subscriptions.push(bar);
  // one decoration per layer; proposal and unverified stand out
  const deco = Object.fromEntries(
    LAYERS.map((l) => [
      l,
      vscode.window.createTextEditorDecorationType({
        isWholeLine: true,
        overviewRulerLane: vscode.OverviewRulerLane.Left,
        ...(l === "proposal"
          ? { backgroundColor: new vscode.ThemeColor("diffEditor.insertedLineBackground"), overviewRulerColor: "#3fb950" }
          : l === "unverified"
            ? { backgroundColor: new vscode.ThemeColor("inputValidation.warningBackground"), overviewRulerColor: "#d29922" }
            : l === "derived"
              ? { opacity: "0.6" }
              : {}),
      }),
    ]),
  ) as Record<(typeof LAYERS)[number], vscode.TextEditorDecorationType>;
  for (const d of Object.values(deco)) ctx.subscriptions.push(d);
  let views = new Map<string, BlameView>();
  let tip = "rootmark";

  async function client(): Promise<RootmarkClient | null> {
    const st = await connect(fsReader(root), fetchTransport, CONSUMER);
    if (st.kind === "offline") {
      bar.text = "rootmark: offline";
      tip = st.reason;
      bar.tooltip = tip;
      bar.show();
      return null;
    }
    const s = statusLine(st.status);
    bar.text = s.text;
    tip = s.tooltip;
    bar.tooltip = tip;
    bar.show();
    return st.client;
  }

  function rel(doc: vscode.TextDocument): string | null {
    const r = vscode.workspace.asRelativePath(doc.uri, false);
    return doc.uri.scheme === "file" && r.endsWith(".md") && !r.startsWith("..") ? r.split("\\").join("/") : null;
  }

  async function refresh(editor: vscode.TextEditor | undefined): Promise<void> {
    if (editor === undefined) return;
    const doc = rel(editor.document);
    for (const d of Object.values(deco)) editor.setDecorations(d, []);
    if (doc === null) return;
    const c = await client();
    if (c === null) return;
    try {
      const v = await blameView(c, doc, editor.document.getText());
      views.set(doc, v);
      if (v.kind !== "located") {
        bar.tooltip = `${tip}\n${v.reason}`;
        return;
      }
      for (const l of LAYERS) {
        editor.setDecorations(
          deco[l],
          v.blocks.filter((b) => b.layer === l).map((b) => ({ range: new vscode.Range(b.start, 0, b.end, 0), hoverMessage: b.hover })),
        );
      }
    } catch (e) {
      void vscode.window.showErrorMessage(`rootmark blame: ${describeError(e)}`);
    }
  }

  function atCursor(): { lineage: string } | null {
    const ed = vscode.window.activeTextEditor;
    const doc = ed === undefined ? null : rel(ed.document);
    if (ed === undefined || doc === null) return null;
    const v = views.get(doc);
    const b = v === undefined ? null : blockAt(v, ed.selection.active.line);
    return b === null ? null : { lineage: b.lineage_id };
  }

  const cmd = (name: string, f: () => Promise<void>): void => {
    ctx.subscriptions.push(
      vscode.commands.registerCommand(name, async () => {
        try {
          await f();
        } catch (e) {
          void vscode.window.showErrorMessage(`rootmark: ${describeError(e)}`);
        }
      }),
    );
  };

  cmd("rootmark.status", async () => {
    const c = await client();
    if (c !== null) void vscode.window.showInformationMessage(tip);
  });
  cmd("rootmark.search", async () => {
    const c = await client();
    if (c === null) return;
    const q = await vscode.window.showInputBox({ prompt: "rootmark search (filters: prefix:<p> from:YYYY-MM-DD to:YYYY-MM-DD proposals)" });
    if (q === undefined || q.trim() === "") return;
    const words = q.split(/\s+/);
    const opt: Parameters<typeof search>[2] = {};
    const text: string[] = [];
    for (const w of words) {
      if (w.startsWith("prefix:")) (opt.path_prefix ??= []).push(w.slice(7));
      else if (w.startsWith("from:")) opt.date_from = w.slice(5);
      else if (w.startsWith("to:")) opt.date_to = w.slice(3);
      else if (w === "proposals") opt.include_proposal = true;
      else text.push(w);
    }
    const r = await search(c, text.join(" "), opt);
    const pick = await vscode.window.showQuickPick(
      r.items.map((i) => ({ label: i.label, item: i })),
      { placeHolder: r.undated_excluded === null ? "results" : `results (${String(r.undated_excluded)} undated notes left out)` },
    );
    if (pick === undefined) return;
    const ed = await vscode.window.showTextDocument(vscode.Uri.joinPath(folder.uri, pick.item.doc_path));
    if (pick.item.line !== null) {
      const p = new vscode.Position(pick.item.line, 0);
      ed.selection = new vscode.Selection(p, p);
      ed.revealRange(new vscode.Range(p, p));
    }
  });
  cmd("rootmark.blame", () => refresh(vscode.window.activeTextEditor));
  cmd("rootmark.clearBlame", async () => {
    const ed = vscode.window.activeTextEditor;
    if (ed !== undefined) for (const d of Object.values(deco)) ed.setDecorations(d, []);
    views = new Map();
    await Promise.resolve();
  });
  cmd("rootmark.history", async () => {
    const b = atCursor();
    const c = b === null ? null : await client();
    if (b === null || c === null) return void vscode.window.showWarningMessage("rootmark: no located block at the cursor (run rootmark: Blame)");
    void vscode.window.showInformationMessage((await historyLines(c, b.lineage)).join("\n"), { modal: true });
  });
  cmd("rootmark.promote", async () => {
    const b = atCursor();
    const c = b === null ? null : await client();
    if (b === null || c === null) return void vscode.window.showWarningMessage("rootmark: no located block at the cursor (run rootmark: Blame)");
    const reason = await vscode.window.showInputBox({ prompt: "reason for the promotion" });
    if (reason === undefined || reason.trim() === "") return;
    void vscode.window.showInformationMessage(await promote(c, b.lineage, reason));
    await refresh(vscode.window.activeTextEditor);
  });
  cmd("rootmark.demote", async () => {
    const b = atCursor();
    const c = b === null ? null : await client();
    if (b === null || c === null) return void vscode.window.showWarningMessage("rootmark: no located block at the cursor (run rootmark: Blame)");
    const reason = await vscode.window.showInputBox({ prompt: "reason for the demotion" });
    if (reason === undefined || reason.trim() === "") return;
    void vscode.window.showInformationMessage(await demote(c, b.lineage, reason));
    await refresh(vscode.window.activeTextEditor);
  });

  ctx.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((e) => void refresh(e)),
    // after a save the server ingests the file (its watcher); the blame
    // is asked again shortly after, and says "changed" until then
    vscode.workspace.onDidSaveTextDocument((d) => {
      const ed = vscode.window.activeTextEditor;
      if (ed?.document === d) setTimeout(() => void refresh(ed), 750);
    }),
    vscode.workspace.onDidChangeTextDocument((e) => {
      const ed = vscode.window.activeTextEditor;
      if (ed?.document === e.document) for (const d of Object.values(deco)) ed.setDecorations(d, []);
    }),
  );
  void refresh(vscode.window.activeTextEditor);
}

export function deactivate(): void {}
