"""P2-0.1 data sets (GOAL P2-0.1 "Veri setleri"; TASKS Y3), written to the
definitions pre-registered in `docs/rounds/P2-0.1/progress/ASSUMPTIONS.md`
Y0-4..Y0-10 before any data. Deterministic: no wall-clock, no randomness
outside the `random.Random` instances seeded here (same seed → same sets).

- **D-B test-2** (Y0-4): the P2-0 generator `p2_0_sets.d_b` with the new
  seed `int(blake3("P2-0.1/D-B").hexdigest()[:8], 16)`; its test part (200
  sessions, a fresh vault). The tuning part is never run.
- **D-A′** (Y0-5, Y0-10 b, c): 50 laundering scenarios for the real
  Obsidian (the agent writes a note, the human renames a note it links to)
  and 10 fake link updates (token, `link_update_of`, a change beyond the
  link targets).
- **D-B2** (Y0-6, Y0-10 d): 150 editor-emulation sessions with
  `rootmark attest`.
- **D-A2** (Y0-7, Y0-10 a): 120 agent writes over six file-system paths.
- **D-C′** (Y0-8): 40 real-Obsidian collision ticks.

D-A and D-C are P2-0's (`p2_0_sets.d_a`, `p2_0_sets.d_c`), unchanged.

    python -m eval.datasets.p2_0_1_sets --out <dir>
"""

from __future__ import annotations

import argparse
import json
import random
import re
from pathlib import Path

import blake3

from eval.datasets import p2_0_sets as p0

MINI = p0.MINI


def d_b_seed() -> int:
    """Y0-4: the first 8 hex characters of BLAKE3("P2-0.1/D-B") as an integer."""
    return int(blake3.blake3(b"P2-0.1/D-B").hexdigest()[:8], 16)


def d_b_test2() -> dict:
    """D-B test-2: P2-0's generator with the new seed, its test part only."""
    seed = d_b_seed()
    _tune, test = p0.d_b(seed)
    return {"seed": seed, "set": "D-B test-2", "sessions": test["sessions"]}


LINK = re.compile(r"\[\[([^\[\]|#]*)")


def _stem(path: str) -> str:
    return Path(path).name.removesuffix(".md")


def human_links(texts: dict[str, str]) -> dict[str, set[str]]:
    """Note → the note stems its (human) text links to."""
    stems = {_stem(p) for p in texts}
    return {
        p: {m.group(1).strip() for m in LINK.finditer(t)} & stems - {_stem(p)}
        for p, t in texts.items()
    }


# fake link-update variants (Y0-5): each changes the bytes beyond link targets
FAKE_VARIANTS = [
    "append_paragraph",
    "word_in_canonical",
    "link_label",
    "heading_text",
    "delete_paragraph",
    "new_link",
    "list_item",
    "code_block",
    "target_and_text",
    "replace_with_agent",
]


def d_a_prime() -> dict:
    """D-A′ (Y0-5, Y0-10 b, c)."""
    r = random.Random("P2-0.1-d-a-prime")
    texts = p0.mini_docs()
    links = human_links(texts)
    names = sorted(texts)
    kinds = ["in_agent_block"] * 25 + ["in_human_block"] * 15 + ["unobserved"] * 10
    r.shuffle(kinds)
    out = []
    for i, kind in enumerate(kinds):
        sid = f"p{i:03d}"
        tag = f"Agent {sid}"
        n = r.choice(names)
        t = r.choice([x for x in names if x != n])
        steps: list[dict] = []
        if kind == "in_human_block":
            if _stem(t) not in links[n]:
                # Y0-10 c: the human first writes a linking paragraph in Obsidian
                steps.append(
                    {"op": "human_type", "doc": n, "text": f"\n\nHuman {sid} see [[{_stem(t)}]]."}
                )
                links[n].add(_stem(t))
            para = p0.paragraph(r, tag, (1, 2))
        else:
            para = f"{p0.paragraph(r, tag, (1, 2))} See [[{_stem(t)}]]."
        delay = r.choice([0, 10, 25, 50]) if kind == "unobserved" else None
        steps.append({"op": "agent_append", "doc": n, "text": f"\n\n{para}\n"})
        to = f"{sid}-{Path(t).name}"
        steps.append({"op": "rename", "doc": t, "to": to, "delay_ms": delay})
        # Obsidian rewrites every [[t]] to the new name
        names = sorted({*names} - {t} | {to})
        links = {(to if k == t else k): v for k, v in links.items()}
        for v in links.values():
            if _stem(t) in v:
                v.discard(_stem(t))
                v.add(_stem(to))
        out.append({"id": sid, "kind": kind, "agent_doc": n, "target": t, "steps": steps})
    fakes = [
        {"id": f"f{j:03d}", "kind": "fake_link_update", "variant": v}
        for j, v in enumerate(FAKE_VARIANTS)
    ]
    return {"set": "D-A′", "scenarios": out, "fakes": fakes}


D_B2_KINDS = ["atomic"] * 50 + ["inplace"] * 30 + ["append"] * 20 + ["mv"] * 30 + ["rapid"] * 20


