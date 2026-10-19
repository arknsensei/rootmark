"""P2-0.2 data sets (GOAL P2-0.2 "Veri setleri"; TASKS X4), written to the
definitions pre-registered in `docs/rounds/P2-0.2/progress/ASSUMPTIONS.md`
X0-4..X0-8 and X0-15..X0-18 before any data. Deterministic: no wall-clock,
no randomness outside the `random.Random` instances seeded here.

- **test-3 vault** (X0-5, X0-17): the mini vault's 12 notes, 3 of them in
  the folder `dizin adı/`, every wikilink rewritten in one of six link
  forms, an `İlgili` paragraph of two more links per note, and six notes
  linking to themselves (three forms).
- **D-B test-3** (X0-4, X0-6): 200 real-Obsidian sessions, renames 35 %
  (`rename`, `rename_again`, `move_folder`).
- **D-A′** (X0-8): 50 laundering scenarios + 10 fake link updates (P2-0.1's
  definition on the test-3 vault, links in the six forms) + 10 rename
  carry scenarios.
- **coverage** (X0-16): the counts that must not be 0 before any session.

D-A, D-A2, D-B2, D-C, D-C′ are P2-0's and P2-0.1's generators, unchanged.

    python -m eval.datasets.p2_0_2_sets --out <dir>
"""

from __future__ import annotations

import argparse
import json
import random
import re
from pathlib import PurePosixPath

import blake3

from eval.datasets import p2_0_1_sets as p1
from eval.datasets import p2_0_sets as p0

FOLDER = "dizin adı"
ARCHIVE = "arşiv"
FORMS = ["wiki", "wiki_alias", "wiki_heading", "embed", "md_name", "md_label"]
SELF_FORMS = ["self_wiki", "self_wiki_heading", "self_md"]
WIKI = re.compile(r"\[\[([^\[\]|#]*)(#[^\[\]|]*)?(\|[^\[\]]*)?\]\]")
FAKE_VARIANTS = p1.FAKE_VARIANTS


def d_b_seed() -> int:
    """X0-4: the first 8 hex characters of BLAKE3("P2-0.2/D-B") as an integer."""
    return int(blake3.blake3(b"P2-0.2/D-B").hexdigest()[:8], 16)


def stem(path: str) -> str:
    return PurePosixPath(path).name.removesuffix(".md")


def folder(path: str) -> str:
    p = str(PurePosixPath(path).parent)
    return "" if p == "." else p


def encoded(path: str) -> str:
    """A vault path as a Markdown link destination: spaces as `%20` (X0-5)."""
    return path.replace(" ", "%20")


def first_heading(text: str) -> str:
    return p0.headings(text)[0]


class Vault3:
    """The test-3 vault being built: texts by path, and the links it holds
    (from path, form, target path) — the generators' bookkeeping, followed
    through renames as Obsidian rewrites them (coverage, X0-16)."""

    def __init__(self, texts: dict[str, str], links: list[dict]):
        self.texts = texts
        self.links = links

    def rename(self, frm: str, to: str) -> None:
        self.texts[to] = self.texts.pop(frm)
        for lk in self.links:
            if lk["from"] == frm:
                lk["from"] = to
            if lk["target"] == frm:
                lk["target"] = to


def render(
    form: str, target: str, texts: dict[str, str], heading: str | None, alias: str | None
) -> str:
    """One link to `target` in `form` (X0-5, X0-17)."""
    t = stem(target)
    if form == "wiki":
        return f"[[{t}]]"
    if form == "wiki_alias":
        return f"[[{t}|{alias or f'{t} notu'}]]"
    if form == "wiki_heading":
        return f"[[{t}#{heading or first_heading(texts[target])}]]"
    if form == "embed":
        return f"![[{t}]]"
    if form == "md_name":
        return f"[{t}]({encoded(target)})"
    if form == "md_label":
        return f"[{alias or f'{t} hakkında'}]({encoded(target)})"
    raise ValueError(form)


