"""D-1 — classify every file of a lab commit for the lab -> product split.

The pattern logic of D-0's `research/d0/classify.py` (kept as it is: it is the
generator of D-0's record), here as functions over a list of paths. Three
lists in research/transfer/lists/ at the same commit: allowlist.txt (goes as
it is), stays.txt (stays in the lab), transforms.md (goes transformed; table
`| pattern | transform | product path |`). A file's class is the class of the
longest pattern that matches it; two classes at the same length is a double
classification (an error).
"""

from __future__ import annotations

import fnmatch

LISTS = "research/transfer/lists"


def parse_list(text: str, name: str):
    out = []
    for n, line in enumerate(text.splitlines(), 1):
        s = line.split("#", 1)[0].strip()
        if s:
            out.append((s, f"{name}:{n}"))
    return out


def parse_transforms(text: str, name: str = "transforms.md"):
    """Rows of '## Dönüşümler', '## İstisnalar' (home-path strings) and
    '## E-posta istisnaları' (e-mail addresses)."""
    rows, home_exc, mail_exc, sec = [], [], [], None
    for n, line in enumerate(text.splitlines(), 1):
        if line.startswith("## "):
            sec = line[3:].strip()
            continue
        if not line.startswith("| `"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        pat = cells[0].strip("`")
        at = f"{name}:{n}"
        if sec and sec.startswith("Dönüşümler"):
            kind = cells[1][:1]
            if kind not in ("M", "K", "Y"):
                raise ValueError(f"{at}: transform type must start with M, K or Y: {cells[1]!r}")
            dest = cells[2].strip("`") if len(cells) > 2 else ""
            rows.append((pat, at, kind, dest))
        elif sec and sec.startswith("İstisnalar"):
            home_exc.append({"file": pat, "string": cells[1].strip("`"), "why": cells[2], "at": at})
        elif sec and sec.startswith("E-posta"):
            mail_exc.append(
                {"file": pat, "address": cells[1].strip("`"), "why": cells[2], "at": at}
            )
    return rows, home_exc, mail_exc


def matches(pat: str, f: str) -> bool:
    if pat.endswith("/"):
        return f.startswith(pat)
    return f == pat or fnmatch.fnmatchcase(f, pat)


def dest_of(f: str, pat: str, dest: str) -> str:
    if dest in ("", "="):
        return f
    if pat.endswith("/") and dest.endswith("/"):
        return dest + f[len(pat) :]
    return dest


def classify(files, allow, stays, trans):
    """files: lab paths. Returns a dict with per-file class and product path,
    and the error lists (unclassified, double, collisions, dead patterns)."""
    pats = (
        [(p, "allow", at, None) for p, at in allow]
        + [(p, "stays", at, None) for p, at in stays]
        + [(p, "transform", at, (kind, dest)) for p, at, kind, dest in trans]
    )
    used = {at: 0 for _, _, at, _ in pats}
    out = {}
    unclassified, double = [], []
    for f in files:
        hits = [(len(p), c, at, p, x) for p, c, at, x in pats if matches(p, f)]
        if not hits:
            unclassified.append(f)
            continue
        best = max(h[0] for h in hits)
        top = [h for h in hits if h[0] == best]
        if len({h[1] for h in top}) > 1:
            double.append({"file": f, "patterns": sorted(h[2] for h in top)})
            continue
        _, c, at, p, x = top[0]
        used[at] += 1
        if c == "stays":
            out[f] = {"class": "stays"}
        elif c == "allow":
            out[f] = {"class": "allow", "kind": "=", "dest": f, "pattern": p}
        else:
            out[f] = {"class": "transform", "kind": x[0], "dest": dest_of(f, p, x[1]), "pattern": p}
    dests = [v["dest"] for v in out.values() if v["class"] != "stays"]
    seen, collisions = set(), []
    for d in dests:
        if d in seen:
            collisions.append(d)
        seen.add(d)
    return {
        "files": out,
        "unclassified": unclassified,
        "double": double,
        "collisions": sorted(set(collisions)),
        "dead_patterns": sorted(at for at, n in used.items() if n == 0),
    }
