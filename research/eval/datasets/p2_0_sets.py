"""P2-0 data sets (GOAL P2-0 "Veri setleri"; TASKS Z5), generated from one
seed, deterministic, no wall-clock and no randomness outside `random.Random`
seeded here.

- **D-A** (agent writes straight to the files): 200 scenarios over the
  mini vault (`spec/fixtures/mini_vault`), seven kinds — a new file, an
  append to a human document, a section replaced, a whole document
  overwritten, a rename, five quick writes, a new file anywhere outside
  `.rootmark/` (nested directories). Every step is a file-system operation
  (`eval.agent_fs_writer`). The generator keeps the files' text, so each
  step is computed on the vault as the previous scenarios left it.
- **D-B** (real human editing in Obsidian): 300 sessions for
  `clients/obsidian/e2e/p2_0/sessions.ts` — one character, a paragraph,
  several notes, quick saves on Obsidian's autosave, a large paste, a
  rename, a save on focus loss. The first 100 are the tuning set (the
  window `W` is chosen on them), the other 200 the test set; the kinds are
  spread over both by the same shuffle.
- **D-C** (both at once): 160 ticks in one vault; in a tick the agent writes
  one note and the human saves another (the plugin's protocol: file, then
  attestation), within a few milliseconds; some ticks touch the same note
  (the agent after the human, after a pause) and a few collide (the agent
  appends to the note the human just saved, before the watcher looks).
  Every paragraph carries its author, so the right layer of every block is
  known: the human's `canonical`, the agent's `unverified`.
- **D-E** (vaults written by `v0.1.0`) is built by
  `scripts/gen_p2_0_old_vaults.sh` with the `v0.1.0` binary and committed
  (`research/data/p2_0_old_vaults/`); this module only lists them.

    python -m eval.datasets.p2_0_sets --out <dir>   (writes d_a.json, d_b_tune.json,
                                                     d_b_test.json, d_c.json)
"""

from __future__ import annotations

import argparse
import json
import random
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MINI = ROOT / "spec" / "fixtures" / "mini_vault"
OLD_VAULTS = ROOT / "research" / "data" / "p2_0_old_vaults"
SEED = 20261018

D_A_KINDS = [
    "new_file",
    "append_human",
    "replace_section",
    "overwrite",
    "rename",
    "rapid5",
    "anywhere",
]
D_B_KINDS = ["char", "paragraph", "multi", "rapid", "paste", "rename", "focus"]
D_E_VAULTS = ["scene", "e2e", "mixed"]

WORDS = [
    "roast",
    "crema",
    "grinder",
    "kettle",
    "bloom",
    "extraction",
    "drum",
    "airflow",
    "cupping",
    "origin",
    "washed",
    "natural",
    "honey",
    "acidity",
    "body",
    "sweetness",
    "finish",
    "ratio",
    "dose",
    "yield",
    "temperature",
    "pressure",
    "burr",
    "calibration",
    "descale",
    "gasket",
    "portafilter",
    "tamp",
    "channel",
    "basket",
    "filter",
    "paper",
    "pour",
    "over",
    "chemex",
    "aeropress",
    "siphon",
    "espresso",
    "lungo",
    "ristretto",
    "milk",
    "foam",
    "latte",
    "flat",
    "white",
    "cortado",
    "barista",
    "shift",
    "log",
    "batch",
    "charge",
    "turning",
    "point",
    "first",
    "crack",
    "development",
    "cooling",
    "tray",
    "bean",
    "green",
    "sack",
    "storage",
    "humidity",
    "altitude",
    "varietal",
    "geisha",
    "bourbon",
    "typica",
]


def mini_docs() -> dict[str, str]:
    """The mini vault's documents (`doc_path` → text)."""
    return {p.name: p.read_text(encoding="utf-8") for p in sorted(MINI.glob("*.md"))}


def sentence(r: random.Random, tag: str) -> str:
    """One sentence; `tag` makes it unique and names its author."""
    n = r.randint(7, 15)
    w = [r.choice(WORDS) for _ in range(n)]
    return f"{tag} {' '.join(w)}."


def paragraph(r: random.Random, tag: str, sentences: tuple[int, int] = (1, 3)) -> str:
    return " ".join(sentence(r, f"{tag}.{i}") for i in range(r.randint(*sentences)))


