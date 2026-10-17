// The blame gutter (CodeMirror 6): one mark per block of the active note,
// set by the plugin after each `blame`. Clicking a mark asks for its history.
import { RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
import { gutter, GutterMarker, type EditorView } from "@codemirror/view";
import type { GutterMark } from "../view";

class Mark extends GutterMarker {
  constructor(readonly m: GutterMark) {
    super();
  }
  override eq(o: GutterMarker): boolean {
    return o instanceof Mark && o.m.lineage_id === this.m.lineage_id && o.m.cls === this.m.cls && o.m.title === this.m.title;
  }
  override toDOM(): Node {
    const el = document.createElement("span");
    el.className = `rootmark-mark ${this.m.cls}`;
    el.textContent = this.m.label;
    el.title = this.m.title;
    return el;
  }
}

/** Replaces the marks of an editor. */
export const setMarks = StateEffect.define<GutterMark[]>();

const marksField = StateField.define<GutterMark[]>({
  create: () => [],
  update: (v, tr) => {
    for (const e of tr.effects) if (e.is(setMarks)) return e.value;
    return tr.docChanged ? [] : v; // an edited note is stale until the next blame
  },
});

export function blameGutter(onClick: (lineage: string) => void): Extension {
  return [
    marksField,
    gutter({
      class: "rootmark-gutter",
      markers: (view: EditorView) => {
        const marks = view.state.field(marksField);
        const doc = view.state.doc;
        const ranges = marks
          .filter((m) => m.line < doc.lines)
          .sort((a, b) => a.line - b.line)
          .map((m) => new Mark(m).range(doc.line(m.line + 1).from));
        return RangeSet.of(ranges, true);
      },
      domEventHandlers: {
        click: (view, line) => {
          const n = view.state.doc.lineAt(line.from).number - 1;
          const m = view.state.field(marksField).find((x) => x.line === n);
          if (m) onClick(m.lineage_id);
          return m !== undefined;
        },
      },
    }),
  ];
}
