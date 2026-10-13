"""Shared helpers for dataset builders: paths, seed, hashing, manifests.

Manifest rule (EVAL §Veri): a builder regenerates its data, then compares the
result with the committed `data/MANIFEST.<name>.json`. A mismatch — or a
missing manifest without `--write-manifest` — is an error.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import blake3

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "data"
SEED = 20261013


class ManifestError(RuntimeError):
    pass


def blake3_hex(data: bytes) -> str:
    return blake3.blake3(data).hexdigest()[:32]


def file_blake3(path: Path) -> str:
    return blake3_hex(path.read_bytes())


def dump_json(obj) -> str:
    return json.dumps(obj, indent=1, sort_keys=True, ensure_ascii=False) + "\n"


def load_manifest(path: Path) -> dict:
    if not path.exists():
        raise ManifestError(f"{path} missing; run with --write-manifest on first build")
    return json.loads(path.read_text(encoding="utf-8"))


def verify_or_write_manifest(path: Path, manifest: dict, write: bool) -> None:
    """Writes `manifest` (with --write-manifest) or demands byte-equal content."""
    text = dump_json(manifest)
    if write:
        path.write_text(text, encoding="utf-8")
        print(f"wrote {path.relative_to(ROOT)}")
        return
    current = load_manifest(path)
    if current != manifest:
        diffs = sorted(k for k in set(current) | set(manifest) if current.get(k) != manifest.get(k))
        raise ManifestError(f"{path.name} mismatch in fields: {diffs}")
    print(f"{path.relative_to(ROOT)} verified")


# user-level git settings must not change what the builders see
GIT_PINNED = (
    "-c",
    "diff.algorithm=myers",
    "-c",
    "color.ui=never",
    "-c",
    "core.quotepath=off",
    "-c",
    "core.autocrlf=false",
    "-c",
    "log.showSignature=false",
)


def git(
    *args: str, cwd: Path | None = None, input: str | None = None, ok_codes: tuple[int, ...] = (0,)
) -> str:
    """Runs git; stdout decoded as UTF-8 without newline translation (file
    contents must reach the parser byte-for-byte)."""
    r = subprocess.run(
        ["git", *GIT_PINNED, *args],
        cwd=cwd,
        capture_output=True,
        input=None if input is None else input.encode("utf-8"),
        check=False,
    )
    if r.returncode not in ok_codes:
        raise RuntimeError(
            f"git {' '.join(args)} failed: {r.stderr.decode('utf-8', 'replace').strip()}"
        )
    return r.stdout.decode("utf-8")


def clone_at(url: str, rev: str, dest: Path, retries: int = 3) -> str:
    """Clone `url` into `dest` (if absent) and check out `rev`; returns full sha."""
    if not (dest / ".git").exists():
        dest.parent.mkdir(parents=True, exist_ok=True)
        last = None
        for _ in range(retries):
            r = subprocess.run(
                ["git", "clone", "--quiet", url, str(dest)],
                capture_output=True,
                text=True,
                check=False,
            )
            if r.returncode == 0:
                break
            last = r.stderr
        else:
            raise RuntimeError(f"clone of {url} failed {retries} times: {last}")
    git("checkout", "--quiet", "--detach", rev, cwd=dest)
    return git("rev-parse", "HEAD", cwd=dest).strip()


def die(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)
