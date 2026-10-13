"""D2 — real git history (EVAL §D2, H1-R).

`rust-lang/book` at a fixed rev; the last 300 non-merge commits touching
`src/*.md`; per commit only modified (`M`, no renames) markdown files. Each
(commit, file) pair stores the parent and child text and the `git diff -U0`
hunk ranges of the old and new side.

    python -m eval.datasets.git_history [--write-manifest]
"""

from __future__ import annotations

import argparse
import re
import shutil
from pathlib import Path

from eval.datasets.common import (
    DATA,
    clone_at,
    die,
    dump_json,
    file_blake3,
    git,
    verify_or_write_manifest,
)

REPO = "https://github.com/rust-lang/book"
REV = "1500248d8f23"
PATHSPEC = "src/*.md"
N_COMMITS = 300
SRC = DATA / "src" / "book"
D2 = DATA / "d2"
MANIFEST = DATA / "MANIFEST.d2.json"
HUNK_RE = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")


def parse_hunks(diff: str) -> list[dict]:
    """`-U0` hunk headers → `{"old": [start, count], "new": [start, count]}` (1-based lines)."""
    out = []
    for line in diff.splitlines():
        m = HUNK_RE.match(line)
        if m:
            a, b, c, d = m.groups()
            out.append(
                {
                    "old": [int(a), 1 if b is None else int(b)],
                    "new": [int(c), 1 if d is None else int(d)],
                }
            )
    return out


def build(src: Path = SRC, d2: Path = D2) -> dict:
    sha = clone_at(REPO, REV, src)
    if not sha.startswith(REV):
        raise RuntimeError(f"checked out {sha}, expected {REV}")
    commits = git(
        "log", "--no-merges", "--format=%H", "-n", str(N_COMMITS), sha, "--", PATHSPEC, cwd=src
    ).split()
    if len(commits) != N_COMMITS:
        raise RuntimeError(f"found {len(commits)} commits, expected {N_COMMITS}")
    if d2.exists():
        shutil.rmtree(d2)
    (d2 / "blobs").mkdir(parents=True)
    pairs = []
    # oldest first, so the pair list reads chronologically
    for c in reversed(commits):
        status = git("diff", "--no-renames", "--name-status", f"{c}^", c, "--", PATHSPEC, cwd=src)
        for line in status.splitlines():
            st, path = line.split("\t", 1)
            if st != "M":
                continue
            old = git("show", f"{c}^:{path}", cwd=src)
            new = git("show", f"{c}:{path}", cwd=src)
            names = []
            for text in (old, new):
                h = git("hash-object", "--stdin", cwd=src, input=text).strip()
                p = d2 / "blobs" / f"{h}.md"
                if not p.exists():
                    p.write_bytes(text.encode("utf-8"))
                names.append(p.name)
            diff = git("diff", "--no-renames", "-U0", f"{c}^", c, "--", path, cwd=src)
            pairs.append(
                {
                    "commit": c,
                    "path": path,
                    "old": names[0],
                    "new": names[1],
                    "hunks": parse_hunks(diff),
                }
            )
    (d2 / "pairs.json").write_text(dump_json(pairs), encoding="utf-8")
    files = {
        p.relative_to(d2).as_posix(): file_blake3(p) for p in sorted(d2.rglob("*")) if p.is_file()
    }
    return {
        "repo": REPO,
        "rev": sha,
        "pathspec": PATHSPEC,
        "commits": commits,
        "n_pairs": len(pairs),
        "files": files,
    }


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write-manifest", action="store_true")
    a = ap.parse_args(argv)
    m = build()
    verify_or_write_manifest(MANIFEST, m, a.write_manifest)
    print(f"D2: {len(m['commits'])} commits, {m['n_pairs']} pairs")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:  # noqa: BLE001 — CLI boundary: any failure is fatal
        die(str(e))