def d_b2() -> dict:
    """D-B2 (Y0-6, Y0-10 d): 150 editor sessions; each save is followed by
    `rootmark attest`."""
    r = random.Random("P2-0.1-d-b2")
    kinds = list(D_B2_KINDS)
    r.shuffle(kinds)
    names = sorted(p0.mini_docs())
    sessions = []
    n_mv = 0
    for i, kind in enumerate(kinds):
        sid = f"d{i:03d}"
        tag = f"Human {sid}"
        doc = r.choice(names)
        if kind in ("atomic", "inplace", "append"):
            saves = [{"how": kind, "text": f"\n\n{p0.paragraph(r, tag)}\n"}]
            sessions.append({"id": sid, "kind": kind, "doc": doc, "saves": saves})
        elif kind == "rapid":
            saves = [
                {
                    "how": "inplace",
                    "text": f"\n\n{p0.sentence(r, f'{tag}.{k}')}\n",
                    "gap_s": round(r.uniform(0.1, 0.3), 3),
                }
                for k in range(r.randint(3, 4))
            ]
            sessions.append({"id": sid, "kind": kind, "doc": doc, "saves": saves})
        else:
            n_mv += 1
            to = f"m{n_mv:03d}-{doc}"
            sessions.append({"id": sid, "kind": kind, "doc": doc, "to": to, "saves": []})
            names = sorted({*names} - {doc} | {to})
    return {"set": "D-B2", "sessions": sessions}


D_A2_PATHS = ["python", "sed", "git_apply", "cp_rsync", "cat", "atomic_rename"]
D_A2_KINDS = ["append_human", "replace_section", "overwrite", "new_file"]


def d_a2() -> dict:
    """D-A2 (Y0-7, Y0-10 a): 120 agent writes, 20 per path, each path's
    scenarios split over the four content kinds by their order (mod 4).
    Steps carry the document's full new text (the runner realises it with
    the path's tool); the generator keeps every file's text."""
    r = random.Random("P2-0.1-d-a2")
    plan = [(p, D_A2_KINDS[j % 4], j) for p in D_A2_PATHS for j in range(20)]
    r.shuffle(plan)
    files = p0.mini_docs()
    human = set(files)
    out = []
    for i, (path_kind, kind, j) in enumerate(plan):
        sid = f"e{i:03d}"
        tag = f"Agent {sid}"
        if kind == "new_file":
            doc = f"agent-e/{sid}.md"
            old = None
            text = f"# {tag} note\n\n{p0.paragraph(r, tag)}\n"
        elif kind == "append_human":
            doc = r.choice(sorted(human & set(files)))
            old = files[doc]
            text = old.rstrip("\n") + f"\n\n{p0.paragraph(r, tag)}\n"
        elif kind == "replace_section":
            doc = r.choice([p for p in sorted(files) if len(p0.headings(files[p])) > 1])
            old = files[doc]
            text = p0.replace_section(old, r.choice(p0.headings(old)[1:]), p0.paragraph(r, tag))
        else:
            doc = r.choice(sorted(files))
            old = files[doc]
            text = f"# {tag} rewrite\n\n{p0.paragraph(r, tag)}\n\n{p0.paragraph(r, tag + 'b')}\n"
        files[doc] = text
        step = {"op": path_kind, "path": doc, "text": text, "kind": kind}
        if path_kind == "cp_rsync":
            step["tool"] = "cp" if j % 2 == 0 else "rsync"
        out.append({"id": sid, "kind": f"{path_kind}/{kind}", "steps": [step]})
    return {"set": "D-A2", "scenarios": out}


def d_c_prime() -> dict:
    """D-C′ (Y0-8): 40 ticks; the agent appends 0/20/50/100 ms (10 each)
    after Obsidian finished writing the human's save of the same note."""
    r = random.Random("P2-0.1-d-c-prime")
    delays = [0, 20, 50, 100] * 10
    r.shuffle(delays)
    names = sorted(p0.mini_docs())
    ticks = []
    for i, d in enumerate(delays):
        tid = f"q{i:03d}"
        ticks.append(
            {
                "id": tid,
                "doc": names[i % len(names)],
                "human": f"\n\n{p0.paragraph(r, f'Human {tid}')}",
                "agent": f"\n\n{p0.paragraph(r, f'Agent {tid}')}\n",
                "agent_delay_ms": d,
            }
        )
    return {"set": "D-C′", "ticks": ticks}


def author_of(text: str) -> str | None:
    """The author a block's text names (`Agent p/e/q…`, `Human d/p/q…`)."""
    m = re.search(r"\b(Agent|Human) [a-z]\d{3}", text)
    return None if m is None else m.group(1).lower()


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args(argv)
    a.out.mkdir(parents=True, exist_ok=True)
    for name, obj in [
        ("d_b_test2", d_b_test2()),
        ("d_a_prime", d_a_prime()),
        ("d_b2", d_b2()),
        ("d_a2", d_a2()),
        ("d_c_prime", d_c_prime()),
    ]:
        (a.out / f"{name}.json").write_text(json.dumps(obj, indent=1, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
