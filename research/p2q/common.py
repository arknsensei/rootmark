"""Shared helpers of the P2-Q pilot (standard library only).

Personal corpora never enter a record: a path is recorded as `b2(path)`, a
file as `b2(bytes)`, a corpus as its summary (file count, bytes, one blake2b
over the sorted per-file pairs). Text of a personal corpus is never printed.
"""

from __future__ import annotations

import hashlib
import json
import os
import random
import shutil
import stat
from pathlib import Path

# directories that are not notes: rootmark's own store, Obsidian's config and
# trash (never copied as notes, never selected)
SKIP_DIRS = {".rootmark", ".obsidian", ".trash", ".git"}


def b2(data: bytes | str, size: int = 16) -> str:
    """blake2b hex (16-byte digest by default: 32 hex)."""
    if isinstance(data, str):
        data = data.encode("utf-8")
    return hashlib.blake2b(data, digest_size=size).hexdigest()


def b2_file(p: Path, size: int = 16) -> str:
    h = hashlib.blake2b(digest_size=size)
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def list_md(root: Path) -> list[str]:
    """Vault-relative POSIX paths of every *.md under root, byte-wise sorted
    (the order of `rootmark ingest-dir`), skipping SKIP_DIRS."""
    out = []
    root = Path(root)
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for name in filenames:
            if name.endswith(".md"):
                rel = Path(dirpath, name).relative_to(root).as_posix()
                out.append(rel)
    return sorted(out, key=lambda s: s.encode("utf-8"))


def corpus_summary(root: Path) -> dict:
    """{files, bytes, summary_b2}: summary over "b2(path) b2(bytes)\\n" lines in
    list_md order — identifies the copy without any of its text."""
    root = Path(root)
    h = hashlib.blake2b(digest_size=16)
    total = 0
    paths = list_md(root)
    for rel in paths:
        p = root / rel
        total += p.stat().st_size
        h.update(f"{b2(rel)} {b2_file(p)}\n".encode())
    return {"files": len(paths), "bytes": total, "summary_b2": h.hexdigest()}


def tree_b2(root: Path, skip: set[str] | None = None) -> dict[str, str]:
    """{relative path: b2(bytes)} of every file under root (for read-only
    proofs and md byte checks); `skip` names top-level entries to leave out."""
    root = Path(root)
    out = {}
    for dirpath, dirnames, filenames in os.walk(root):
        rel_dir = Path(dirpath).relative_to(root)
        if skip and rel_dir == Path("."):
            dirnames[:] = [d for d in dirnames if d not in skip]
        for name in filenames:
            rel = (rel_dir / name).as_posix()
            if skip and rel in skip:
                continue
            out[rel] = b2_file(Path(dirpath, name))
    return dict(sorted(out.items()))


def seeded_sample(items: list[str], n: int, seed: int, salt: str) -> list[str]:
    """n distinct items chosen by a fixed seed (+ a salt per use), in the
    order chosen; items must already be in a stable order."""
    rng = random.Random(f"{seed}:{salt}")
    return rng.sample(list(items), n)


def fresh_copy(src: Path, dst: Path) -> None:
    """Copy src to a new dst (dst must not exist) and make it writable (the
    snapshot under src is read-only)."""
    dst = Path(dst)
    if dst.exists():
        raise FileExistsError(f"{dst} exists")
    shutil.copytree(src, dst, symlinks=False)
    for dirpath, dirnames, filenames in os.walk(dst):
        for name in dirnames + filenames:
            p = Path(dirpath, name)
            p.chmod(p.stat().st_mode | stat.S_IWUSR)
    Path(dst).chmod(Path(dst).stat().st_mode | stat.S_IWUSR)


def dump(obj, path: Path) -> None:
    """Deterministic JSON record (sorted keys, LF, trailing newline)."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(obj, ensure_ascii=False, indent=1, sort_keys=True) + "\n", encoding="utf-8"
    )


def load(path: Path):
    return json.loads(Path(path).read_text(encoding="utf-8"))
