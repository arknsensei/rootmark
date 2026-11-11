#!/usr/bin/env python3
"""D-0/R3 — classify every tracked file of a commit for the lab -> product split.

(docs/rounds/D-0/progress/DESIGN.md §3.) Three lists in
reports/rounds/D-0/transfer/: allowlist.txt (goes as it is), stays.txt
(stays in the lab), transforms.md (goes transformed; table
`| pattern | transform | product path |`). A file's class is the class of
the longest pattern that matches it; two classes at the same length is a
double classification (error). Then the leak check over the outgoing set
(allowlist + the product paths of transforms) and a relative-link check of
the outgoing Markdown files.

    python3 -I research/d0/classify.py [--commit <sha>] [--out <json>]

Exit 0 only if unclassified = 0, double = 0, every leak count = 0 and no
outgoing Markdown file links to a path that does not go. The record holds
counts and repository paths only; the host user's name and home directory
are read at run time to search for them and never written.
"""

from __future__ import annotations

import argparse
import fnmatch
import getpass
import json
import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[2]
TR = ROOT / "reports/rounds/D-0/transfer"
LEAK_PREFIXES = ["research/", "reports/", "docs/rounds/", "docs/archive/"]
KEY_GLOBS = [
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "id_rsa*",
    "id_ed25519*",
    ".env",
    ".env.*",
    "*secret*",
]
HOME_RE = re.compile(rb"/(?:Users|home)/([A-Za-z0-9._-]+)/")


def git(*a, binary=False):
    p = subprocess.run(["git", "-C", str(ROOT), *a], check=False, capture_output=True)
    if p.returncode != 0:
        sys.exit(f"git {' '.join(a)}: {p.stderr.decode()[-300:]}")
    return p.stdout if binary else p.stdout.decode()


def read_list(path: Path):
    out = []
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        s = line.split("#", 1)[0].strip()
        if s:
            out.append((s, f"{path.name}:{n}"))
    return out