def headings(text: str) -> list[str]:
    """ATX heading titles of `text` (the `replace_section` targets)."""
    return [m.group(2).strip() for m in re.finditer(r"^(#{1,6})\s+(.*)$", text, re.MULTILINE)]


def replace_section(text: str, heading: str, body: str) -> str:
    """`text` with the body under the one ATX heading `heading` replaced up to
    the next heading of the same or a higher level."""
    lines = text.split("\n")
    at = None
    level = 0
    for i, line in enumerate(lines):
        m = re.match(r"^(#{1,6})\s+(.*)$", line)
        if m and m.group(2).strip() == heading:
            at, level = i, len(m.group(1))
            break
    if at is None:
        raise ValueError(f"no heading {heading!r}")
    end = len(lines)
    for j in range(at + 1, len(lines)):
        m = re.match(r"^(#{1,6})\s", lines[j])
        if m and len(m.group(1)) <= level:
            end = j
            break
    new = lines[: at + 1] + ["", body, ""] + lines[end:]
    return "\n".join(new).rstrip("\n") + "\n"


def d_a(seed: int = SEED) -> dict:
    """200 agent-write scenarios (D-A); see the module comment."""
    r = random.Random(f"{seed}-d-a")
    files = mini_docs()
    human = set(files)
    kinds = [D_A_KINDS[i % len(D_A_KINDS)] for i in range(200)]
    r.shuffle(kinds)
    out = []
    n_new = 0
    for i, kind in enumerate(kinds):
        sid = f"a{i:03d}"
        tag = f"Agent {sid}"
        steps: list[dict] = []
        if kind in ("new_file", "anywhere"):
            n_new += 1
            path = (
                f"agent-notes/n{n_new:03d}.md"
                if kind == "new_file"
                else f"{r.choice(['deep/a', 'x/y/z', 'drafts', 'misc/archive'])}/f{n_new:03d}.md"
            )
            text = f"# {tag} note\n\n{paragraph(r, tag)}\n"
            steps.append({"op": "write", "path": path, "text": text})
            files[path] = text
        elif kind == "append_human":
            path = r.choice(sorted(human & set(files)))
            text = files[path].rstrip("\n") + f"\n\n{paragraph(r, tag)}\n"
            steps.append({"op": "write", "path": path, "text": text})
            files[path] = text
        elif kind == "replace_section":
            cands = [p for p in sorted(files) if len(headings(files[p])) > 1]
            path = r.choice(cands)
            heading = r.choice(headings(files[path])[1:])
            text = replace_section(files[path], heading, paragraph(r, tag, (1, 2)))
            steps.append({"op": "write", "path": path, "text": text})
            files[path] = text
        elif kind == "overwrite":
            path = r.choice(sorted(files))
            text = f"# {tag} rewrite\n\n{paragraph(r, tag)}\n\n{paragraph(r, tag + 'b')}\n"
            steps.append({"op": "write", "path": path, "text": text})
            files[path] = text
        elif kind == "rename":
            path = r.choice(sorted(files))
            to = f"renamed/{sid}-{Path(path).name}"
            steps.append({"op": "rename", "path": path, "to": to})
            files[to] = files.pop(path)
            human.discard(path)
        elif kind == "rapid5":
            path = r.choice(sorted(files))
            text = files[path]
            for k in range(5):
                text = text.rstrip("\n") + f"\n\n{sentence(r, f'{tag}.{k}')}\n"
                steps.append({"op": "write", "path": path, "text": text})
            files[path] = text
        out.append({"id": sid, "kind": kind, "steps": steps})
    return {"seed": seed, "set": "D-A", "scenarios": out}


