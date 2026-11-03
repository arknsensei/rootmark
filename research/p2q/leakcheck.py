"""P2-Q content leak check (GOAL Q-9; TASKS Q0, Q4).

From every document of a corpus a seeded 32-byte piece is taken (one line,
cut at UTF-8 character boundaries, at least 8 letters). A piece already
present in the baseline (the repository at ROUND_START: text that was public
before the round) is generic and set aside, by count. The remaining pieces
are searched for in the targets — the repository tree, a PR's text, CI
artifacts — with `grep -F` / `git grep -F` reading the pieces from a file in
a private temporary directory that is removed afterwards. The method does not
depend on the corpus: any directory of Markdown files gives its pieces (the
verifier runs it with D-NOTES, VERIFY VQ3 (b)).

    python -m p2q.leakcheck --corpus <dir> [--corpus <dir> ...] \
        [--repo <git dir> --rev HEAD --baseline <rev>] [--path <file|dir> ...]

Output (stdout, JSON): per corpus the counts (documents, pieces, generic) and
per target the number of matching pieces and their blake2b — never a piece.
Exit 1 if any piece is found.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import subprocess
import sys
import tempfile
from pathlib import Path

from p2q.common import b2, list_md

PIECE = 32
SEED = 20261103


def piece_of(data: bytes, rng: random.Random) -> bytes | None:
    """A 32-byte single-line piece of `data` at a seeded offset, cut on UTF-8
    boundaries, with at least 8 letters; None if the document has none."""
    lines = [ln for ln in data.split(b"\n") if len(ln.strip()) >= PIECE]
    if not lines:
        return None
    for _ in range(16):
        ln = rng.choice(lines)
        start = rng.randrange(0, len(ln) - PIECE + 1)
        while start < len(ln) and (ln[start] & 0xC0) == 0x80:
            start += 1
        end = start + PIECE
        while end > start and end < len(ln) and (ln[end] & 0xC0) == 0x80:
            end -= 1
        cand = ln[start:end]
        try:
            text = cand.decode("utf-8")
        except UnicodeDecodeError:
            continue
        if len(cand) >= PIECE - 3 and sum(c.isalpha() for c in text) >= 8 and "\r" not in text:
            return cand
    return None


def pieces(corpus: Path, seed: int = SEED) -> tuple[list[bytes], int]:
    """(pieces, documents) of a corpus: one piece per document that has one."""
    out = []
    paths = list_md(corpus)
    for rel in paths:
        rng = random.Random(f"{seed}:{rel}")
        p = piece_of((corpus / rel).read_bytes(), rng)
        if p is not None:
            out.append(p)
    return sorted(set(out)), len(paths)


def _grep(
    cmd: list[str], pats: list[bytes], cwd: Path | None = None, tail: list[str] | None = None
) -> set[bytes]:
    """The patterns `cmd` (a grep -F -o ... -f <file>) finds."""
    if not pats:
        return set()
    with tempfile.TemporaryDirectory(prefix="p2q-leak-") as d:
        f = Path(d, "p")
        f.write_bytes(b"\n".join(pats) + b"\n")
        os.chmod(f, 0o600)
        r = subprocess.run(
            cmd + ["-f", str(f)] + (tail or []), check=False, cwd=cwd, capture_output=True
        )
    if r.returncode not in (0, 1):
        raise RuntimeError(f"{cmd[0]} failed ({r.returncode}): {r.stderr[:200]!r}")
    found = set()
    for line in r.stdout.split(b"\n"):
        # `git grep -o` prints "<rev>:<path>:<match>", grep -h -o "<match>"
        for p in pats:
            if p in line:
                found.add(p)
    return found


def in_git(repo: Path, rev: str, pats: list[bytes]) -> set[bytes]:
    return (
        _grep(["git", "grep", "-a", "-F", "-o", "-I", "-h"], pats, cwd=repo, tail=[rev])
        if pats
        else set()
    )


def _grep_paths(paths: list[Path], pats: list[bytes]) -> set[bytes]:
    with tempfile.TemporaryDirectory(prefix="p2q-leak-") as d:
        f = Path(d, "p")
        f.write_bytes(b"\n".join(pats) + b"\n")
        os.chmod(f, 0o600)
        r = subprocess.run(
            ["grep", "-r", "-a", "-F", "-o", "-h", "-f", str(f), "--"] + [str(p) for p in paths],
            check=False,
            capture_output=True,
        )
    if r.returncode not in (0, 1):
        raise RuntimeError(f"grep failed ({r.returncode}): {r.stderr[:200]!r}")
    return {p for p in pats if p in r.stdout}


def run(
    corpora: list[Path],
    repo: Path | None,
    rev: str,
    baseline: str | None,
    paths: list[Path],
    seed: int = SEED,
) -> dict:
    report = {"piece_bytes": PIECE, "seed": seed, "corpora": [], "targets": {}, "found": 0}
    allp: list[bytes] = []
    for c in corpora:
        ps, ndocs = pieces(c, seed)
        generic = in_git(repo, baseline, ps) if (repo and baseline) else set()
        kept = [p for p in ps if p not in generic]
        report["corpora"].append(
            {
                "corpus_b2": b2(str(c.name)),
                "documents": ndocs,
                "pieces": len(ps),
                "generic": len(generic),
                "checked": len(kept),
            }
        )
        allp += kept
    allp = sorted(set(allp))
    hits = {}
    if repo:
        hits[f"git:{rev}"] = in_git(repo, rev, allp)
    if paths:
        hits["paths"] = _grep_paths(paths, allp)
    for k, v in hits.items():
        report["targets"][k] = {"found": len(v), "found_b2": sorted(b2(x) for x in v)}
        report["found"] += len(v)
    report["pass"] = report["found"] == 0
    return report


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", type=Path, action="append", required=True)
    ap.add_argument("--repo", type=Path)
    ap.add_argument("--rev", default="HEAD")
    ap.add_argument("--baseline")
    ap.add_argument("--path", type=Path, action="append", default=[])
    ap.add_argument("--seed", type=int, default=SEED)
    a = ap.parse_args(argv)
    rep = run(a.corpus, a.repo, a.rev, a.baseline, a.path, a.seed)
    print(json.dumps(rep, indent=1, sort_keys=True))
    return 0 if rep["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