def read_transforms(path: Path):
    """Rows of the first table under '## Dönüşümler' and of '## İstisnalar'."""
    rows, exc, sec = [], [], None
    for n, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if line.startswith("## "):
            sec = line[3:].strip()
            continue
        if not line.startswith("| `"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        pat = cells[0].strip("`")
        if sec and sec.startswith("Dönüşümler"):
            dest = cells[2].strip("`") if len(cells) > 2 else ""
            rows.append((pat, f"{path.name}:{n}", cells[1], dest))
        elif sec and sec.startswith("İstisnalar"):
            exc.append(
                {
                    "file": pat,
                    "string": cells[1].strip("`"),
                    "why": cells[2],
                    "at": f"{path.name}:{n}",
                }
            )
    return rows, exc


def matches(pat: str, f: str) -> bool:
    if pat.endswith("/"):
        return f.startswith(pat)
    return f == pat or fnmatch.fnmatchcase(f, pat)


def dest_of(f: str, pat: str, dest: str) -> str:
    """Product path of a transformed file: '=' keeps the path; for a directory
    pattern 'a/' -> 'b/' the rest of the path is kept; else the given path."""
    if dest in ("", "="):
        return f
    if pat.endswith("/") and dest.endswith("/"):
        return dest + f[len(pat) :]
    return dest


LINK_RE = re.compile(r"\]\(([^)\s]+)\)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--commit", default=(ROOT / "docs/rounds/D-0/progress/ROUND_START").read_text().strip()
    )
    ap.add_argument("--out", default=str(TR / "classification.json"))
    a = ap.parse_args()
    files = [x for x in git("ls-tree", "-r", "-z", "--name-only", a.commit).split("\0") if x]
    allow = read_list(TR / "allowlist.txt")
    stays = read_list(TR / "stays.txt")
    trans, exceptions = read_transforms(TR / "transforms.md")
    pats = (
        [(p, "allow", at, None) for p, at in allow]
        + [(p, "stays", at, None) for p, at in stays]
        + [(p, "transform", at, (how, dest)) for p, at, how, dest in trans]
    )
    used = {at: 0 for _, _, at, _ in pats}
    cls, unclassified, double, product = {}, [], [], {}
    for f in files:
        hits = [(len(p), c, at, p, x) for p, c, at, x in pats if matches(p, f)]
        if not hits:
            unclassified.append(f)
            continue
        best = max(h[0] for h in hits)
        top = [h for h in hits if h[0] == best]
        if len({h[1] for h in top}) > 1:
            double.append({"file": f, "patterns": [h[2] for h in top]})
            continue
        _, c, at, p, x = top[0]
        used[at] += 1
        cls[f] = c
        if c == "allow":
            product[f] = f
        elif c == "transform":
            product[f] = dest_of(f, p, x[1])
    goes = sorted(f for f, c in cls.items() if c == "allow")
    trn = sorted(f for f, c in cls.items() if c == "transform")
    dests = sorted(set(product.values()))
    collisions = len(product) - len(dests)
    # ---- leak check: the product paths, and the bytes of what goes as it is
    leaks = {"lab_prefix": [], "rootmark_dir": [], "server_token": [], "key_pattern": []}
    for d in dests:
        if any(d.startswith(p) for p in LEAK_PREFIXES):
            leaks["lab_prefix"].append(d)
        if ".rootmark" in PurePosixPath(d).parts:
            leaks["rootmark_dir"].append(d)
        name = PurePosixPath(d).name
        if name == "server.token":
            leaks["server_token"].append(d)
        if any(fnmatch.fnmatchcase(name, g) for g in KEY_GLOBS):
            leaks["key_pattern"].append(d)
    me = getpass.getuser().encode()
    home = os.path.expanduser("~").encode()
    host_hits, home_strings = 0, []
    exc_ok = {(e["file"], e["string"]) for e in exceptions}
    for f in sorted(product):
        blob = git("show", f"{a.commit}:{f}", binary=True)
        if home in blob or re.search(
            rb"(?<![A-Za-z0-9])" + re.escape(me) + rb"(?![A-Za-z0-9])", blob
        ):
            host_hits += 1
        for m in sorted(set(HOME_RE.findall(blob))):
            s = m.decode("utf-8", "replace")
            home_strings.append({"file": f, "user": s, "justified": (f, s) in exc_ok})
    unjustified = [h for h in home_strings if not h["justified"]]
    # ---- relative links of the Markdown files that go as they are
    goes_set = set(goes) | set(trn)
    broken = []
    for f in goes:
        if not f.endswith(".md") or "/fixtures" in f or "fixtures_variants/" in f:
            continue  # fixture notes are test data: their links are the test's subject
        text = git("show", f"{a.commit}:{f}")
        for t in LINK_RE.findall(text):
            if re.match(r"^[a-z]+:", t) or t.startswith("#"):
                continue
            target = os.path.normpath(os.path.join(os.path.dirname(f), t.split("#", 1)[0]))
            if target in (".", ""):
                continue
            if target not in goes_set and not any(
                x.startswith(target.rstrip("/") + "/") for x in goes_set
            ):
                broken.append({"file": f, "link": t})
    by_dir = {}
    for f, c in cls.items():
        top_dir = f.split("/", 1)[0] if "/" in f else "(kök)"
        by_dir.setdefault(top_dir, {"allow": 0, "stays": 0, "transform": 0})[c] += 1
    doc = {
        "classification": "D-0/R3 (docs/rounds/D-0/progress/DESIGN.md §3)",
        "script": "research/d0/classify.py",
        "commit": a.commit,
        "total": len(files),
        "goes": len(goes),
        "stays": sum(1 for c in cls.values() if c == "stays"),
        "transforms": len(trn),
        "unclassified": len(unclassified),
        "double_classified": len(double),
        "product_paths": len(dests),
        "product_path_collisions": collisions,
        "leaks": {k: len(v) for k, v in leaks.items()},
        "leak_files": {k: v for k, v in leaks.items() if v},
        "host_user_or_home_in_outgoing": host_hits,
        "home_path_strings": home_strings,
        "home_path_strings_unjustified": len(unjustified),
        "broken_relative_links_in_outgoing_md": broken,
        "by_top_dir": dict(sorted(by_dir.items())),
        "dead_patterns": sorted(at for at, n in used.items() if n == 0),
        "unclassified_files": unclassified[:200],
        "double_files": double[:200],
        "transformed_files": trn,
    }
    Path(a.out).write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    keys = (
        "total",
        "goes",
        "stays",
        "transforms",
        "unclassified",
        "double_classified",
        "product_path_collisions",
        "leaks",
        "host_user_or_home_in_outgoing",
        "home_path_strings_unjustified",
    )
    print(json.dumps({k: doc[k] for k in keys}, ensure_ascii=False))
    print(f"broken_links={len(broken)} dead_patterns={len(doc['dead_patterns'])}")
    bad = (
        unclassified
        or double
        or collisions
        or any(leaks.values())
        or host_hits
        or unjustified
        or broken
    )
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