def d_b(seed: int = SEED) -> tuple[dict, dict]:
    """300 human editing sessions (D-B): tuning (100), test (200)."""
    r = random.Random(f"{seed}-d-b")
    names = sorted(mini_docs())
    kinds = [D_B_KINDS[i % len(D_B_KINDS)] for i in range(300)]
    r.shuffle(kinds)
    sessions = []
    n_renamed = 0
    for i, kind in enumerate(kinds):
        if i == 100:
            # the test set runs in a fresh vault: its names start over
            names = sorted(mini_docs())
        sid = f"b{i:03d}"
        tag = f"Human {sid}"
        doc = r.choice(names)
        steps: list[dict] = []
        if kind == "char":
            steps += [
                {"op": "type", "doc": doc, "text": r.choice("abcdefghijklmnopqrstuvwxyz")},
                {"op": "save", "how": r.choice(["command", "autosave"])},
            ]
        elif kind == "paragraph":
            steps += [
                {"op": "type", "doc": doc, "text": f"\n\n{paragraph(r, tag)}"},
                {"op": "save", "how": "command"},
            ]
        elif kind == "multi":
            for k, d in enumerate(r.sample(names, r.randint(2, 3))):
                steps += [
                    {"op": "type", "doc": d, "text": f"\n\n{paragraph(r, f'{tag}.{k}')}"},
                    {"op": "save", "how": "command"},
                ]
        elif kind == "rapid":
            for k in range(r.randint(3, 4)):
                steps += [
                    {"op": "type", "doc": doc, "text": f"\n\n{sentence(r, f'{tag}.{k}')}"},
                    {"op": "save", "how": "autosave"},
                ]
        elif kind == "paste":
            big = "\n\n".join(paragraph(r, f"{tag}.{k}", (2, 4)) for k in range(r.randint(8, 16)))
            steps += [
                {"op": "type", "doc": doc, "text": f"\n\n## {tag} pasted notes\n\n{big}"},
                {"op": "save", "how": "command"},
            ]
        elif kind == "rename":
            n_renamed += 1
            to = f"r{n_renamed:03d}-{doc}"
            steps.append({"op": "rename", "doc": doc, "to": to})
            names = sorted({*names} - {doc} | {to})
        elif kind == "focus":
            steps += [
                {"op": "type", "doc": doc, "text": f"\n\n{paragraph(r, tag)}"},
                {"op": "save", "how": "blur"},
            ]
        sessions.append({"id": sid, "kind": kind, "steps": steps})
    return (
        {"seed": seed, "set": "D-B tune", "sessions": sessions[:100]},
        {"seed": seed, "set": "D-B test", "sessions": sessions[100:]},
    )


def d_c(seed: int = SEED) -> dict:
    """160 ticks of an agent and a human at once (D-C); every paragraph is
    tagged with its author (`Agent c…` / `Human c…`)."""
    r = random.Random(f"{seed}-d-c")
    files = mini_docs()
    ticks = []
    for i in range(160):
        tid = f"c{i:03d}"
        mode = r.choices(["apart", "same_doc_after", "collide"], weights=[80, 15, 5])[0]
        names = sorted(files)
        hdoc = r.choice(names)
        if mode == "apart":
            adoc = r.choice([n for n in names if n != hdoc] + [f"agent-c/{tid}.md"])
        else:
            adoc = hdoc
        htext = files[hdoc].rstrip("\n") + f"\n\n{paragraph(r, f'Human {tid}')}\n"
        files[hdoc] = htext
        base = files.get(adoc, f"# Agent {tid} note\n")
        atext = base.rstrip("\n") + f"\n\n{paragraph(r, f'Agent {tid}')}\n"
        files[adoc] = atext
        ticks.append(
            {
                "id": tid,
                "mode": mode,
                "human": {"path": hdoc, "text": htext},
                "agent": {"path": adoc, "text": atext},
                # the agent after the human: right away (collide) or after
                # the watcher has seen the human's save (same_doc_after)
                "agent_delay_ms": {"apart": 2, "same_doc_after": 600, "collide": 2}[mode],
            }
        )
    return {"seed": seed, "set": "D-C", "ticks": ticks}


def author_of(text: str) -> str | None:
    """The author a block's text names (`Agent …`, `Human …`), or None (the
    mini vault's own content: human)."""
    m = re.search(r"\b(Agent|Human) [a-c]\d{3}", text)
    return None if m is None else m.group(1).lower()


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args(argv)
    a.out.mkdir(parents=True, exist_ok=True)
    tune, test = d_b()
    for name, obj in [("d_a", d_a()), ("d_b_tune", tune), ("d_b_test", test), ("d_c", d_c())]:
        (a.out / f"{name}.json").write_text(json.dumps(obj, indent=1, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