def vault3() -> dict:
    """The test-3 vault (X0-5): `texts` (path → text), `links`, `counts`."""
    r = random.Random(f"{d_b_seed()}-vault3")
    mini = p0.mini_docs()
    names = sorted(mini)
    moved = sorted(r.sample(names, 3))
    path = {n: (f"{FOLDER}/{n}" if n in moved else n) for n in names}
    by_stem = {stem(n): n for n in names}
    pool = FORMS * 8
    r.shuffle(pool)
    texts = {path[n]: mini[n] for n in names}
    links: list[dict] = []
    k = 0
    # (3) the mini vault's wikilinks, notes sorted, text order
    for n in names:
        p = path[n]

        def sub(m: re.Match, p: str = p) -> str:
            nonlocal k
            target = path[by_stem[m.group(1).strip()]]
            heading = m.group(2)[1:] if m.group(2) else None
            alias = m.group(3)[1:] if m.group(3) else None
            form = pool[k]
            k += 1
            links.append({"from": p, "form": form, "target": target})
            return render(form, target, texts, heading, alias)

        texts[p] = WIKI.sub(sub, texts[p])
    # then each note's `İlgili` paragraph: two more links, in order
    for n in names:
        p = path[n]
        xs = r.sample([x for x in names if x != n], 2)
        parts = []
        for x in xs:
            form = pool[k]
            k += 1
            links.append({"from": p, "form": form, "target": path[x]})
            parts.append(render(form, path[x], texts, None, None))
        texts[p] = texts[p].rstrip("\n") + f"\n\nİlgili: {parts[0]} ve {parts[1]}.\n"
    # (4) six notes linking to themselves
    selfs = r.sample(names, 6)
    for i, n in enumerate(selfs):
        p = path[n]
        t = stem(p)
        form = SELF_FORMS[i // 2]
        link = {
            "self_wiki": f"[[{t}]]",
            "self_wiki_heading": f"[[{t}#{t}]]",
            "self_md": f"[{t}]({encoded(p)})",
        }[form]
        links.append({"from": p, "form": form, "target": p})
        texts[p] = texts[p].rstrip("\n") + f"\n\nBu not: {link}.\n"
    return {
        "seed": d_b_seed(),
        "set": "test-3 vault",
        "texts": texts,
        "links": links,
        "counts": link_counts(links),
    }


def link_counts(links: list[dict]) -> dict[str, int]:
    """Places per form (X0-16); `md_path` = a Markdown link to a note in a
    folder (also counted in its `md_*` form)."""
    out = {f: 0 for f in FORMS + SELF_FORMS + ["md_path"]}
    for lk in links:
        out[lk["form"]] += 1
        if lk["form"].startswith("md_") and folder(lk["target"]):
            out["md_path"] += 1
    return out


D_B3_KINDS = (
    ["char"] * 22
    + ["paragraph"] * 22
    + ["multi"] * 22
    + ["rapid"] * 22
    + ["paste"] * 21
    + ["focus"] * 21
    + ["rename"] * 40
    + ["rename_again"] * 15
    + ["move_folder"] * 15
)
RENAMES = {"rename", "rename_again", "move_folder"}


def d_b3() -> dict:
    """D-B test-3 (X0-6): 200 sessions on the test-3 vault, with each
    rename's coverage facts (X0-16) from the generator's own bookkeeping."""
    v3 = vault3()
    vault = Vault3(dict(v3["texts"]), [dict(x) for x in v3["links"]])
    r = random.Random(f"{d_b_seed()}-d-b3")
    kinds = list(D_B3_KINDS)
    r.shuffle(kinds)
    # a rename_again with no renamed note before it trades places with the
    # next rename (deterministic)
    seen = False
    for i in range(len(kinds)):
        if kinds[i] in ("rename", "move_folder"):
            seen = True
        elif kinds[i] == "rename_again" and not seen:
            j = next(j for j in range(i + 1, len(kinds)) if kinds[j] == "rename")
            kinds[i], kinds[j] = kinds[j], kinds[i]
            seen = True
    renamed: set[str] = set()
    n_ren = 0
    sessions = []
    for i, kind in enumerate(kinds):
        sid = f"t{i:03d}"
        tag = f"Human {sid}"
        names = sorted(vault.texts)
        doc = r.choice(names)
        steps: list[dict] = []
        facts = None
        if kind == "char":
            steps += [
                {"op": "type", "doc": doc, "text": r.choice("abcdefghijklmnopqrstuvwxyz")},
                {"op": "save", "how": r.choice(["command", "autosave"])},
            ]
        elif kind == "paragraph":
            steps += [
                {"op": "type", "doc": doc, "text": f"\n\n{p0.paragraph(r, tag)}"},
                {"op": "save", "how": "command"},
            ]
        elif kind == "multi":
            for k, d in enumerate(r.sample(names, r.randint(2, 3))):
                steps += [
                    {"op": "type", "doc": d, "text": f"\n\n{p0.paragraph(r, f'{tag}.{k}')}"},
                    {"op": "save", "how": "command"},
                ]
        elif kind == "rapid":
            for k in range(r.randint(3, 4)):
                steps += [
                    {"op": "type", "doc": doc, "text": f"\n\n{p0.sentence(r, f'{tag}.{k}')}"},
                    {"op": "save", "how": "autosave"},
                ]
        elif kind == "paste":
            big = "\n\n".join(
                p0.paragraph(r, f"{tag}.{k}", (2, 4)) for k in range(r.randint(8, 16))
            )
            steps += [
                {"op": "type", "doc": doc, "text": f"\n\n## {tag} pasted notes\n\n{big}"},
                {"op": "save", "how": "command"},
            ]
        elif kind == "focus":
            steps += [
                {"op": "type", "doc": doc, "text": f"\n\n{p0.paragraph(r, tag)}"},
                {"op": "save", "how": "blur"},
            ]
        else:
            if kind == "rename_again":
                doc = r.choice(sorted(renamed & set(names)))
            if kind == "move_folder":
                f = folder(doc)
                to = f"{ARCHIVE if f == FOLDER else FOLDER}/{PurePosixPath(doc).name}"
            else:
                n_ren += 1
                d = folder(doc)
                to = f"{d + '/' if d else ''}r{n_ren:03d}-{PurePosixPath(doc).name}"
            facts = {
                "self_link": any(lk["from"] == doc and lk["target"] == doc for lk in vault.links),
                "in_folder": folder(doc) != "",
                "incoming_md": any(
                    lk["target"] == doc and lk["from"] != doc and lk["form"].startswith("md_")
                    for lk in vault.links
                ),
                "incoming": sum(
                    1 for lk in vault.links if lk["target"] == doc and lk["from"] != doc
                ),
            }
            steps.append({"op": "rename", "doc": doc, "to": to})
            vault.rename(doc, to)
            renamed.discard(doc)
            renamed.add(to)
        s = {"id": sid, "kind": kind, "steps": steps}
        if facts is not None:
            s["facts"] = facts
        sessions.append(s)
    return {"seed": d_b_seed(), "set": "D-B test-3", "sessions": sessions}


def coverage() -> dict:
    """X0-16: the counts that must not be 0 before any session runs."""
    v3 = vault3()
    s = d_b3()["sessions"]
    ren = [x for x in s if x["kind"] in RENAMES]
    kinds = {k: sum(1 for x in s if x["kind"] == k) for k in dict.fromkeys(D_B3_KINDS)}
    facts = {
        "renames": len(ren),
        "self_link": sum(x["facts"]["self_link"] for x in ren),
        "in_folder": sum(x["facts"]["in_folder"] for x in ren),
        "incoming_md": sum(x["facts"]["incoming_md"] for x in ren),
        "self_link_by_kind": {
            k: sum(x["facts"]["self_link"] for x in ren if x["kind"] == k) for k in sorted(RENAMES)
        },
    }
    zero = [k for k, n in {**v3["counts"], **kinds}.items() if n == 0]
    zero += [f"facts.{k}" for k in ("self_link", "in_folder", "incoming_md") if facts[k] == 0]
    return {
        "seed": d_b_seed(),
        "link_counts": v3["counts"],
        "session_kinds": kinds,
        "rename_facts": facts,
        "zero": zero,
        "ok": not zero,
    }


def d_a_prime() -> dict:
    """D-A′ (X0-8): P2-0.1's definition (`p2_0_1_sets.d_a_prime`, Y0-5,
    Y0-10 b, c) on the test-3 vault with the links in the six forms, plus
    10 rename carry scenarios."""
    r = random.Random("P2-0.2-d-a-prime")
    v3 = vault3()
    texts = dict(v3["texts"])
    links = {p: set() for p in texts}
    for lk in v3["links"]:
        if lk["from"] != lk["target"]:
            links[lk["from"]].add(lk["target"])
    names = sorted(texts)
    kinds = ["in_agent_block"] * 25 + ["in_human_block"] * 15 + ["unobserved"] * 10
    r.shuffle(kinds)
    out = []

    def rename_in_place(t: str, sid: str) -> str:
        d = folder(t)
        return f"{d + '/' if d else ''}{sid}-{PurePosixPath(t).name}"

    def renamed(t: str, to: str) -> None:
        nonlocal names, links
        texts[to] = texts.pop(t)
        names = sorted(texts)
        links = {(to if k == t else k): {to if x == t else x for x in v} for k, v in links.items()}

    for i, kind in enumerate(kinds):
        sid = f"p{i:03d}"
        tag = f"Agent {sid}"
        n = r.choice(names)
        t = r.choice([x for x in names if x != n])
        form = r.choice(FORMS)
        link = render(form, t, texts, None, None)
        steps: list[dict] = []
        if kind == "in_human_block":
            if t not in links[n]:
                # Y0-10 c: the human first writes a linking paragraph in Obsidian
                steps.append({"op": "human_type", "doc": n, "text": f"\n\nHuman {sid} see {link}."})
                links[n].add(t)
            para = p0.paragraph(r, tag, (1, 2))
        else:
            para = f"{p0.paragraph(r, tag, (1, 2))} See {link}."
        delay = r.choice([0, 10, 25, 50]) if kind == "unobserved" else None
        steps.append({"op": "agent_append", "doc": n, "text": f"\n\n{para}\n"})
        to = rename_in_place(t, sid)
        steps.append({"op": "rename", "doc": t, "to": to, "delay_ms": delay})
        renamed(t, to)
        out.append(
            {"id": sid, "kind": kind, "form": form, "agent_doc": n, "target": t, "steps": steps}
        )
    # X0-8: 10 rename carry scenarios — the agent writes an unlinked
    # paragraph into T, observed `unverified`, then the human renames T
    for j in range(10):
        sid = f"p{50 + j:03d}"
        tag = f"Agent {sid}"
        t = r.choice(names)
        to = rename_in_place(t, sid) if j < 5 else f"{ARCHIVE}/{PurePosixPath(t).name}"
        if to in texts:
            raise ValueError(f"D-A′ carry: {to} exists")
        steps = [
            {
                "op": "agent_append",
                "doc": t,
                "text": f"\n\n{p0.paragraph(r, tag, (1, 2))}\n",
                "wait_unverified": True,
            },
            {"op": "rename", "doc": t, "to": to, "delay_ms": None},
        ]
        renamed(t, to)
        out.append(
            {"id": sid, "kind": "carry", "agent_doc": t, "target": t, "to": to, "steps": steps}
        )
    fakes = [
        {"id": f"f{j:03d}", "kind": "fake_link_update", "variant": v}
        for j, v in enumerate(FAKE_VARIANTS)
    ]
    return {"set": "D-A′", "vault": v3["texts"], "scenarios": out, "fakes": fakes}


def classify(text: str, renamed_to: str, in_renamed_note: bool) -> set[str]:
    """X0-18: the format classes of the links in a block's text (as Obsidian
    wrote it) that name the renamed note's new path; `self` too in the
    renamed note itself; `none` when there is no such link."""
    out: set[str] = set()
    t = stem(renamed_to)
    for m in re.finditer(r"(!?)\[\[([^\[\]|#]*)(#[^\[\]|]*)?(\|[^\[\]]*)?\]\]", text):
        if m.group(2).strip().split("/")[-1] != t:
            continue
        out.add(
            "embed"
            if m.group(1)
            else "wiki_alias"
            if m.group(4)
            else "wiki_heading"
            if m.group(3)
            else "wiki"
        )
    for m in re.finditer(r"\[([^\[\]\n]*)\]\(\s*<?([^\s()<>#]*)", text):
        dest = m.group(2)
        try:
            from urllib.parse import unquote

            dec = unquote(dest)
        except ValueError:
            dec = dest
        if stem(dec) != t:
            continue
        out.add("md_path" if "/" in dest else "md_name" if m.group(1) == stem(dec) else "md_label")
    if out and in_renamed_note:
        out.add("self")
    return out or {"none"}


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    from pathlib import Path

    o = Path(a.out)
    o.mkdir(parents=True, exist_ok=True)
    for name, obj in [
        ("vault3", vault3()),
        ("d_b3", d_b3()),
        ("d_a_prime", d_a_prime()),
        ("coverage", coverage()),
    ]:
        (o / f"{name}.json").write_text(json.dumps(obj, indent=1, ensure_ascii=False) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
